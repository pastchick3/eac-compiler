use crate::ir::{Expression, SSAFunction, SSAProgram, SSAVar, Statement, CFG};
use crate::x64::{Register, RegisterBuilder, X64Function, X64Program, X64};
use std::collections::HashMap;

enum Tag {
    IfNoAlt(String),
    IfBody(String),
    IfAlt(String),
    WhileBody(String),
}

pub struct X64Builder {
    reg_builder: RegisterBuilder,
    tags: HashMap<usize, Vec<Tag>>,
    successors: Vec<usize>,
}

impl X64Builder {
    pub fn new() -> Self {
        X64Builder {
            reg_builder: RegisterBuilder::new(),
            tags: HashMap::new(),
            successors: Vec::new(),
        }
    }
    pub fn build(&mut self, cfg: SSAProgram) -> X64Program {
        cfg.into_iter()
            .map(|SSAFunction { name, body, .. }| X64Function {
                name,
                body: self.build_body(body),
            })
            .collect()
    }

    fn build_body(&mut self, body: CFG) -> Vec<X64> {
        self.reg_builder.clear();
        self.tags.clear();
        let mut asms = Vec::new();
        for (i, block) in body.into_iter().enumerate() {
            self.successors = block.successors.into_iter().collect();
            self.successors.sort_unstable();
            asms.extend(self.build_block(i, block.statements));
        }
        asms
    }

    fn build_block(&mut self, i: usize, stmts: Vec<Statement>) -> Vec<X64> {
        let mut asms = Vec::new();
        for stmt in stmts {
            asms.extend(self.build_stmt(stmt));
        }
        for tag in &self.tags[&i] {
            match tag {
                Tag::IfNoAlt(tag) => {
                    let mut tag_start = tag.clone();
                    tag_start.push_str("Start");
                    asms.insert(0, X64::Tag(tag_start));
                }
                Tag::IfBody(tag) => {
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.insert(0, X64::Jump(tag_end));
                }
                Tag::IfAlt(tag) => {
                    let mut tag_start = tag.clone();
                    tag_start.push_str("Start");
                    asms.insert(0, X64::Tag(tag_start));
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.push(X64::Jump(tag_end));
                }
                Tag::WhileBody(tag) => {
                    let mut tag_start = tag.clone();
                    tag_start.push_str("Start");
                    asms.push(X64::Jump(tag_start));
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.push(X64::Tag(tag_end));
                }
            }
        }
        asms
    }

    fn build_stmt(&mut self, stmt: Statement) -> Vec<X64> {
        match stmt {
            Statement::Nop => Vec::new(),
            Statement::Phi(_, _) => Vec::new(),
            Statement::Declaration(_) => Vec::new(),
            Statement::Compound(stmts) => {
                stmts.into_iter().flat_map(|s| self.build_stmt(s)).collect()
            }
            Statement::Expression(expr) => self.build_expr(expr).0,
            Statement::If {
                condition,
                alternative,
                ..
            } => {
                let (mut asms, regs) = self.build_expr(condition);
                let mut tag = format!("R{}", regs[0]);
                if alternative.is_none() {
                    let next = self.successors[1];
                    let tag = Tag::IfNoAlt(tag.clone());
                    self.tags.entry(next).or_default().push(tag);
                } else {
                    let body = self.successors[0];
                    let if_body = Tag::IfBody(tag.clone());
                    self.tags.entry(body).or_default().push(if_body);
                    let alt = self.successors[1];
                    let if_alt = Tag::IfAlt(tag.clone());
                    self.tags.entry(alt).or_default().push(if_alt);
                }
                tag.push_str("Start");
                asms.extend(vec![X64::CmpNum(regs[0], 0), X64::Je(tag)]);
                asms
            }
            Statement::While { condition, .. } => {
                let (mut asms, regs) = self.build_expr(condition);
                let mut tag = format!("R{}", regs[0]);
                let body = self.successors[0];
                let while_body = Tag::WhileBody(tag.clone());
                self.tags.entry(body).or_default().push(while_body);
                let mut tag_start = tag.clone();
                tag_start.push_str("Start");
                asms.insert(0, X64::Tag(tag_start));
                tag.push_str("End");
                asms.extend(vec![X64::CmpNum(regs[0], 0), X64::Je(tag)]);
                asms
            }
            Statement::Return(expr) => match expr {
                Some(expr) => {
                    let (mut asms, regs) = self.build_expr(expr);
                    asms.push(X64::Ret(Some(regs[0])));
                    asms
                }
                None => vec![X64::Ret(None)],
            },
        }
    }

    fn build_expr(&mut self, expr: Expression) -> (Vec<X64>, Vec<Register>) {
        match expr {
            Expression::Identifier(SSAVar { name, subscript }) => (
                Vec::new(),
                vec![self.reg_builder.from_var(SSAVar { name, subscript })],
            ),
            Expression::Number(num) => {
                let reg = self.reg_builder.create_temp();
                (vec![X64::MovNum(reg, num)], vec![reg])
            }
            Expression::Call {
                function,
                arguments,
            } => {
                if let Expression::Identifier(SSAVar { name, .. }) = *function {
                    let (mut asms, regs) = self.build_expr(*arguments);
                    asms.push(X64::Call(name, regs));
                    (asms, vec![])
                } else {
                    panic!();
                }
            }
            Expression::Arguments(exprs) => {
                let mut asms = Vec::new();
                let mut regs = Vec::new();
                for expr in exprs {
                    let (a, r) = self.build_expr(expr);
                    asms.extend(a);
                    regs.extend(r);
                }
                (asms, regs)
            }
            Expression::Prefix {
                operator,
                expression,
            } => match operator {
                "+" => self.build_expr(*expression),
                "-" => {
                    let (mut asm, regs) = self.build_expr(*expression);
                    asm.push(X64::Neg(regs[0]));
                    (asm, regs)
                }
                "!" => {
                    let (mut asm, regs) = self.build_expr(*expression);
                    let reg = self.reg_builder.create_temp();
                    let tag = format!("R{}", reg);
                    asm.extend(vec![
                        X64::MovNum(reg, 1),
                        X64::CmpNum(regs[0], 0),
                        X64::Je(tag.clone()),
                        X64::MovNum(reg, 0),
                        X64::Tag(tag),
                    ]);
                    (asm, vec![reg])
                }
                _ => panic!(),
            },
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let (mut left_asms, left_regs) = self.build_expr(*left);
                let (right_asms, right_regs) = self.build_expr(*right);
                let (asms, reg) = match operator {
                    "*" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![
                                X64::MovReg(reg, left_regs[0]),
                                X64::Imul(reg, right_regs[0]),
                            ],
                            reg,
                        )
                    }
                    "/" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![
                                X64::MovReg(reg, left_regs[0]),
                                X64::Idiv(reg, right_regs[0]),
                            ],
                            reg,
                        )
                    }
                    "+" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![X64::MovReg(reg, left_regs[0]), X64::Add(reg, right_regs[0])],
                            reg,
                        )
                    }
                    "-" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![X64::MovReg(reg, left_regs[0]), X64::Sub(reg, right_regs[0])],
                            reg,
                        )
                    }
                    "&&" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![X64::MovReg(reg, left_regs[0]), X64::And(reg, right_regs[0])],
                            reg,
                        )
                    }
                    "||" => {
                        let reg = self.reg_builder.create_temp();
                        (
                            vec![X64::MovReg(reg, left_regs[0]), X64::Or(reg, right_regs[0])],
                            reg,
                        )
                    }
                    "=" => (vec![X64::MovReg(left_regs[0], right_regs[0])], left_regs[0]),
                    op => {
                        let reg = self.reg_builder.create_temp();
                        let tag = format!("R{}", reg);
                        let asm = match op {
                            "<" => X64::Jl(tag.clone()),
                            ">" => X64::Jg(tag.clone()),
                            "<=" => X64::Jle(tag.clone()),
                            ">=" => X64::Jge(tag.clone()),
                            "==" => X64::Je(tag.clone()),
                            "!=" => X64::Jne(tag.clone()),
                            _ => panic!(),
                        };
                        (
                            vec![
                                X64::MovNum(reg, 1),
                                X64::CmpReg(left_regs[0], right_regs[0]),
                                asm,
                                X64::MovNum(reg, 0),
                                X64::Tag(tag),
                            ],
                            reg,
                        )
                    }
                };
                left_asms.extend(right_asms);
                left_asms.extend(asms);
                (left_asms, vec![reg])
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::ssa;

    // #[test]
    fn name() {
        let ast = parser::parse(
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
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![X64::MovNum(0, 1)],
        }];
        assert_eq!(asm, expected);
    }
}
