// Build a cfg -> Gather initial information -> Solve the equations to produce LiveOut(b) for each block b

use crate::ir::{Function, Program, SSAFunction, SSAProgram, Statement, CFG};

pub fn build_ssa(ast: Program) -> SSAProgram {
    ast.into_iter().map(build_cfg).collect()
}

fn build_cfg(func: Function) -> SSAFunction {
    let Function {
        void,
        name,
        parameters,
        body,
    } = func;
    let mut cfg = CFG::new();
    _build_cfg(body, &mut cfg);
    SSAFunction {
        void,
        name,
        parameters,
        body: cfg,
    }
}

fn _build_cfg(stmt: Statement, cfg: &mut CFG) -> bool {
    match stmt {
        Statement::Nop => unreachable!(),
        stmt @ Statement::Declaration(_) => cfg.push(stmt),
        Statement::Compound(stmts) => {
            cfg.enter_new_block();
            for stmt in stmts {
                if _build_cfg(stmt, cfg) {
                    return true;
                }
            }
            cfg.enter_new_block();
        }
        stmt @ Statement::Expression(_) => cfg.push(stmt),
        Statement::If {
            condition,
            body,
            alternative,
        } => {
            cfg.enter_if(condition);
            cfg.enter_if_body();
            let body_return = _build_cfg(*body, cfg);
            cfg.exit_if_body();
            let alt_return = match alternative {
                Some(alt) => {
                    cfg.enter_if_alt();
                    let alt_return = _build_cfg(*alt, cfg);
                    cfg.exit_if_alt();
                    alt_return
                }
                None => false,
            };
            cfg.exit_if();
            if body_return && alt_return {
                return true;
            }
        }
        Statement::While { condition, body } => {
            cfg.enter_while(condition);
            if _build_cfg(*body, cfg) {
                return true;
            }
            cfg.exit_while();
        }
        stmt @ Statement::Return(_) => {
            cfg.push(stmt);
            return true;
        }
    }
    false
}
