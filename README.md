# ironcc
a toy compiler written in Rust

# Syntax
```
topLevel = stmt*

stmt = "return" expr ";"
        | "if" "(" expr ")" stmt ("else" stmt)?
        | expr-stmt
        | "{" compound-stmt

compound-stmt = stmt* "}"

expr-stmt = expr? ";"

expr = assign

assign = equality ("=" assign)?

equality = relational ("=="|"!=" relational)*

relational = add (("<"|">"|"<="|">=") add)*

add = mul (("+"|"-") mul)*

mul = unary (("*"|"/") unary)*

unary = ("+"|"-")? primary

primary = "(" expr ")" | <ident> | <num>

```

<xxxx> means token.
