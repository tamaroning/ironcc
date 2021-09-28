#[derive(Debug, Clone)]
pub enum AST {
    Int(i32),
    Float(f64),
    BinaryOp(Box<AST>, Box<AST>, BinaryOp),
    Variable(String),
    Return(Box<AST>),
    ExprStmt(Box<AST>),
    Block(Vec<AST>),
    If(Box<AST>, Box<AST>, Box<AST>), // cond, then, els
    For(Box<AST>, Box<AST>, Box<AST>, Box<AST>), // init, cond, step, body
    While(Box<AST>, Box<AST>), // cond, body
    Nil, // forのcond、ifのelse、expr-stmtのexprにおいて式や文などが存在しないときに用いる
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
    Assign,
}

impl AST {
    pub fn eval_const_expr(&self) -> i32 {
        match &self {
            AST::Int(n) => *n,
            AST::BinaryOp(l, r, op) => {
                let l = l.eval_const_expr();
                let r = r.eval_const_expr();
                match op {
                    &BinaryOp::Add => l + r,
                    &BinaryOp::Sub => l - r,
                    &BinaryOp::Mul => l * r,
                    &BinaryOp::Div => l / r,
                    &BinaryOp::Eq => (l == r) as i32,
                    &BinaryOp::Ne => (l != r) as i32,
                    &BinaryOp::Lt => (l < r) as i32,
                    &BinaryOp::Le => (l <= r) as i32,
                    _ => panic!("Unknown operator"),
                }

            },
            _ => panic!("Expected constant expression"),
        }
    }
}