use std::fmt::Binary;
use std::collections::HashMap;

use crate::lexer;
use crate::node;
use crate::node::UnaryOps;
use crate::types::Type;

use lexer::Token;
use lexer::TokenKind;
use node::{AST, BinaryOps};

pub fn run(filepath: String, tokens: Vec<Token>) -> Vec<AST> {
    let mut parser = Parser::new(filepath, tokens);
    let ast = parser.read_program();
    println!("locals: {:?}", parser.locals);
    ast
}

pub struct Parser {
    filepath: String,
    pos: usize,
    tokens: Vec<Token>,
    pub locals: HashMap<String, Type>,
}

impl Parser {
    pub fn new(path: String, tok: Vec<Token>) -> Parser {
        Parser {
            filepath: path,
            pos: 0,
            tokens: tok,
            locals: HashMap::new(),
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
        //println!("parse {}", ret.val.clone());
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

    pub fn consume_expected(&mut self, s: &str) {
        if self.cur().matches(s) {
            self.next();
        } else {
            panic!("Expected {}, but found {}", s, self.cur().val);
        }
    }

    pub fn new_lvar(&mut self, name: String, ty: Type) {
        self.locals.insert(name, ty);
    }

    //
    // ---------------- Generate AST ----------------
    //

    fn read_program(&mut self) -> Vec<AST> {
        let mut ret = Vec::new();
        while !self.cur().is_eof() {
           ret.push(self.read_top_level());
        }
        ret
    }

    fn read_top_level(&mut self) -> AST {
        self.read_func_def()
    }

    fn read_func_def(&mut self) -> AST {
        self.locals = HashMap::new();
        let func_ty = self.read_declspec();
        let (func_ty, func_name) = self.read_declarator(func_ty);
        self.consume_expected("{");
        let body = self.read_compound_stmt();
        
        return AST::FuncDef(Box::new(func_ty), func_name, Box::new(body));
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
            let mut cond = AST::Nil;
            if !self.consume(";") {
                cond = self.read_expr();
                self.consume_expected(";");
            }
            let mut step = AST::Nil;
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
            let ast;
            if self.cur().matches("int") {
                ast = self.read_declaration();
            } else {
                ast = self.read_stmt();
            }
            v.push(ast);
        }
        AST::Block(v)
    }

    fn read_declaration(&mut self) -> AST {
        let mut assigns = Vec::new();
        let declspec = self.read_declspec();
        let (ty, name) = self.read_declarator(declspec.clone());
        self.new_lvar(name.clone(), ty.clone());

        if self.consume("=") {
            let lhs = AST::Variable(name);
            let rhs = self.read_expr();
            let assign = AST::BinaryOp(
                Box::new(lhs), Box::new(rhs), BinaryOps::Assign);
            assigns.push(assign);
        }

        while self.consume(",") {
            let (ty, name) = self.read_declarator(declspec.clone());
            self.new_lvar(name.clone(), ty.clone());
            
            if self.consume("=") {
                let lhs = AST::Variable(name);
                let rhs = self.read_expr();
                let assign = AST::BinaryOp(
                    Box::new(lhs), Box::new(rhs), BinaryOps::Assign);
                assigns.push(assign);
            }
        }
        self.consume_expected(";");
        AST::Block(assigns)
    }

    fn read_declspec(&mut self) -> Type {
        match self.next().val.as_str() {
            "int" => return Type::Int,
            _ => panic!("Unknown type"),
        }
    }

    fn read_declarator(&mut self, mut ty: Type) -> (Type, String) {
        while self.consume("*") {
            ty = Type::Ptr(Box::new(ty));
        }
        let name = self.read_ident();
        ty = self.read_type_suffix(ty);
        (ty, name)
    }

    fn read_type_suffix(&mut self, mut ty: Type) -> Type {
        if self.consume("[") {    
            let arr_sz = self.read_num();
            self.consume_expected("]");
            ty = self.read_type_suffix(ty);
            ty = Type::Array(Box::new(ty), arr_sz as i32);
        } else if self.consume("(") {
            let (types, names) = self.read_func_params();
            // ret type, param types
            return Type::Func(Box::new(ty), types, names);
        }
        ty
    }

    fn read_func_params(&mut self) -> (Vec<Type>, Vec<String>) {
        let mut types = Vec::new();
        let mut names = Vec::new();

        if !self.consume(")") {
            let (ty, name) = self.read_param();
            types.push(ty);
            names.push(name);
            while self.consume(",") {
                let (ty, name) = self.read_param();
                types.push(ty);
                names.push(name);
            }  
            self.consume_expected(")");
        }
        (types, names)
    }

    fn read_param(&mut self) -> (Type, String) {
        let ty = self.read_declspec();
        let (ty, name) = self.read_declarator(ty);
        (ty, name)
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
            ret = AST::BinaryOp(Box::new(ret), Box::new(rhs), BinaryOps::Assign); 
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
            return AST::UnaryOp(Box::new(self.read_unary()), UnaryOps::Plus);
        } else if self.consume("-") {
            return AST::UnaryOp(Box::new(self.read_unary()), UnaryOps::Minus);
        } else if self.consume("&") {
            return AST::UnaryOp(Box::new(self.read_unary()), UnaryOps::Addr);
        } else if self.consume("*") {
            return AST::UnaryOp(Box::new(self.read_unary()), UnaryOps::Deref);
        }
        self.read_postfix()
    }

    fn read_postfix(&mut self) -> AST {
        let mut ret = self.read_primary();
        // x[y] is short for *(x+y)
        if self.consume("[") {
            let rhs = self.read_expr();
            self.consume_expected("]");
            ret = AST::UnaryOp(
                Box::new(AST::BinaryOp(Box::new(ret), Box::new(rhs), BinaryOps::Add)),
                UnaryOps::Deref,
            );
        }
        ret
    }

    fn read_primary(&mut self) -> AST {
        if self.consume("(") {
            let ast = self.read_expr();
            self.consume_expected(")");
            return ast;
        } else if self.consume("sizeof") {
            let ast = self.read_unary();
            return AST::UnaryOp(Box::new(ast), UnaryOps::Sizeof);
        } else if self.cur().is_ident() {
            if self.peek().matches("(") {
                return self.read_func_call();
            }
            return AST::Variable(self.read_ident());
        }else{
            return self.read_ast_num();
        }
    }

    fn read_func_call(&mut self) -> AST {
        let name = self.read_ident();
        let mut args = Vec::new();
        self.consume_expected("(");
        if !self.consume(")") {
            args.push(self.read_assign());
            while self.consume(",") {
                args.push(self.read_assign());
            }
            self.consume_expected(")");
        }
        AST::FuncCall(name, args)
    }

    fn read_ast_num(&mut self) -> AST {
        match self.next() {
            Token{ kind: TokenKind::IntNum, val: n, ..}  => AST::Int(n.parse::<i32>().unwrap()),
            Token{ kind: TokenKind::FloatNum, val: n, ..}  => AST::Float(n.parse::<f64>().unwrap()),
            _ => panic!("Numerical literal is expected"),
        } 
    }

    fn read_num(&mut self) -> f64 {
        match self.next() {
            Token{ kind: TokenKind::IntNum, val: n, ..}  => n.parse::<f64>().unwrap(),
            Token{ kind: TokenKind::FloatNum, val: n, ..}  => n.parse::<f64>().unwrap(),
            _ => panic!("Numerical literal is expected"),
        } 
    }

    fn read_ident(&mut self) -> String {
        self.next().val
    }
}