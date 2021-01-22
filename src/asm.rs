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
            .map(
                |SSAFunction {
                     name,
                     parameters,
                     body,
                     ..
                 }| X64Function {
                    name,
                    body: self.build_body(parameters, body),
                },
            )
            .collect()
    }

    fn build_body(&mut self, parameters: Vec<SSAVar>, body: CFG) -> Vec<X64> {
        self.reg_builder.clear();
        self.tags.clear();
        let mut asms = Vec::new();
        for var in parameters {
            self.reg_builder.from_var(var);
        }
        for (index, block) in body.into_iter().enumerate() {
            self.successors = block.successors.into_iter().collect();
            self.successors.sort_unstable();
            asms.extend(self.build_block(index, block.statements));
        }
        asms
    }

    fn build_block(&mut self, index: usize, stmts: Vec<Statement>) -> Vec<X64> {
        let mut asms = Vec::new();
        for stmt in stmts {
            asms.extend(self.build_stmt(stmt));
        }
        for tag in self.tags.entry(index).or_default() {
            match tag {
                Tag::IfNoAlt(tag) => {
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.push(X64::Tag(tag_end));
                }
                Tag::IfBody(tag) => {
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.push(X64::Jump(tag_end));
                }
                Tag::IfAlt(tag) => {
                    let mut tag_start = tag.clone();
                    tag_start.push_str("Start");
                    asms.insert(0, X64::Tag(tag_start));
                    let mut tag_end = tag.clone();
                    tag_end.push_str("End");
                    asms.push(X64::Tag(tag_end));
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
            Statement::Declaration(var) => {
                self.reg_builder.from_var(var);
                Vec::new()
            }
            Statement::Compound(stmts) => {
                stmts.into_iter().flat_map(|s| self.build_stmt(s)).collect()
            }
            Statement::Expression(expr) => self.build_expr(expr).0,
            Statement::If {
                condition,
                alternative,
                ..
            } => {
                let (mut asms, reg) = self.build_expr(condition);
                if self.successors.len() == 1 {
                    return asms;
                }
                let mut tag = format!("R{}", reg);
                if alternative.is_none() {
                    let body = self.successors[1] - 1;
                    let if_no_alt = Tag::IfNoAlt(tag.clone());
                    self.tags.entry(body).or_default().push(if_no_alt);
                    tag.push_str("End");
                    asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(tag)]);
                } else {
                    let body = self.successors[0];
                    let if_body = Tag::IfBody(tag.clone());
                    self.tags.entry(body).or_default().push(if_body);
                    let alt = self.successors[1];
                    let if_alt = Tag::IfAlt(tag.clone());
                    self.tags.entry(alt).or_default().push(if_alt);
                    tag.push_str("Start");
                    asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(tag)]);
                }
                asms
            }
            Statement::While { condition, .. } => {
                let (mut asms, reg) = self.build_expr(condition);
                let mut tag = format!("R{}", reg);
                let body = self.successors[0];
                let while_body = Tag::WhileBody(tag.clone());
                self.tags.entry(body).or_default().push(while_body);
                let mut tag_start = tag.clone();
                tag_start.push_str("Start");
                asms.insert(0, X64::Tag(tag_start));
                tag.push_str("End");
                asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(tag)]);
                asms
            }
            Statement::Return(Some(expr)) => {
                let (mut asms, reg) = self.build_expr(expr);
                asms.push(X64::Ret(Some(reg)));
                asms
            }
            Statement::Return(None) => vec![X64::Ret(None)],
        }
    }

    fn build_expr(&mut self, expr: Expression) -> (Vec<X64>, Register) {
        match expr {
            Expression::Identifier(var) => (Vec::new(), self.reg_builder.from_var(var)),
            Expression::Number(num) => {
                let reg = self.reg_builder.create_temp();
                (vec![X64::MovNum(reg, num)], reg)
            }
            Expression::Call {
                function,
                arguments,
            } => {
                if let (Expression::Identifier(SSAVar { name, .. }), Expression::Arguments(exprs)) =
                    (*function, *arguments)
                {
                    let mut asms = Vec::new();
                    let mut regs = Vec::new();
                    for expr in exprs {
                        let (a, r) = self.build_expr(expr);
                        asms.extend(a);
                        regs.push(r);
                    }
                    asms.push(X64::Call(name, regs));
                    (asms, 0)
                } else {
                    unreachable!();
                }
            }
            Expression::Arguments(_) => unreachable!(),
            Expression::Prefix {
                operator,
                expression,
            } => match operator {
                "+" => self.build_expr(*expression),
                "-" => {
                    let (mut asm, reg) = self.build_expr(*expression);
                    asm.push(X64::Neg(reg));
                    (asm, reg)
                }
                "!" => {
                    let (mut asm, reg) = self.build_expr(*expression);
                    let r = self.reg_builder.create_temp();
                    let tag = format!("R{}", r);
                    asm.extend(vec![
                        X64::MovNum(r, 1),
                        X64::CmpNum(reg, 0),
                        X64::Je(tag.clone()),
                        X64::MovNum(r, 0),
                        X64::Tag(tag),
                    ]);
                    (asm, r)
                }
                _ => unreachable!(),
            },
            Expression::Infix {
                left,
                operator,
                right,
            } => {
                let (mut left_asms, left_reg) = self.build_expr(*left);
                let (right_asms, right_reg) = self.build_expr(*right);
                let (asms, reg) = if operator == "=" {
                    (vec![X64::MovReg(left_reg, right_reg)], left_reg)
                } else {
                    let reg = self.reg_builder.create_temp();
                    match operator {
                        "*" => (
                            vec![X64::MovReg(reg, left_reg), X64::Imul(reg, right_reg)],
                            reg,
                        ),
                        "/" => (
                            vec![X64::MovReg(reg, left_reg), X64::Idiv(reg, right_reg)],
                            reg,
                        ),
                        "+" => (
                            vec![X64::MovReg(reg, left_reg), X64::Add(reg, right_reg)],
                            reg,
                        ),
                        "-" => (
                            vec![X64::MovReg(reg, left_reg), X64::Sub(reg, right_reg)],
                            reg,
                        ),
                        "&&" => (
                            vec![X64::MovReg(reg, left_reg), X64::And(reg, right_reg)],
                            reg,
                        ),
                        "||" => (
                            vec![X64::MovReg(reg, left_reg), X64::Or(reg, right_reg)],
                            reg,
                        ),
                        op => {
                            let tag = format!("R{}", reg);
                            let asm = match op {
                                "<" => X64::Jl(tag.clone()),
                                ">" => X64::Jg(tag.clone()),
                                "<=" => X64::Jle(tag.clone()),
                                ">=" => X64::Jge(tag.clone()),
                                "==" => X64::Je(tag.clone()),
                                "!=" => X64::Jne(tag.clone()),
                                _ => unreachable!(),
                            };
                            (
                                vec![
                                    X64::MovNum(reg, 1),
                                    X64::CmpReg(left_reg, right_reg),
                                    asm,
                                    X64::MovNum(reg, 0),
                                    X64::Tag(tag),
                                ],
                                reg,
                            )
                        }
                    }
                };
                left_asms.extend(right_asms);
                left_asms.extend(asms);
                (left_asms, reg)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser;
    use crate::ssa;

    #[test]
    fn simple_var_num() {
        let ast = parser::parse(
            "
            void main() {
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
            body: vec![X64::MovNum(1, 1)],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn call() {
        let ast = parser::parse(
            "
            void f(int a) {}

            void main(int a) {
                f(a);
            }
        ",
        );
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![
            X64Function {
                name: String::from("f"),
                body: vec![],
            },
            X64Function {
                name: String::from("main"),
                body: vec![X64::Call(String::from("f"), vec![0])],
            },
        ];
        assert_eq!(asm, expected);
    }

    #[test]
    fn expr_prefix() {
        let ast = parser::parse(
            "
            void main(int a) {
                int b;
                +a;
                -b;
                !0;
            }
        ",
        );
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::Neg(1),
                X64::MovNum(2, 0),
                X64::MovNum(3, 1),
                X64::CmpNum(2, 0),
                X64::Je(String::from("R3")),
                X64::MovNum(3, 0),
                X64::Tag(String::from("R3")),
            ],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn expr_infix() {
        let ast = parser::parse(
            "
            void main(int a, int b) {
                a = 0 * 1 / 2 + 3 - 4 && 5 || 6;
                b = a < a > a <= a >= a == a != a;
            }
        ",
        );
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::MovNum(2, 0),
                X64::MovNum(3, 1),
                X64::MovReg(4, 2),
                X64::Imul(4, 3),
                X64::MovNum(5, 2),
                X64::MovReg(6, 4),
                X64::Idiv(6, 5),
                X64::MovNum(7, 3),
                X64::MovReg(8, 6),
                X64::Add(8, 7),
                X64::MovNum(9, 4),
                X64::MovReg(10, 8),
                X64::Sub(10, 9),
                X64::MovNum(11, 5),
                X64::MovReg(12, 10),
                X64::And(12, 11),
                X64::MovNum(13, 6),
                X64::MovReg(14, 12),
                X64::Or(14, 13),
                X64::MovReg(0, 14),
                X64::MovNum(15, 1),
                X64::CmpReg(0, 0),
                X64::Jl(String::from("R15")),
                X64::MovNum(15, 0),
                X64::Tag(String::from("R15")),
                X64::MovNum(16, 1),
                X64::CmpReg(15, 0),
                X64::Jg(String::from("R16")),
                X64::MovNum(16, 0),
                X64::Tag(String::from("R16")),
                X64::MovNum(17, 1),
                X64::CmpReg(16, 0),
                X64::Jle(String::from("R17")),
                X64::MovNum(17, 0),
                X64::Tag(String::from("R17")),
                X64::MovNum(18, 1),
                X64::CmpReg(17, 0),
                X64::Jge(String::from("R18")),
                X64::MovNum(18, 0),
                X64::Tag(String::from("R18")),
                X64::MovNum(19, 1),
                X64::CmpReg(18, 0),
                X64::Je(String::from("R19")),
                X64::MovNum(19, 0),
                X64::Tag(String::from("R19")),
                X64::MovNum(20, 1),
                X64::CmpReg(19, 0),
                X64::Jne(String::from("R20")),
                X64::MovNum(20, 0),
                X64::Tag(String::from("R20")),
                X64::MovReg(1, 20),
            ],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn stmt_if() {
        let ast = parser::parse(
            "
            void main() {
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
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::MovNum(0, 0),
                X64::CmpNum(0, 0),
                X64::Je(String::from("R0Start")),
                X64::MovNum(1, 1),
                X64::Jump(String::from("R0End")),
                X64::Tag(String::from("R0Start")),
                X64::MovNum(2, 2),
                X64::Tag(String::from("R0End")),
                X64::MovNum(3, 3),
                X64::CmpNum(3, 0),
                X64::Je(String::from("R3End")),
                X64::MovNum(4, 4),
                X64::Tag(String::from("R3End")),
                X64::MovNum(5, 5),
                X64::MovNum(6, 6),
            ],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn stmt_while() {
        let ast = parser::parse(
            "
            void main() {
                while (0) {
                    1;
                }
                while (2) {}
            }
        ",
        );
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::Tag(String::from("R0Start")),
                X64::MovNum(0, 0),
                X64::CmpNum(0, 0),
                X64::Je(String::from("R0End")),
                X64::MovNum(1, 1),
                X64::Jump(String::from("R0Start")),
                X64::Tag(String::from("R0End")),
                X64::Tag(String::from("R2Start")),
                X64::MovNum(2, 2),
                X64::CmpNum(2, 0),
                X64::Je(String::from("R2End")),
                X64::Jump(String::from("R2Start")),
                X64::Tag(String::from("R2End")),
            ],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn stmt_return() {
        let ast = parser::parse(
            "
            void main() {
                if (0) {
                    return 1;
                }
                return;
            }
        ",
        );
        let ssa = ssa::construct(ast);
        let cfg = ssa::destruct(ssa);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            body: vec![
                X64::MovNum(0, 0),
                X64::CmpNum(0, 0),
                X64::Je(String::from("R0End")),
                X64::MovNum(1, 1),
                X64::Ret(Some(1)),
                X64::Tag(String::from("R0End")),
                X64::Ret(None),
            ],
        }];
        assert_eq!(asm, expected);
    }
}
