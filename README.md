# ironcc
A toy C compiler written in Rust.

ironcc is aiming to suppport C99, and C11.

# Status
ironcc supports the following functions:

- function definition
- local variable declaration
- return statement
- assignment
- types (int and pointer)
- control syntax (if, else, for)
- numerical literal
- binary operations (+, -, *, /)
- comparison operations (==, !=, <, >, <=, >=)
- unary operations (+, -)

# Syntax
```
program = top-level*

top-level = func-def

func-def = declspec declarator "{" compound-stmt

stmt = "return" expr ";"
        | "if" "(" expr ")" stmt ("else" stmt)?
        | "for" "(" expr-stmt expr? ";" expr? ")" stmt
        | "while" "(" expr ")" stmt
        | "{" compound-stmt
        | expr-stmt

compound-stmt = (declaration | stmt)* "}"

declaration = declspec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"

declspec = "int"

declarator = "*"* <ident> type-suffix

type-suffix = "(" func-params
        |"[" <num> "]"
        | ε

func-params = (param ("," param)*)? ")"

param = declspec declarator

type-suffix = "(" func-params no
        | "[" <num> "]" type-suffix
        | ε

expr-stmt = expr? ";"

expr = assign

assign = equality ("=" assign)?

equality = relational ("=="|"!=" relational)*

relational = add (("<"|">"|"<="|">=") add)*

add = mul (("+"|"-") mul)*

mul = unary (("*"|"/") unary)*

unary = ("+" | "-" | "*" | "&") unary
        | postfix

postfix = primary ("[" expr "]")*

primary = "(" expr ")"
        | "sizeof" unary
        | <ident> func-args?
        | <num>

func-call = <ident> "(" (assign ("," assign)*)? ")"

<XXX> means token.
```

# Todo
- support arithmetic operations of pointers
- support multi-[] operator (like a[3][4], b[0][1][3])