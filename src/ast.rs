#[derive(Debug, PartialEq)]
pub enum Expression {
    Identifier(String),
    Number(i32),
    Call {
        function: Box<Expression>,
        arguments: Box<Expression>,
    },
    Arguments(Vec<Expression>),
    Prefix {
        operator: &'static str,
        expression: Box<Expression>,
    },
    Infix {
        left: Box<Expression>,
        operator: &'static str,
        right: Box<Expression>,
    },
}

#[derive(Debug, PartialEq)]
pub enum Statement {
    Declaration(Expression),
    Compound(Vec<Statement>),
    Expression(Expression),
    If {
        condition: Expression,
        body: Box<Statement>,
        alternative: Option<Box<Statement>>,
    },
    While {
        condition: Expression,
        body: Box<Statement>,
    },
    Return(Option<Expression>),
}

#[derive(Debug, PartialEq)]
pub struct Function {
    pub void: bool,
    pub name: String,
    pub parameters: Vec<String>,
    pub body: Statement,
}

pub type Program = Vec<Function>;
