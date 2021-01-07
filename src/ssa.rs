use crate::ir::{CFGBuilder, Function, Program, SSAFunction, SSAProgram, Statement};

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

fn build_ssa(func: SSAFunction) -> SSAFunction {
    func
}

fn _build_cfg(stmt: Statement, cfg: &mut CFGBuilder) -> bool {
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
                    statements: vec![Statement::Declaration(Expression::Identifier(
                        String::from("a"),
                    ))],
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
}
