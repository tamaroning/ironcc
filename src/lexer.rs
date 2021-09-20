use std::iter;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::str::FromStr;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub enum TokenKind {
    Ident,
    Num(f64),
    Symbol,
    Keyword,
    NewLine, // not used
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub val: String,
    pub line: u32,
}

impl Token {
    pub fn matches(&self, s: &str) -> bool {
        self.val.as_str() == s
    }
}

pub struct Lexer<'a> {
    cur_line: u32,
    filepath: String,
    peek: iter::Peekable<str::Chars<'a>>,
    peek_pos: usize, 
}

pub fn run(filepath: String) -> Vec<Token> {
    let mut file = File::open(filepath.clone()).expect("File not found");
    let mut content = String::new();
    file.read_to_string(&mut content).expect("Couldn't open the file");
    let mut lexer = Lexer::new(filepath.clone(), content.as_str());
    
    let mut tokens = Vec::new();
    loop {
        let token = lexer.read_token();
        match token {
            Some(Token { kind: TokenKind::Eof, .. }) => {
                tokens.push(token.unwrap());
                break;
            }
            Some(_) => {
                println!("{:?}", token.as_ref().unwrap());
                tokens.push(token.unwrap());
            },
            _ => panic!("Lexer error"),
        }
    }
    tokens
}

impl<'a> Lexer<'a> {
    pub fn new(path: String, input: &'a str) -> Lexer<'a> {
        Lexer {
            cur_line: 0,
            filepath: path,
            peek: input.chars().peekable(),
            peek_pos: 0,
        }
    }

    pub fn peek_next(&mut self) -> Option<char> {
        self.peek_pos += 1;
        self.peek.next()
    }

    pub fn peek_skip(&mut self, n: usize) {
        for _ in 0..n {
            self.peek_next();
        }
    }

    pub fn starts_with(&self, s: &str) -> bool {
        //println!("cmp");
        //println!("{:?}",s.to_string());
        //println!("{:?}", String::from_iter(self.peek.clone().take(s.len())));
        String::from_iter(self.peek.clone().take(s.len())) == s.to_string()
    }

    pub fn get_filepath(&self) -> String {
        self.filepath.clone()
    }

    pub fn read_symbol(&mut self) -> Token {
        // multicharacter symbols
        let ops = vec!["==", "!=", "<=", ">="];
        for op in ops {
            if self.starts_with(op) {
                self.peek_skip(2);
                return Token { kind: TokenKind::Symbol, val: op.to_string(), line: self.cur_line };
            }
        }
        // single character symbols
        let sym = self.peek_next().unwrap().to_string();
        Token { kind: TokenKind::Symbol, val: sym, line: self.cur_line }
    }

    pub fn read_newline(&mut self) -> Token {
        self.peek_next();
        self.cur_line += 1;
        Token { kind: TokenKind::NewLine, val: "".to_string(), line: self.cur_line }
    }

    // ident, keyword
    pub fn read_string_token(&mut self) -> Token {
        let mut string = String::new();
        loop {
            match self.peek.peek() {
                Some(&c) => match c {
                    'a'..='z' | 'A'..='Z' | '0'..='9' => string.push(c),
                    _ => break,
                },
                _ => break,
            }
            self.peek_next();
        }
        let tk = match string.as_str() {
            "def" | "extern" => TokenKind::Keyword,
            _ => TokenKind::Ident,
        };
        Token { kind: tk, val: string, line: self.cur_line }
    }

    pub fn read_num(&mut self) -> Token {
        let mut s = String::new();
        loop {
            match self.peek.peek() {
                Some(&c) => match c {
                    '0'..='9' => s.push(c),
                    _ => break,
                },
                _ => break,
            }
            self.peek_next();
        }
        // TODO: conversion error
        let f = f64::from_str(&s).unwrap();
        Token { kind: TokenKind::Num(f), val: s, line: self.cur_line }
    }

    pub fn read_token(&mut self) -> Option<Token> {
        match self.peek.peek() {
            Some(&c) => match c {
                'a'..='z' | 'A'..='Z' => Some(self.read_string_token()),
                '+' | '-' | '*' | '/' | '(' | ')' | '=' | '<' | '>' | '!' => Some(self.read_symbol()),
                '0'..='9' => Some(self.read_num()),
                ' ' | '\t' => {
                    self.peek_next();
                    self.read_token()
                },
                // comment
                '#' => {
                    self.peek_next();
                    loop {
                        match self.peek.peek() {
                            Some('\n') => {
                                self.peek_next();
                                return self.read_token();
                            },
                            None => break,
                            _ => (),
                        }
                        self.peek_next();
                    }
                    self.read_token()
                },
                '\n' => {
                    self.read_newline();
                    self.read_token()
                },
                // TODO: unexpected character error
                _ => None,
            },
            // TODO: None always means Eof?
            _ => Some(Token{ kind: TokenKind::Eof, val: "".to_string(), line: self.cur_line }),
        }
    }
}