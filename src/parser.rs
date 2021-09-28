use std::fmt::Binary;

use crate::lexer;
use crate::node;
use crate::node::UnaryOps;

use lexer::Token;
use lexer::TokenKind;
use node::{AST, BinaryOps};

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
    pub fn cur(&self) -> Token {
        if self.pos < self.tokens.len() {
            return self.tokens[self.pos].clone();
        }
        panic!("Couldn't read a token");
    }

    // for LL(2)
    pub fn peek(&self) -> Token {
        if self.pos < self.tokens.len() {
            return self.tokens[self.pos + 1].clone();
        }
        panic!("Couldn't read a token");
    }

    pub fn next(&mut self) -> Token {
        let ret = self.cur();
        self.pos += 1;
        ret
    }

    // read forward
    pub fn consume(&mut self, s: &str) -> bool {
        if self.cur().matches(s) {
            self.next();
            return true;
        }
        false
    }

    pub fn consume_expected(&mut self, s: &str) -> bool {
        if self.cur().matches(s) {
            self.next();
        } else {
            panic!("Unexpected token");
        }
    }

    //
    // ---------------- Generate AST ----------------
    //

    fn read_toplevel(&mut self) -> Vec<AST> {
        let mut ret = Vec::new();
        while !self.cur().is_eof() {
           ret.push(self.read_stmt());
        }
        ret
    }

    fn read_stmt(&mut self) -> AST {
        if self.consume("return") {
            let expr = self.read_expr();
            self.consume_expected(";");
            return AST::Return(Box::new(expr));
        } else if self.consume("if") {
            self.consume_expected("(");
            let cond = self.read_expr();
            self.consume_expected(")");
            let then = self.read_stmt();
            let mut els = AST::Nil;
            if self.consume("else") {
                els = self.read_stmt();
            }
            return AST::If(Box::new(cond), Box::new(then), Box::new(els));
        } else if self.consume("for") {
            self.consume_expected("(");
            let init = self.read_expr_stmt();
            let cond = AST::Nil;
            if !self.consume(";") {
                cond = self.read_expr();
                self.consume_expected(";");
            }
            let step = AST::Nil;
            if !self.consume(")") {
                step = self.read_expr();
                self.consume(")");
            }
            let body = self.read_stmt();
            return AST::For(Box::new(init), Box::new(cond), Box::new(step), Box::new(body));
        } else if self.consume("while") {
            self.consume_expected("(");
            let cond = self.read_expr();
            self.consume_expected(")");
            let body = self.read_stmt();
            return AST::While(Box::new(cond), Box::new(body)); 
        } else if self.consume("{") {
            return self.read_compound_stmt();
        } else {
            return self.read_expr_stmt();
        }
    }

    fn read_compound_stmt(&mut self) -> AST {
        let mut v = Vec::new();
        while !self.consume("}") {
            v.push(self.read_stmt());
        }
        AST::Block(v)
    }

    fn read_expr_stmt(&mut self) -> AST {
        if self.consume(";") {
            return AST::Nil;
        } else {
            let expr = self.read_expr();
            self.consume_expected(";");
            return AST::ExprStmt(Box::new(expr));
        }
    }

    fn read_expr(&mut self) -> AST {
        self.read_assign()
    }

    fn read_assign(&mut self) -> AST {
        let mut ret = self.read_equality();
        if self.consume("=") {
            let rhs = self.read_assign();
            ret = AST::BinaryOp(ret, rhs, BinaryOps::Assign); 
        }
        ret
    }

    fn read_equality(&mut self) -> AST {
        let mut ast = self.read_relational();
        loop {
            if self.consume("==") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_relational()),
                    BinaryOps::Eq,
                );
            } else if self.consume("!=") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_relational()),
                    BinaryOps::Ne,
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
                    BinaryOps::Lt,
                );
            } else if self.consume("<=") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_add()),
                    BinaryOps::Le,
                );
            } else if self.consume(">") {
                ast = AST::BinaryOp(
                    Box::new(self.read_add()),
                    Box::new(ast),
                    BinaryOps::Lt,
                );
            } else if self.consume(">=") {
                ast = AST::BinaryOp(
                    Box::new(self.read_add()),
                    Box::new(ast),
                    BinaryOps::Le,
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
                    BinaryOps::Add,
                );
            } else if self.consume("-") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_mul()),
                    BinaryOps::Sub,
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
                    BinaryOps::Mul,
                );
            } else if self.consume("/") {
                ast = AST::BinaryOp(
                    Box::new(ast),
                    Box::new(self.read_unary()),
                    BinaryOps::Div,
                );
            } else {
                break;
            }
        }
        ast
    }

    fn read_unary(&mut self) -> AST {
        if self.consume("+") {
            return AST::UnaryOp(self.read_unary(), UnaryOps::Plus);
        } else if self.consume("-") {
            return AST::UnaryOp(self.read_unary(), UnaryOps::Minus);
        } else if self.consume("&") {
            return AST::UnaryOp(self.read_unary(), UnaryOps::Addr);
        } else if self.consume("*") {
            return AST::UnaryOp(self.read_unary(), UnaryOps::Deref);
        }
        self.read_primary()
    }

    fn read_primary(&mut self) -> AST {
        if self.consume("(") {
            let ast = self.read_expr();
            self.consume_expected(")");
            return ast;
        } else if self.cur().is_ident() {
            return AST::Variable(self.cur().val);
        }else{
            return self.read_num();
        }
    }

    fn read_num(&mut self) -> AST {
        match self.next() {
            Token{ kind: TokenKind::IntNum, val: n, ..}  => AST::Int(n.parse::<i32>().unwrap()),
            Token{ kind: TokenKind::FloatNum, val: n, ..}  => AST::Float(n.parse::<f64>().unwrap()),
            _ => panic!("Numerical literal is expected"),
        } 
    }
}