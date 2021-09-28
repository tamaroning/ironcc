use std::iter;
use std::str;
use std::fs::File;
use std::io::prelude::*;
use std::iter::FromIterator;

#[derive(Debug, Clone)]
pub enum TokenKind {
    IntNum,
    FloatNum,
    Symbol,
    Keyword,
    Ident,
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

    pub fn is_eof(&self) -> bool {
        return match self.kind {
            TokenKind::Eof => true,
            _ => false,
        };
    }

    pub fn is_ident(&self) -> bool {
        return match self.kind {
            TokenKind::Ident => true,
            _ => false,
        };
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

    pub fn get_filepath(&self) -> String {
        self.filepath.clone()
    }

    pub fn peek_next(&mut self) -> Option<char> {
        //println!("lex '{}'", self.peek.peek().unwrap());
        self.peek_pos += 1;
        self.peek.next()
    }

    // advance by n characters
    pub fn advance_by(&mut self, n: usize) {
        for _ in 0..n {
            self.peek_next();
        }
    }

    // read forward expected string
    pub fn skip_token(&mut self, s: &str) {
        if self.read_token().unwrap().val != s {
            panic!("Expected {}", s);
        }
    }

    pub fn starts_with(&self, s: &str) -> bool {
        String::from_iter(self.peek.clone().take(s.len())) == s.to_string()
    }

    pub fn read_symbol(&mut self) -> Token {
        // multicharacter symbols
        let ops = vec!["==", "!=", "<=", ">="];
        for op in ops {
            if self.starts_with(op) {
                self.advance_by(2);
                return Token { kind: TokenKind::Symbol, val: op.to_string(), line: self.cur_line };
            }
        }
        // single character symbols
        let sym = self.peek_next().unwrap().to_string();
        Token { kind: TokenKind::Symbol, val: sym, line: self.cur_line }
    }

    pub fn read_newline(&mut self) -> Token {
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
            "if"|"else"|"for"|"while" => TokenKind::Keyword,
            _ => TokenKind::Ident,
        };
        Token { kind: tk, val: string, line: self.cur_line }
    }

    pub fn read_num(&mut self) -> Token {
        let mut s = String::new();
        let mut is_float = false;
        loop {
            match self.peek.peek() {
                Some(&c) => match c {
                    '0'..='9' => s.push(c),
                    '.' => {
                        s.push(c);
                        is_float = true;
                    }
                    _ => break,
                },
                _ => break,
            }
            self.peek_next();
        }
        if is_float {
            Token { kind: TokenKind::FloatNum, val: s, line: self.cur_line }
        } else {
            Token { kind: TokenKind::IntNum, val: s, line: self.cur_line }
        }
    }

    fn read_directive(&mut self) {
        let dir_string = self.read_token().unwrap().val;
        if dir_string == "include" {
            self.read_include_directive();
        } else {
            panic!("Unknown direvtive");
        }
    }

    fn read_include_directive(&mut self) {
        self.skip_token("<");
        let mut filename = String::new();
        while !(*self.peek.peek().unwrap() == '>') {
            filename.push(self.peek_next().unwrap());
        }
        self.skip_token(">");
        println!("include: {}", filename);
        // TODO: implement #include here
    }

    pub fn read_token(&mut self) -> Option<Token> {
        match self.peek.peek() {
            Some(&c) => match c {
                'a'..='z' | 'A'..='Z' => Some(self.read_string_token()),
                '+'|'-'|'*'|'/'|'('|')'|'='|'<'|'>'|'!'|'&'|','|';'|'{'|'}' => Some(self.read_symbol()),
                '0'..='9' => Some(self.read_num()),
                ' ' | '\t' | '\r' => {
                    self.peek_next();
                    self.read_token()
                },
                '\n' => {
                    self.peek_next();
                    self.read_newline();
                    self.read_token()
                },
                '#' => {
                    self.peek_next();
                    self.read_directive();
                    self.read_token()
                }
                // TODO: unexpected character
                _ => panic!("Unknown character: '{}' code: {}", c, c as u8),
            },
            // TODO: None always means Eof?
            _ => Some(Token{ kind: TokenKind::Eof, val: "".to_string(), line: self.cur_line }),
        }
    }
}