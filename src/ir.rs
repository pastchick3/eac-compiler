use std::collections::HashSet;

// IR used in the parser.
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
    Nop, // For CFG use only.
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

// IR used in the data-flow analysis.
#[derive(Debug, PartialEq, Default)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub predecessors: HashSet<usize>,
    pub successors: HashSet<usize>,
}

#[derive(Debug, PartialEq)]
pub struct CFG {
    blocks: Vec<Block>,
    current: usize,
    while_cond: usize,
    if_cond: usize,
    if_alt: bool,
    if_enter_body: usize,
    if_exit_body: usize,
    if_enter_alt: usize,
    if_exit_alt: usize,
}

impl CFG {
    pub fn new() -> Self {
        CFG {
            blocks: vec![Block::default()],
            current: 0,
            while_cond: 0,
            if_cond: 0,
            if_alt: false,
            if_enter_body: 0,
            if_exit_body: 0,
            if_enter_alt: 0,
            if_exit_alt: 0,
        }
    }

    pub fn push(&mut self, stmt: Statement) {
        self.blocks[self.current].statements.push(stmt);
    }

    pub fn enter_new_block(&mut self) -> bool {
        if self.blocks[self.current].statements.is_empty() {
            false
        } else {
            self.blocks[self.current]
                .successors
                .insert(self.current + 1);
            let mut block = Block::default();
            block.predecessors.insert(self.current);
            self.blocks.push(block);
            self.current += 1;
            true
        }
    }

    fn connect(&mut self, pred: usize, succ: usize) {
        if pred != succ {
            self.blocks[pred].successors.insert(succ);
            self.blocks[succ].predecessors.insert(pred);
        }
    }

    fn disconnect(&mut self, pred: usize, succ: usize) {
        if pred != succ {
            self.blocks[pred].successors.remove(&succ);
            self.blocks[succ].predecessors.remove(&pred);
        }
    }

    pub fn enter_if(&mut self, condition: Expression) {
        self.enter_new_block();
        let stmt = Statement::If {
            condition,
            body: Box::new(Statement::Nop),
            alternative: None,
        };
        self.push(stmt);
        self.if_cond = self.current;
        self.if_alt = false;
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
        self.if_alt = true;
        self.if_enter_alt = self.current;
    }

    pub fn exit_if_alt(&mut self) {
        self.enter_new_block();
        self.if_exit_alt = self.current - 1;
    }

    pub fn exit_if(&mut self) {
        self.connect(self.if_exit_body, self.current);
        if self.if_alt {
            self.connect(self.if_cond, self.if_enter_alt);
        } else {
            self.connect(self.if_cond, self.current);
        }
        self.if_alt = false;
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

    pub fn exit_while(&mut self) {
        self.blocks[self.current].successors.insert(self.while_cond);
        self.blocks[self.while_cond]
            .predecessors
            .insert(self.current);
        self.enter_new_block();
    }
}

#[derive(Debug, PartialEq)]
pub struct SSAFunction {
    pub void: bool,
    pub name: String,
    pub parameters: Vec<String>,
    pub body: CFG,
}

pub type SSAProgram = Vec<SSAFunction>;
