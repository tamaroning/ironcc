#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
    Array(Box<Type>, i32),                   // type, size
    Func(Box<Type>, Vec<Type>, Vec<String>), // ret type, param types, param names
}
