#[derive(Debug, Clone)]
pub enum AST {
    Num(f64),
    Variable(String),
    BinaryOp(BinaryOpAST),
}

#[derive(Debug, Clone)]
pub struct BinaryOpAST {
    pub lhs: Box<AST>,
    pub rhs: Box<AST>,
    pub op: BinaryOp,
}

#[derive(Debug, Clone)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
}

impl BinaryOpAST {
    pub fn new(l: Box<AST>, r: Box<AST>, o: BinaryOp) -> BinaryOpAST{
        BinaryOpAST {
            lhs: l,
            rhs: r, 
            op: o,
        }
    }
}