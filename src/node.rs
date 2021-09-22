#[derive(Debug, Clone)]
pub enum AST {
    Int(i32),
    Float(f64),
    BinaryOp(Box<AST>, Box<AST>, BinaryOp),
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
