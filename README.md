# ironcc
a toy compiler written in Rust

# Syntax
```
topLevel = stmt*

stmt = "return" expr ";"
        | "if" "(" expr ")" stmt ("else" stmt)?
        | "for" "(" expr-stmt expr? ";" expr? ")" stmt
        | "while" "(" expr ")" stmt
        | "{" compound-stmt
        | expr-stmt

compound-stmt = (declaration | stmt)* "}"

declaration = declspec (declarator ("=" expr)? ("," declarator ("=" expr)?)*)? ";"

declspec = "int"

declarator = "*"* <ident>

expr-stmt = expr? ";"

expr = assign

assign = equality ("=" assign)?

equality = relational ("=="|"!=" relational)*

relational = add (("<"|">"|"<="|">=") add)*

add = mul (("+"|"-") mul)*

mul = unary (("*"|"/") unary)*

unary = ("+" | "-" | "*" | "&") unary
        | primary

primary = "(" expr ")" | <ident> func-args? | <num>
->fun-argsがあればfunc-callとみなす

func-call = <ident> "(" (assign ("," assign)*)? ")"

```

```<xxxx>``` means token.
