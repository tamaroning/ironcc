use std::iter;
use std::str;

#[derive(Debug)]
pub enum TokenKind {
    Ident,
    Num,
    Symbol,
    Keyword,
    NewLine,
    Eof,
}

#[derive(Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub val: String,
    pub line: u32,
}

pub struct Lexer<'a> {
    cur_line: u32,
    filepath: String,
    peek: iter::Peekable<str::Chars<'a>>,
    peek_pos: u32, 
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

    pub fn get_filepath(&self) -> String {
        self.filepath.clone()
    }

    pub fn read_symbol(&mut self) -> Token {
        let c = self.peek_next();
        // one character symbol
        let mut sym = String::new();
        sym.push(c.unwrap());
        Token { kind: TokenKind::Symbol, val: sym, line: self.cur_line }
    }

    pub fn read_newline(&mut self) -> Token {
        self.peek_next();
        self.cur_line += 1;
        Token { kind: TokenKind::NewLine, val: "".to_string(), line: self.cur_line }
    }

    // 
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
        let mut ident = String::new();
        loop {
            match self.peek.peek() {
                Some(&c) => match c {
                    '0'..='9' => ident.push(c),
                    _ => break,
                },
                _ => break,
            }
            self.peek_next();
        }
        Token { kind: TokenKind::Num, val: ident, line: self.cur_line }
    }

    pub fn read_token(&mut self) -> Option<Token> {
        match self.peek.peek() {
            Some(&c) => match c {
                'a'..='z' | 'A'..='Z' => Some(self.read_string_token()),
                '+' | '-' => Some(self.read_symbol()),
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
                                return Some(self.read_newline());
                            },
                            None => break,
                            _ => (),
                        }
                        self.peek_next();
                    }
                    self.read_token()
                },
                '\n' => Some(self.read_newline()),
                _ => None,
            },
            // TODO: None always means Eof?
            _ => Some(Token{ kind: TokenKind::Eof, val: "".to_string(), line: self.cur_line }),
        }
    }
}