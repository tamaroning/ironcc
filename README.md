# ironcc
a toy compiler written in Rust

# Syntax
```
topLevel = expr-stmt*

expr-stmt = expr ";"

expr = equality

equality = relational ("=="|"!=" relational)*

relational = add (("<"|">"|"<="|">=") add)*

add = mul (("+"|"-") mul)*

mul = unary (("*"|"/") unary)*

unary = ("+"|"-")? primary

primary = "(" expr ")" | <num>

```

<xxxx> means token.
