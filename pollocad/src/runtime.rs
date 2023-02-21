use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use fxhash::FxBuildHasher;
use threadpool::ThreadPool;

use crate::ast::*;
use crate::geometry::Solid;

type Result = std::result::Result<Value, Error>;

struct CacheEntry {
    used: bool,
    value: Value,
}

pub struct Runtime {
    pool: ThreadPool,
    cache: RwLock<HashMap<usize, CacheEntry, FxBuildHasher>>,
}

impl Runtime {
    fn new() -> Runtime {
        Runtime {
            pool: ThreadPool::new(8),
            cache: RwLock::new(HashMap::with_hasher(FxBuildHasher::default())),
        }
    }

    /*pub fn exec(nodes: &[Arc<Node>]) -> Result {
    }*/
}

pub struct CallCtx<'a> {
    pub pos: &'a [Value],
    pub named: &'a HashMap<String, Value>,
    pub children: &'a [Value],
    pub is_heavy: bool,
}

impl CallCtx<'_> {
    fn heavy(&mut self) {
        self.is_heavy = true;
    }
}

pub trait BuiltinFunc {
    fn is_heavy(&self) -> bool {
        false
    }

    fn call(&self, args: &mut CallCtx) -> std::result::Result<Value, String>;
}

#[derive(Clone, Debug)]
pub struct Error {
    node: Arc<Node>,
    reason: String,
}

fn err(node: &Arc<Node>, reason: impl Into<String>) -> Error {
    Error {
        node: node.clone(),
        reason: reason.into(),
    }
}

#[derive(Clone)]
pub enum Value {
    Undefined,
    Num(f64),
    BuiltinFunc(Arc<dyn BuiltinFunc>),
    Solid(Arc<Solid>),
}

struct Env {
    executor: Arc<Runtime>,
    parent: Option<Arc<Env>>,
    vars: HashMap<String, Value>,
}

impl Env {
    fn new(executor: Arc<Runtime>) -> Self {
        Env {
            executor,
            parent: None,
            vars: Default::default(),
        }
    }

    fn get(&self, name: &str) -> Option<Value> {
        self.vars
            .get(name)
            .cloned()
            .or_else(|| self.parent.as_ref().and_then(|p| p.get(name)))
    }

    fn child(self: &Arc<Env>, vars: HashMap<String, Value>) -> Arc<Env> {
        Arc::new(Env {
            executor: self.executor.clone(),
            parent: Some(self.clone()),
            vars,
        })
    }
}

/*impl std::hash::Hash for Expr {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        match self {
            Expr::Let(n) => {
                n.name.hash(h);
                n.value.expr.hash(h);
                n.body.hash(h);
            },

            _ => unimplemented!(),
        }
    }
}*/

fn is_body_heavy(env: &Env, node: &[Arc<Node>]) -> bool {
    node.iter().any(|n| is_node_heavy(env, n))
}

fn is_node_heavy(env: &Env, node: &Node) -> bool {
    match &node.expr {
        Expr::Let(let_) => is_node_heavy(env, &let_.value) || is_body_heavy(env, &let_.body),
        //Expr::UnOp(op) => is_node_heavy(env, &op.operand),
        //Expr::BinOp(op) => is_node_heavy(env, &op.left) || is_node_heavy(env, &op.right),
        Expr::Return(value) => is_node_heavy(env, &value),
        Expr::Call(call) => match env.get(&call.name) {
            Some(Value::BuiltinFunc(func)) => func.is_heavy(),
            _ => false,
        },
        _ => false,
    }
}

fn exec_expr(env: Arc<Env>, node: &Arc<Node>) -> Result {
    match &node.expr {
        Expr::Let(let_) => {
            let mut var = HashMap::new();
            var.insert(let_.name.clone(), exec_expr(env.clone(), &let_.value)?);

            exec_body(env.child(var), let_.body.as_ref())
        }
        Expr::Var(name) => env
            .get(&name)
            .ok_or_else(|| err(node, format!("Variable {} does not exist", name))),
        Expr::Num(num) => Ok(Value::Num(*num)),
        Expr::Call(call) => {
            let func = env
                .get(&call.name)
                .ok_or_else(|| err(node, format!("Function {} does not exist", call.name)))?;

            let Value::BuiltinFunc(func) = func else { return Err(err(node, "")) };

            let pos_args = call
                .args
                .iter()
                .filter_map(|(name, expr)| name.as_ref().map_or_else(|| Some(expr), |_| None))
                .map(|expr| exec_expr(env.clone(), &expr))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            let named_args = call
                .args
                .iter()
                .filter_map(|(name, expr)| name.as_ref().map(|name| (name, expr)))
                .map(|(name, expr)| exec_expr(env.clone(), expr).map(|val| (name.clone(), val)))
                .collect::<std::result::Result<HashMap<_, _>, _>>()?;

            let children = call
                .body
                .iter()
                .map(|expr| exec_expr(env.clone(), expr))
                .collect::<std::result::Result<Vec<_>, _>>()?;

            let mut args = CallCtx {
                pos: &pos_args,
                named: &named_args,
                children: &children,
                is_heavy: false,
            };

            func.call(&mut args).map_err(|e| err(node, e))
        }
        Expr::Return(node) => exec_expr(env, node),
    }
}

fn exec_body(env: Arc<Env>, nodes: &[Arc<Node>]) -> Result {
    let mut geo = vec![];

    for node in nodes {
        let v = match node.as_ref() {
            Node {
                expr: Expr::Return(n),
                ..
            } => {
                if !geo.is_empty() {
                    return Err(err(
                        node,
                        "A body may either produce geometry or return a value, but not both",
                    ));
                }

                return exec_expr(env, n);
            }
            _ => exec_expr(env.clone(), node)?,
        };

        match v {
            Value::Solid(s) => geo.push(s),
            _ => {}
        }
    }

    Ok(Value::Solid(Arc::new(Solid::combine(
        geo.iter().map(|s| s.as_ref()),
    ))))
}

pub fn exec(nodes: &[Arc<Node>]) -> Result {
    let env = Arc::new(Env::new(Arc::new(Runtime::new())));

    exec_body(
        env.child(crate::builtins::get_builtins()),
        &[Arc::new(Node {
            pos: 0..0,
            expr: Expr::Call(CallExpr {
                name: "union".to_string(),
                args: Vec::new(),
                body: nodes.to_vec(),
            }),
        })],
    )
}
