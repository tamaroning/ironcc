#[derive(Debug, Clone)]
pub enum Type {
    Int,
    Ptr(Box<Type>),
}