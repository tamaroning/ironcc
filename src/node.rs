#[derive(Debug, Clone)]
pub enum AST {
    Num(f64),
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
