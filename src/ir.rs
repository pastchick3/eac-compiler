use std::collections::HashSet;

// IR used in the parser.
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct SSAVar {
    pub name: String,
    pub subscript: Option<usize>,
}

impl SSAVar {
    pub fn new(name: &str) -> Self {
        SSAVar {
            name: name.to_string(),
            subscript: None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Identifier(SSAVar),
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

#[derive(Debug, PartialEq, Clone)]
pub enum Statement {
    Nop,                          // For CFG use only.
    Phi(SSAVar, HashSet<SSAVar>), // For SSA use only.
    Declaration(SSAVar),
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
    pub parameters: Vec<SSAVar>,
    pub body: Statement,
}

pub type Program = Vec<Function>;

// IR used in the data-flow analysis.
#[derive(Debug, PartialEq, Default)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub predecessors: HashSet<usize>,
    pub successors: HashSet<usize>,
}

pub type CFG = Vec<Block>;

#[derive(Debug, PartialEq)]
pub struct SSAFunction {
    pub void: bool,
    pub name: String,
    pub parameters: Vec<SSAVar>,
    pub body: CFG,
}

pub type SSAProgram = Vec<SSAFunction>;

// A supporting builder used in the data-flow analysis.
#[derive(Debug, PartialEq)]
pub struct CFGBuilder {
    blocks: Vec<Block>,
    current: usize,
    while_cond: usize,
    if_cond: usize,
    if_enter_body: usize,
    if_exit_body: usize,
    if_alt: bool,
    if_enter_alt: usize,
    if_exit_alt: usize,
}

impl CFGBuilder {
    pub fn new() -> Self {
        CFGBuilder {
            blocks: vec![Block::default()],
            current: 0,
            while_cond: 0,
            if_cond: 0,
            if_enter_body: 0,
            if_exit_body: 0,
            if_alt: false,
            if_enter_alt: 0,
            if_exit_alt: 0,
        }
    }

    pub fn get_cfg(self) -> CFG {
        self.blocks
    }

    pub fn push(&mut self, stmt: Statement) {
        self.blocks[self.current].statements.push(stmt);
    }

    pub fn enter_new_block(&mut self) {
        if !self.blocks[self.current].statements.is_empty() {
            self.blocks[self.current]
                .successors
                .insert(self.current + 1);
            let mut block = Block::default();
            block.predecessors.insert(self.current);
            self.blocks.push(block);
            self.current += 1;
        }
    }

    fn connect(&mut self, pred: usize, succ: usize) {
        self.blocks[pred].successors.insert(succ);
        self.blocks[succ].predecessors.insert(pred);
    }

    fn disconnect(&mut self, pred: usize, succ: usize) {
        self.blocks[pred].successors.remove(&succ);
        self.blocks[succ].predecessors.remove(&pred);
    }

    pub fn enter_if(&mut self, condition: Expression, alt: bool) {
        self.enter_new_block();
        let alternative = match alt {
            true => Some(Box::new(Statement::Nop)),
            false => None,
        };
        let stmt = Statement::If {
            condition,
            body: Box::new(Statement::Nop),
            alternative,
        };
        self.push(stmt);
        self.if_cond = self.current;
        self.if_alt = alt;
        self.enter_new_block();
    }

    pub fn enter_if_body(&mut self) {
        self.if_enter_body = self.current;
    }

    pub fn exit_if_body(&mut self) {
        self.enter_new_block();
        self.if_exit_body = self.current - 1;
        self.disconnect(self.if_exit_body, self.current);
    }

    pub fn enter_if_alt(&mut self) {
        self.if_enter_alt = self.current;
    }

    pub fn exit_if_alt(&mut self) {
        self.enter_new_block();
        self.if_exit_alt = self.current - 1;
    }

    pub fn exit_if(&mut self) {
        self.enter_new_block();
        self.connect(self.if_exit_body, self.current);
        if self.if_alt {
            self.connect(self.if_cond, self.if_enter_alt);
        } else {
            self.connect(self.if_cond, self.current);
        }
        self.enter_new_block();
    }

    pub fn enter_while(&mut self, condition: Expression) {
        self.enter_new_block();
        let stmt = Statement::While {
            condition,
            body: Box::new(Statement::Nop),
        };
        self.push(stmt);
        self.while_cond = self.current;
        self.enter_new_block();
    }

    pub fn exit_while(&mut self, body_return: bool) {
        self.enter_new_block();
        let while_exit_body = self.current - 1;
        if !body_return {
            self.connect(while_exit_body, self.while_cond);
        }
        self.disconnect(while_exit_body, self.current);
        self.connect(self.while_cond, self.current);
    }
}
