use crate::lexer;
use crate::node;

use lexer::Token;
use lexer::TokenKind;
use node::{AST, BinaryOp};

pub fn run(filepath: String, tokens: Vec<Token>) -> Vec<AST> {
    let mut parser = Parser::new(filepath, tokens);
    let ast = parser.read_toplevel();
    ast
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

    // toplevel = expr | ;
    fn read_toplevel(&mut self) -> Vec<AST> {
        let mut ret = Vec::new();
        while !self.cur().unwrap().is_eof() {
            if self.consume(";") {
                continue;
            } else {
                ret.push(self.read_expr());
            }
        }
        ret
    }

    fn read_expr(&mut self) -> AST {
        self.read_equality()
    }

    fn read_equality(&mut self) -> AST {
        let mut ast = self.read_relational();
        loop {
            if self.consume("==") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_relational()),
                    BinaryOp::Eq,
                );
            } else if self.consume("!=") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_relational()),
                    BinaryOp::Ne,
                );
            } else {
                break;
            }
        }
        ast
    }

    fn read_relational(&mut self) -> AST {
        let mut ast = self.read_add();
        loop {
            if self.consume("<") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_add()),
                    BinaryOp::Lt,
                );
            } else if self.consume("<=") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_add()),
                    BinaryOp::Le,
                );
            } else if self.consume(">") {
                ast = AST::BinaryOp(
                    Box::new(self.read_add()),
                    Box::new(ast),
                    BinaryOp::Lt,
                );
            } else if self.consume(">=") {
                ast = AST::BinaryOp(
                    Box::new(self.read_add()),
                    Box::new(ast),
                    BinaryOp::Le,
                );
            } else {
                break;
            }
        }
        ast
    }

    fn read_add(&mut self) -> AST {
        let mut ast = self.read_mul();
        loop {
            if self.consume("+") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_mul()),
                    BinaryOp::Add,
                );
            } else if self.consume("-") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_mul()),
                    BinaryOp::Sub,
                );
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
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_unary()),
                    BinaryOp::Mul,
                );
            } else if self.consume("/") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_unary()),
                    BinaryOp::Div,
                );
            } else {
                break;
            }
        }
        ast
    }

    fn read_unary(&mut self) -> AST {
        if self.consume("-") {
            return AST::BinaryOp(
                Box::new(AST::Int(0)),
                Box::new(self.read_primary()),
                BinaryOp::Sub,
            );
        }
        self.consume("+");
        self.read_primary()
    }

    // primary ::= ( expr )
    //           | num
    fn read_primary(&mut self) -> AST {
        if self.consume("(") {
            let ast = self.read_expr();
            self.consume(")");
            return ast;
        } else{
            return self.read_num();
        }
    }

    fn read_num(&mut self) -> AST {
        match self.next() {
            Token{ kind: TokenKind::IntNum, val: n, ..}  => AST::Int(n.parse::<i32>().unwrap()),
            _ => panic!("Numerical literal is expected"),
        } 
    }

    fn read_id(&mut self) -> String {
        self.next().val
    }
}