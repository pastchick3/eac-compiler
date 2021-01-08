use crate::ir::{
    Block, CFGBuilder, Expression, Function, Program, SSAFunction, SSAProgram, SSAVar, Statement,
    CFG,
};
use std::collections::{HashMap, HashSet};

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
        mut parameters,
        mut body,
    } = func;
    insert_phi(&mut body);
    let reaching_def = find_reaching_def(&mut parameters, &mut body);
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
        for var in vars {
            let phi = Statement::Phi(SSAVar::new(&var), HashSet::new());
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

type ReachingMap = HashMap<String, HashSet<usize>>;

fn find_reaching_def(parameters: &mut Vec<SSAVar>, body: &mut CFG) -> Vec<ReachingMap> {
    let mut def_map = HashMap::new();
    let mut reaches = vec![ReachingMap::new(); body.len()];
    let mut de_defs = vec![HashMap::new(); body.len()];
    let mut def_kills = vec![HashSet::new(); body.len()];
    for SSAVar { name, subscript } in parameters {
        let current_subscript = def_map.entry(name.to_string()).or_default();
        *subscript = Some(*current_subscript);
        reaches[0].insert(
            name.to_string(),
            vec![*current_subscript].into_iter().collect(),
        );
        *current_subscript += 1;
    }
    for i in 0..body.len() {
        let de_def = &mut de_defs[i];
        let def_kill = &mut def_kills[i];
        for stmt in &mut body[i].statements {
            if let Statement::Phi(SSAVar { name, subscript }, ..)
            | Statement::Declaration(Expression::Identifier(SSAVar { name, subscript })) = stmt
            {
                let current_subscript = def_map.entry(name.to_string()).or_default();
                *subscript = Some(*current_subscript);
                match de_def.get(name) {
                    Some(_) => de_def.remove(name),
                    None => de_def.insert(name.to_string(), *current_subscript),
                };
                def_kill.insert(name.to_string());
                *current_subscript += 1;
            }
        }
    }
    let mut old_reaches = Vec::new();
    while old_reaches != reaches {
        old_reaches = reaches.clone();
        for i in 0..body.len() {
            let preds = find_predecessors(body, i);
            for pred in preds {
                let mut fall_through = reaches[pred].clone();
                for killed in &def_kills[pred] {
                    fall_through.remove(killed);
                }
                for (name, subs) in fall_through {
                    reaches[i].entry(name).or_default().extend(subs);
                }
                for (name, sub) in de_defs[pred].clone() {
                    reaches[i].entry(name).or_default().insert(sub);
                }
            }
        }
    }
    reaches
}

fn find_predecessors(body: &[Block], index: usize) -> HashSet<usize> {
    let mut predecessors = HashSet::new();
    let mut stack = vec![index];
    while let Some(pred) = stack.pop() {
        predecessors.extend(body[pred].predecessors.clone());
        stack.extend(body[pred].predecessors.clone());
    }
    predecessors
}

fn rename_ssa(reaching_maps: Vec<ReachingMap>, body: &mut CFG) {
    for (block, reaching_map) in body.iter_mut().zip(reaching_maps) {
        let mut var_map = HashMap::new();
        for stmt in &mut block.statements {
            rename_stmt_vars(stmt, &reaching_map, &mut var_map);
        }
    }
}

fn rename_stmt_vars(
    stmt: &mut Statement,
    reaching_map: &ReachingMap,
    var_map: &mut HashMap<String, usize>,
) {
    match stmt {
        Statement::Nop => {}
        Statement::Phi(var, values) => {
            let subs = reaching_map
                .get(&var.name)
                .unwrap_or_else(|| panic!("Undefined variable `{}`.", var.name));
            for sub in subs {
                let value = SSAVar {
                    name: var.name.to_string(),
                    subscript: Some(*sub),
                };
                values.insert(value);
            }
            var_map.insert(var.name.to_string(), var.subscript.unwrap());
        }
        Statement::Declaration(expr) => {
            if let Expression::Identifier(SSAVar { name, subscript }) = expr {
                var_map.insert(name.to_string(), subscript.unwrap());
            }
        }
        Statement::Compound(stmts) => {
            for stmt in stmts {
                rename_stmt_vars(stmt, reaching_map, var_map);
            }
        }
        Statement::Expression(expr) => {
            rename_expr_vars(expr, reaching_map, var_map);
        }
        Statement::If {
            condition,
            body,
            alternative,
        } => {
            rename_expr_vars(condition, reaching_map, var_map);
            rename_stmt_vars(body, reaching_map, var_map);
            if let Some(alt) = alternative {
                rename_stmt_vars(alt, reaching_map, var_map);
            }
        }
        Statement::While { condition, body } => {
            rename_expr_vars(condition, reaching_map, var_map);
            rename_stmt_vars(body, reaching_map, var_map);
        }
        Statement::Return(expr) => {
            if let Some(expr) = expr {
                rename_expr_vars(expr, reaching_map, var_map);
            }
        }
    }
}

fn rename_expr_vars(
    expr: &mut Expression,
    reaching_map: &ReachingMap,
    var_map: &mut HashMap<String, usize>,
) {
    match expr {
        Expression::Identifier(SSAVar { name, subscript }) => {
            let sub = match var_map.get(name) {
                Some(sub) => *sub,
                None => {
                    let reach = reaching_map
                        .get(name)
                        .unwrap_or_else(|| panic!("Undefined variable `{}`.", name));
                    *reach.iter().next().unwrap()
                }
            };
            *subscript = Some(sub);
        }
        Expression::Number(_) => {}
        Expression::Call { arguments, .. } => {
            rename_expr_vars(arguments, reaching_map, var_map);
        }
        Expression::Arguments(exprs) => {
            for expr in exprs {
                rename_expr_vars(expr, reaching_map, var_map);
            }
        }
        Expression::Prefix { expression, .. } => {
            rename_expr_vars(expression, reaching_map, var_map);
        }
        Expression::Infix { left, right, .. } => {
            rename_expr_vars(left, reaching_map, var_map);
            rename_expr_vars(right, reaching_map, var_map);
        }
    }
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
                    4;
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
    fn reaching_def() {
        let mut ast = parser::parse(
            "
            void main(int a) {
                int a;
                int b;
                if (0) {
                    int a;
                }
            }
        ",
        );
        let mut ssa = build_cfg(ast.remove(0));
        find_reaching_def(&mut ssa.parameters, &mut ssa.body);
        let expected = SSAFunction {
            void: true,
            name: String::from("main"),
            parameters: vec![SSAVar {
                name: "a".to_string(),
                subscript: Some(0),
            }],
            body: vec![
                Block {
                    statements: vec![
                        Statement::Declaration(Expression::Identifier(SSAVar {
                            name: "a".to_string(),
                            subscript: Some(1),
                        })),
                        Statement::Declaration(Expression::Identifier(SSAVar {
                            name: "b".to_string(),
                            subscript: Some(0),
                        })),
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
                    statements: vec![Statement::Declaration(Expression::Identifier(SSAVar {
                        name: "a".to_string(),
                        subscript: Some(2),
                    }))],
                    predecessors: vec![1].into_iter().collect(),
                    successors: vec![3].into_iter().collect(),
                },
                Block {
                    statements: vec![],
                    predecessors: vec![1, 2].into_iter().collect(),
                    successors: vec![].into_iter().collect(),
                },
            ],
        };
        assert_eq!(ssa, expected);
    }

    #[test]
    fn ssa() {
        let mut ast = parser::parse(
            "
            void main(int a) {
                int b;
                if (0) {
                    int b;
                }
                main(a);
                b;
            }
        ",
        );
        let cfg = build_cfg(ast.remove(0));
        let ssa = build_ssa(cfg);
        let expected = SSAFunction {
            void: true,
            name: String::from("main"),
            parameters: vec![SSAVar {
                name: "a".to_string(),
                subscript: Some(0),
            }],
            body: vec![
                Block {
                    statements: vec![Statement::Declaration(Expression::Identifier(SSAVar {
                        name: "b".to_string(),
                        subscript: Some(0),
                    }))],
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
                    statements: vec![Statement::Declaration(Expression::Identifier(SSAVar {
                        name: "b".to_string(),
                        subscript: Some(1),
                    }))],
                    predecessors: vec![1].into_iter().collect(),
                    successors: vec![3].into_iter().collect(),
                },
                Block {
                    statements: vec![
                        Statement::Phi(
                            SSAVar {
                                name: "b".to_string(),
                                subscript: Some(2),
                            },
                            vec![
                                SSAVar {
                                    name: "b".to_string(),
                                    subscript: Some(0),
                                },
                                SSAVar {
                                    name: "b".to_string(),
                                    subscript: Some(1),
                                },
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Statement::Phi(
                            SSAVar {
                                name: "a".to_string(),
                                subscript: Some(1),
                            },
                            vec![
                                SSAVar {
                                    name: "a".to_string(),
                                    subscript: Some(0),
                                },
                                SSAVar {
                                    name: "a".to_string(),
                                    subscript: Some(0),
                                },
                            ]
                            .into_iter()
                            .collect(),
                        ),
                        Statement::Expression(Expression::Call {
                            function: Box::new(Expression::Identifier(SSAVar {
                                name: "main".to_string(),
                                subscript: None,
                            })),
                            arguments: Box::new(Expression::Arguments(vec![
                                Expression::Identifier(SSAVar {
                                    name: "a".to_string(),
                                    subscript: Some(1),
                                }),
                            ])),
                        }),
                        Statement::Expression(Expression::Identifier(SSAVar {
                            name: "b".to_string(),
                            subscript: Some(2),
                        })),
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
