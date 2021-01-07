use crate::ir::{
    CFGBuilder, Expression, Function, Program, SSAFunction, SSAProgram, SSAVar, Statement, CFG,
};
use std::collections::HashMap;

pub fn build(ast: Program) -> SSAProgram {
    ast.into_iter().map(build_cfg).map(build_ssa).collect()
}

fn build_cfg(func: Function) -> SSAFunction {
    let Function {
        void,
        name,
        parameters,
        body,
    } = func;
    let mut cfg_builder = CFGBuilder::new();
    _build_cfg(body, &mut cfg_builder);
    SSAFunction {
        void,
        name,
        parameters,
        body: cfg_builder.get_cfg(),
    }
}

fn _build_cfg(stmt: Statement, cfg: &mut CFGBuilder) -> bool {
    match stmt {
        Statement::Nop => unreachable!(),
        Statement::Phi(_, _) => unreachable!(),
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
            let body_return = _build_cfg(*body, cfg);
            cfg.exit_while(body_return);
        }
        stmt @ Statement::Return(_) => {
            cfg.push(stmt);
            return true;
        }
    }
    false
}

fn build_ssa(func: SSAFunction) -> SSAFunction {
    let SSAFunction {
        void,
        name,
        parameters,
        mut body,
    } = func;
    insert_phi(&mut body);
    let reaching_def = find_reaching_def(&mut body);
    rename_ssa(reaching_def, &mut body);
    SSAFunction {
        void,
        name,
        parameters,
        body,
    }
}

fn insert_phi(body: &mut CFG) {
    for block in body {
        if block.predecessors.len() <= 1 {
            continue;
        }
        let mut vars = Vec::new();
        for stmt in &block.statements {
            find_stmt_vars(stmt, &mut vars);
        }
        let mut preds = Vec::new();
        preds.resize(block.predecessors.len(), String::new());
        for var in vars {
            let phi = Statement::Phi(var, preds.clone());
            block.statements.insert(0, phi);
        }
    }
}

fn find_stmt_vars(stmt: &Statement, vars: &mut Vec<String>) {
    match stmt {
        Statement::Nop => {}
        Statement::Phi(_, _) => unreachable!(),
        Statement::Declaration(expr) => {
            if let Expression::Identifier(var) = expr {
                vars.push(var.name.to_string());
            }
        }
        Statement::Compound(stmts) => {
            for stmt in stmts {
                find_stmt_vars(stmt, vars);
            }
        }
        Statement::Expression(expr) => {
            find_expr_vars(expr, vars);
        }
        Statement::If {
            condition,
            body,
            alternative,
        } => {
            find_expr_vars(condition, vars);
            find_stmt_vars(body, vars);
            if let Some(alt) = alternative {
                find_stmt_vars(alt, vars);
            }
        }
        Statement::While { condition, body } => {
            find_expr_vars(condition, vars);
            find_stmt_vars(body, vars);
        }
        Statement::Return(expr) => {
            if let Some(expr) = expr {
                find_expr_vars(expr, vars);
            }
        }
    }
}

fn find_expr_vars(expr: &Expression, vars: &mut Vec<String>) {
    match expr {
        Expression::Identifier(var) => {
            vars.push(var.name.to_string());
        }
        Expression::Number(_) => {}
        Expression::Call { arguments, .. } => {
            find_expr_vars(arguments, vars);
        }
        Expression::Arguments(exprs) => {
            for expr in exprs {
                find_expr_vars(expr, vars);
            }
        }
        Expression::Prefix { expression, .. } => {
            find_expr_vars(expression, vars);
        }
        Expression::Infix { left, right, .. } => {
            find_expr_vars(left, vars);
            find_expr_vars(right, vars);
        }
    }
}

type DefMap<'a> = HashMap<String, usize>;

fn find_reaching_def(body: &mut CFG) -> Vec<DefMap> {
    let mut def_map = DefMap::new();
    for block in body {
        for stmt in &mut block.statements {
            if let Statement::Declaration(Expression::Identifier(var)) = stmt {
                let current_subscript = def_map.entry(var.name.to_string()).or_default();
                // var.subscript = Some(*current_subscript);
            }
        }
    }
    vec![def_map]
}

fn rename_ssa(def_maps: Vec<DefMap>, body: &mut CFG) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{Block, Expression};
    use crate::parser;

    #[test]
    fn cfg_linear() {
        let mut ast = parser::parse(
            "
            int main() {
                int a;
                {}
                {
                    1;
                }
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let expected = SSAFunction {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: vec![
                Block {
                    statements: vec![Statement::Declaration(Expression::Identifier(SSAVar::new(
                        "a",
                    )))],
                    predecessors: vec![].into_iter().collect(),
                    successors: vec![1].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(1))],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![2].into_iter().collect(),
                },
                Block {
                    statements: vec![],
                    predecessors: vec![1].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn cfg_if() {
        let mut ast = parser::parse(
            "
            int main() {
                if (0) {
                    1;
                } else {
                    2;
                }
                if (3) {
                    4
                }
                if (5) {} else {}
                if (6) {}
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let expected = SSAFunction {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: vec![
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(0),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![].into_iter().collect(),
                    successors: vec![1, 2].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(1))],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![3].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(2))],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![3].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(3),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![1, 2].into_iter().collect(),
                    successors: vec![4, 5].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(4))],
                    predecessors: vec![3].into_iter().collect(),
                    successors: vec![5].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(5),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![3, 4].into_iter().collect(),
                    successors: vec![6].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(6),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![5].into_iter().collect(),
                    successors: vec![7].into_iter().collect(),
                },
                Block {
                    statements: vec![],
                    predecessors: vec![6].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn cfg_while() {
        let mut ast = parser::parse(
            "
            int main() {
                while (0) {
                    1;
                }
                while (2) {}
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let expected = SSAFunction {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: vec![
                Block {
                    statements: vec![Statement::While {
                        condition: Expression::Number(0),
                        body: Box::new(Statement::Nop),
                    }],
                    predecessors: vec![1].into_iter().collect(),
                    successors: vec![1, 2].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(1))],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![0].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::While {
                        condition: Expression::Number(2),
                        body: Box::new(Statement::Nop),
                    }],
                    predecessors: vec![0, 2].into_iter().collect(),
                    successors: vec![2, 3].into_iter().collect(),
                },
                Block {
                    statements: vec![],
                    predecessors: vec![2].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn cfg_return() {
        let mut ast = parser::parse(
            "
            int main() {
                if (0) {
                    return 1;
                }
                while (2) {
                    return 3;
                }
                {
                    return 4;
                }
                5;
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let expected = SSAFunction {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: vec![
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(0),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![].into_iter().collect(),
                    successors: vec![1, 2].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Return(Some(Expression::Number(1)))],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![2].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::While {
                        condition: Expression::Number(2),
                        body: Box::new(Statement::Nop),
                    }],
                    predecessors: vec![0, 1].into_iter().collect(),
                    successors: vec![3, 4].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Return(Some(Expression::Number(3)))],
                    predecessors: vec![2].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Return(Some(Expression::Number(4)))],
                    predecessors: vec![2].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(cfg, expected);
    }

    #[test]
    fn ssa() {
        let mut ast = parser::parse(
            "
            int main(int d) {
                int b;
                int c;
                if (0) {
                    1;
                }
                int a;
                -b + c;
                main(d);
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let ssa = build_ssa(cfg);
        let expected = SSAFunction {
            void: false,
            name: String::from("main"),
            parameters: vec![SSAVar::new("d")],
            body: vec![
                Block {
                    statements: vec![
                        Statement::Declaration(Expression::Identifier(SSAVar::new("b"))),
                        Statement::Declaration(Expression::Identifier(SSAVar::new("c"))),
                    ],
                    predecessors: vec![].into_iter().collect(),
                    successors: vec![1].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::If {
                        condition: Expression::Number(0),
                        body: Box::new(Statement::Nop),
                        alternative: None,
                    }],
                    predecessors: vec![0].into_iter().collect(),
                    successors: vec![2, 3].into_iter().collect(),
                },
                Block {
                    statements: vec![Statement::Expression(Expression::Number(1))],
                    predecessors: vec![1].into_iter().collect(),
                    successors: vec![3].into_iter().collect(),
                },
                Block {
                    statements: vec![
                        Statement::Phi(String::from("d"), vec![String::from(""), String::from("")]),
                        Statement::Phi(String::from("c"), vec![String::from(""), String::from("")]),
                        Statement::Phi(String::from("b"), vec![String::from(""), String::from("")]),
                        Statement::Phi(String::from("a"), vec![String::from(""), String::from("")]),
                        Statement::Declaration(Expression::Identifier(SSAVar::new("a"))),
                        Statement::Expression(Expression::Infix {
                            left: Box::new(Expression::Prefix {
                                operator: "-",
                                expression: Box::new(Expression::Identifier(SSAVar::new("b"))),
                            }),
                            operator: "+",
                            right: Box::new(Expression::Identifier(SSAVar::new("c"))),
                        }),
                        Statement::Expression(Expression::Call {
                            function: Box::new(Expression::Identifier(SSAVar::new("main"))),
                            arguments: Box::new(Expression::Arguments(vec![
                                Expression::Identifier(SSAVar::new("d")),
                            ])),
                        }),
                    ],
                    predecessors: vec![1, 2].into_iter().collect(),
                    successors: vec![4].into_iter().collect(),
                },
                Block {
                    statements: vec![],
                    predecessors: vec![3].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(ssa, expected);
    }
}
