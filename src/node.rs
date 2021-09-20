#[derive(Debug, Clone)]
pub enum AST {
    Num(f64),
    BinaryOp(Box<AST>, Box<AST>, BinaryOp),
    Variable(String),
    FuncCall(String, Vec<AST>),
    Prototype(String, Vec<String>),
    Def(String, Vec<String>, Box<AST>),
    Extern(String, Vec<String>),
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq, // ==
    Ne, // !=
    Lt, // <
    Le, // <=
}
