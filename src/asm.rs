use crate::ir::{SSAProgram, SSAFunction, CFG, Statement, Expression, SSAVar};
use crate::x64::{X64, X64Function, X64Program, Register, RegisterBuilder};

pub fn build(cfg: SSAProgram) -> X64Program {
    cfg.into_iter().map(|SSAFunction {name, body, ..}| X64Function{name, body: build_body(body)}).collect()
}

fn build_body(body: CFG) -> Vec<X64> {
    let mut reg_builder = RegisterBuilder::new();
    let mut x64s = Vec::new();
    for block in body {
        for stmt in block.statements {
            x64s.extend(build_stmt(stmt, &mut reg_builder));
        }
    }
    x64s
}

fn build_stmt(stmt: Statement, reg_builder: &mut RegisterBuilder) -> Vec<X64> {
    match stmt {
        Statement::Nop => Vec::new(),
        Statement::Phi(_, _) => Vec::new(),
        Statement::Declaration(expr) => Vec::new(),
        Statement::Compound(stmts) => stmts.into_iter().flat_map(|s|build_stmt(s, reg_builder)).collect(),
        Statement::Expression(expr) => build_expr(expr, reg_builder).0,
        Statement::If {
            condition, ..
        } => {

        }
        Statement::While {
            condition,..
        } => {

        }
        Statement::Return(expr) => {
            match expr {
                Some(expr) => {}
                None => {}
            }
        }
    }
}

fn build_expr(expr: Expression, reg_builder: &mut RegisterBuilder) -> (Vec<X64>, Register) {
    match expr {
        Expression::Identifier(expr) => {
            match expr {
                var @ SSAVar => (Vec::new(), reg_builder.from_var(var)),
                _ => panic!()
            }
        }
        Expression::Number(num) => {
            let reg = reg_builder.create_temp();
            (vec![X64::MovNum(reg, num)], reg)
        }
        Expression::Call {
            function: Box<Expression>,
            arguments: Box<Expression>,
        },
        Expression::Arguments(Vec<Expression>),
        Expression::Prefix {
            operator: &'static str,
            expression: Box<Expression>,
        },
        Expression::Infix {
            left: Box<Expression>,
            operator: &'static str,
            right: Box<Expression>,
        },
    }
}
