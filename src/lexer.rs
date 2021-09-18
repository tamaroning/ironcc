use std::iter;
use std::str;

pub enum TokenKind {
    Ident,
    Num,
    Symbol,
    Keyword,
    Eof,
}

pub struct Token {
    kind: TokenKind,
    val: String,
    line: u32,
}

pub struct Lexer<'a> {
    cur_line: u32,
    filepath: String,
    peek: iter::Peekable<str::Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(path: String, input: &'a str) -> Lexer<'a> {
        Lexer {
            cur_line: 0,
            filepath: path,
            peek: input.chars().peekable(),
        }
    }

    pub fn get_filepath(&self) -> String {
        self.filepath.clone()
    }

    pub fn read_token(&mut self) -> Token {
        println!("{}", self.peek.peek().unwrap());
        self.peek.next();

        Token {
            kind: TokenKind::Eof,
            val: "a".to_string(),
            line: 0,
        }

    }
}