struct Program {}

#[derive(Debug, Clone)]
pub enum Declaration {
    Function {
        name: String,
        arguments: Vec<(String, Type)>,
        return_type: Type,
        body: Vec<Statement>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Fun(Vec<Type>, Box<Type>),
    U32,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Option<Type>, Expression),
    Expr(Expression),
    Return(Expression),
}

#[derive(Debug, Clone)]
pub enum Expression {
    Call(String, Vec<Expression>),
    Variable(String),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    NumLiteral(u32),
}
