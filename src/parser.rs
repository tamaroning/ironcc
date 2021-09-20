use crate::lexer;
use crate::node;

use lexer::Token;
use lexer::TokenKind;
use node::{AST, BinaryOp};

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

    // toplevel = definition | external | expr | ;
    fn read_toplevel(&mut self) -> Vec<AST> {
        let mut ret = Vec::new();
        while !self.cur().unwrap().is_eof() {
            if self.consume(";") {
                continue;
            } else if self.cur().unwrap().matches("def") {
                ret.push(self.read_definition());
            } else if self.cur().unwrap().matches("extern") {
                ret.push(self.read_external());
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
                Box::new(AST::Num(0.0)),
                Box::new(self.read_primary()),
                BinaryOp::Sub,
            );
        }
        self.consume("+");
        self.read_primary()
    }

    // primary ::= ( expr )
    //           | ident_expr
    //           | num
    fn read_primary(&mut self) -> AST {
        if self.consume("(") {
            let ast = self.read_expr();
            self.consume(")");
            return ast;
        } else if self.cur().unwrap().is_ident() {
            return self.read_ident_expr();
        } else{
            return self.read_num();
        }
    }

    // ident_expr ::= id ( expr )
    //              | id
    fn read_ident_expr(&mut self) -> AST {
        if self.peek().unwrap().matches("(") {
            let ident = self.read_id();
            self.consume("(");
            let mut v = Vec::new();
            while !self.consume(")") {
                v.push(self.read_expr());
            }
            return AST::FuncCall(ident, v);
        }
        let ident = self.next().val;

        AST::Variable(ident)
    }

    fn read_num(&mut self) -> AST {
        match self.next().kind {
            TokenKind::Num(f) => AST::Num(f),
            _ => panic!("Numerical literal is expected"),
        } 
    }

    // definition ::= def prototype expr
    fn read_definition(&mut self) -> AST {
        self.consume("def");
        let (id, args) = self.read_prototype();
        let expr = self.read_expr();
        AST::Def(id, args , Box::new(expr))
    }

    // prototype ::= id ( id* )
    fn read_prototype(&mut self) -> (String, Vec<String>) {
        let id = self.read_id();
        let mut v = Vec::new();
        self.consume("(");
        while !self.consume(")") {
            v.push(self.read_id());
        }
        (id, v)
    }

    fn read_id(&mut self) -> String {
        self.next().val
    }

    // external ::= extern prototype
    fn read_external(&mut self) -> AST {
        self.consume("extern");
        let (id, args) = self.read_prototype();
        AST::Extern(id, args)
    }
}