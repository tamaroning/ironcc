use crate::lexer;
use crate::node;

use lexer::Token;
use lexer::TokenKind;
use node::{AST, BinaryOpAST, BinaryOp};

pub fn run(filepath: String) -> Vec<AST> {
    let tokens = lexer::run(filepath.clone());
    let nodes = Vec::new();
    
    let mut parser = Parser::new(filepath, tokens);
    let ast = parser.read_toplevel();

    // test
    println!("{:?}", ast);

    nodes
}

pub struct Parser {
    filepath: String,
    pos: usize,
    tokens: Vec<Token>,
}

impl Parser {
    pub fn new(path: String, tok: Vec<Token>) -> Parser {
        Parser {
            filepath: path,
            pos: 0,
            tokens: tok,
        }
    }

    pub fn get_filepath(&self) -> String {
        self.filepath.clone()
    }

    // for LL(1)
    pub fn cur(&self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            return Some(self.tokens[self.pos].clone());
        }
        None
    }

    // for LL(2)
    pub fn peek(&self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            return Some(self.tokens[self.pos + 1].clone());
        }
        None
    }

    pub fn next(&mut self) -> Token {
        let ret = self.cur().unwrap();
        self.pos += 1;
        ret
    }

    // read forward
    pub fn consume(&mut self, s: &str) -> bool {
        if self.cur().unwrap().matches(s) {
            self.next();
            return true;
        }
        false
    }

    fn read_toplevel(&mut self) -> AST {
        self.read_expr()
    }

    fn read_expr(&mut self) -> AST {
        let mut ast = self.read_mul();
        loop {
            if self.consume("+") {
                ast = AST::BinaryOp(BinaryOpAST{
                    lhs: Box::new(ast),
                    rhs: Box::new(self.read_mul()),
                    op: BinaryOp::Add,
                });
            } else if self.consume("-") {
                ast = AST::BinaryOp(BinaryOpAST{
                    lhs: Box::new(ast),
                    rhs: Box::new(self.read_mul()),
                    op: BinaryOp::Sub,
                });
            } else {
                break;
            }
        }
        ast
    }

    fn read_mul(&mut self) -> AST {
        let mut ast = self.read_unary();
        loop {
            if self.consume("*") {
                ast = AST::BinaryOp(BinaryOpAST{
                    lhs: Box::new(ast),
                    rhs: Box::new(self.read_unary()),
                    op: BinaryOp::Mul,
                });
            } else if self.consume("/") {
                ast = AST::BinaryOp(BinaryOpAST{
                    lhs: Box::new(ast),
                    rhs: Box::new(self.read_unary()),
                    op: BinaryOp::Div,
                });
            } else {
                break;
            }
        }
        ast
    }

    fn read_unary(&mut self) -> AST {
        if self.consume("-") {
            return AST::BinaryOp(BinaryOpAST{
                lhs: Box::new(AST::Num(0.0)),
                rhs: Box::new(self.read_primary()),
                op: BinaryOp::Sub,
            });
        }
        self.consume("+");
        self.read_primary()
    }

    fn read_primary(&mut self) -> AST {
        if self.consume("(") {
            let ast = self.read_expr();
            self.consume(")");
            return ast;
        }
        return self.read_num()
    }

    fn read_num(&mut self) -> AST {
        match self.next().kind {
            TokenKind::Num(f) => AST::Num(f),
            _ => panic!("Numerical literal is expected"),
        } 
    }
}