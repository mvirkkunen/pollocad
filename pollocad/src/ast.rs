use std::ops::Range;
use std::sync::Arc;

#[derive(PartialEq, Clone, Debug)]
pub struct Node {
    pub pos: Range<usize>,
    pub expr: Expr,
}

/*impl std::hash::Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.expr.hash(h);
    }
}*/

#[derive(PartialEq, Clone, Debug)]
pub enum Expr {
    Let(LetExpr),
    Call(CallExpr),
    Var(String),
    Num(f64),
    //UnOp(UnOpExpr),
    //BinOp(BinOpExpr),
    Return(Arc<Node>),
}

#[derive(PartialEq, Clone, Debug)]
pub struct LetExpr {
    pub name: String,
    pub value: Arc<Node>,
    pub body: Vec<Arc<Node>>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct CallExpr {
    pub name: String,
    //pub pos_args: Vec<Arc<Node>>,
    pub args: Vec<(Option<String>, Arc<Node>)>,
    pub body: Vec<Arc<Node>>,
}

/*#[derive(PartialEq, Clone, Debug)]
pub struct UnOpExpr {
    pub op: String,
    pub operand: Arc<Node>,
}

#[derive(PartialEq, Clone, Debug)]
pub struct BinOpExpr {
    pub op: String,
    pub left: Arc<Node>,
    pub right: Arc<Node>,
}*/

/*#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum UnOp {
    Neg,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}*/
