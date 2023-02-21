use std::collections::BTreeMap;
use std::sync::{Arc, RwLock};
use threadpool::ThreadPool;

use crate::ast::*;
use crate::geometry::Solid;

pub struct FuncArgs {
    pub pos: Vec<Value>,
    pub named: BTreeMap<String, Value>,
    pub children: Vec<Value>,
}

pub trait BuiltinFunc {
    fn is_heavy(&self) -> bool {
        false
    }

    fn call(&self, args: &FuncArgs) -> std::result::Result<Value, String>;
}

#[derive(Clone, Debug)]
pub struct Error {
    node: Arc<Node>,
    reason: String,
}

type Result = std::result::Result<Value, Error>;

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

struct CacheEntry {
    value: Value,
}

pub struct Executor {
    pool: ThreadPool,
    cache: RwLock<BTreeMap<usize, Arc<CacheEntry>>>,
}

impl Executor {
    fn new() -> Executor {
        Executor {
            pool: ThreadPool::new(8),
            cache: RwLock::new(BTreeMap::new()),
        }
    }
}

struct Env {
    executor: Arc<Executor>,
    parent: Option<Arc<Env>>,
    vars: BTreeMap<String, Value>,
}

impl Env {
    fn new(executor: Arc<Executor>) -> Self {
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

    fn child(self: &Arc<Env>, vars: BTreeMap<String, Value>) -> Arc<Env> {
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
            let mut var = BTreeMap::new();
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

            let args = FuncArgs {
                pos: call
                    .pos_args
                    .iter()
                    .map(|expr| exec_expr(env.clone(), expr))
                    .collect::<std::result::Result<Vec<_>, _>>()?,

                named: call
                    .named_args
                    .iter()
                    .map(|(name, expr)| exec_expr(env.clone(), expr).map(|val| (name.clone(), val)))
                    .collect::<std::result::Result<BTreeMap<_, _>, _>>()?,

                children: call
                    .body
                    .iter()
                    .map(|expr| exec_expr(env.clone(), expr))
                    .collect::<std::result::Result<Vec<_>, _>>()?,
            };

            func.call(&args).map_err(|e| err(node, e))
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
    let env = Arc::new(Env::new(Arc::new(Executor::new())));

    exec_body(
        env.child(crate::builtins::get_builtins()),
        &[Arc::new(Node {
            pos: 0..0,
            expr: Expr::Call(CallExpr {
                name: "union".to_string(),
                pos_args: Vec::new(),
                named_args: BTreeMap::new(),
                body: nodes.to_vec(),
            }),
        })],
    )
}
