use std::collections::HashMap;
use crate::types::Type;

#[derive(Debug, Clone)]
pub enum AST {
    Int(i32),
    Float(f64),
    BinaryOp(Box<AST>, Box<AST>, BinaryOps),
    UnaryOp(Box<AST>, UnaryOps),
    Variable(String),
    Return(Box<AST>),
    ExprStmt(Box<AST>),
    Block(Vec<AST>),
    If(Box<AST>, Box<AST>, Box<AST>), // cond, then, els
    For(Box<AST>, Box<AST>, Box<AST>, Box<AST>), // init, cond, step, body
    While(Box<AST>, Box<AST>), // cond, body
    FuncCall(String, Vec<AST>), // func-name, args
    FuncDef(Type, Vec<String>, String, Box<AST>, HashMap<String, Type>), // type, arg names, func name, body 
    Nil, // forのcond、ifのelse、expr-stmtのexprにおいて式や文などが存在しないときに用いる
}

#[derive(Debug, Clone)]
pub enum BinaryOps {
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

#[derive(Debug, Clone)]
pub enum UnaryOps {
    Plus, // +
    Minus, // -
    Addr, // &
    Deref, // *
}

impl AST {
    pub fn eval_const_expr(&self) -> i32 {
        match &self {
            AST::Int(n) => *n,
            AST::BinaryOp(l, r, op) => {
                let l = l.eval_const_expr();
                let r = r.eval_const_expr();
                match op {
                    &BinaryOps::Add => l + r,
                    &BinaryOps::Sub => l - r,
                    &BinaryOps::Mul => l * r,
                    &BinaryOps::Div => l / r,
                    &BinaryOps::Eq => (l == r) as i32,
                    &BinaryOps::Ne => (l != r) as i32,
                    &BinaryOps::Lt => (l < r) as i32,
                    &BinaryOps::Le => (l <= r) as i32,
                    _ => panic!("Unknown operator"),
                }

            },
            _ => panic!("Expected constant expression"),
        }
    }
}