use crate::ir::{Expression, SSAFunction, SSAProgram, SSAVar, Statement, CFG};
use crate::x64::{Register, VRegisterAllocator, X64Function, X64Program, X64};
use std::collections::HashMap;

enum Tag {
    IfNoAlt(String),
    IfBody(String),
    IfAlt(String),
    WhileBody(String),
}

pub struct X64Builder {
    allocator: VRegisterAllocator,
    tags: HashMap<usize, Vec<Tag>>,
    successors: Vec<usize>,
}

impl X64Builder {
    pub fn new() -> Self {
        X64Builder {
            allocator: VRegisterAllocator::new(),
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
                    param_cnt: parameters.len(),
                    body: self.build_body(parameters, body),
                },
            )
            .collect()
    }

    fn build_body(&mut self, parameters: Vec<SSAVar>, body: CFG) -> Vec<X64> {
        self.allocator.clear();
        self.tags.clear();
        let mut asms = Vec::new();
        for var in parameters {
            self.allocator.from_var(var);
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
                Tag::IfNoAlt(tag) => asms.push(X64::Tag(format!("{}End", tag))),
                Tag::IfBody(tag) => asms.push(X64::Jmp(format!("{}End", tag))),
                Tag::IfAlt(tag) => {
                    asms.insert(0, X64::Tag(format!("{}Start", tag)));
                    asms.push(X64::Tag(format!("{}End", tag)));
                }
                Tag::WhileBody(tag) => {
                    asms.push(X64::Jmp(format!("{}Start", tag)));
                    asms.push(X64::Tag(format!("{}End", tag)));
                }
            }
        }
        asms
    }

    fn build_stmt(&mut self, stmt: Statement) -> Vec<X64> {
        match stmt {
            Statement::Nop => Vec::new(),
            Statement::Phi(_, _) => unreachable!(),
            Statement::Declaration(var) => {
                self.allocator.from_var(var);
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
                // Return immediately if the body and the alternative are empty.
                if self.successors.len() == 1 {
                    return asms;
                }
                if alternative.is_none() {
                    let body = self.successors[1] - 1;
                    let if_no_alt = Tag::IfNoAlt(format!("{}", reg));
                    self.tags.entry(body).or_default().push(if_no_alt);
                    asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(format!("{}End", reg))]);
                } else {
                    let body = self.successors[0];
                    let if_body = Tag::IfBody(format!("{}", reg));
                    self.tags.entry(body).or_default().push(if_body);
                    let alt = self.successors[1];
                    let if_alt = Tag::IfAlt(format!("{}", reg));
                    self.tags.entry(alt).or_default().push(if_alt);
                    asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(format!("{}Start", reg))]);
                }
                asms
            }
            Statement::While { condition, .. } => {
                let (mut asms, reg) = self.build_expr(condition);
                let body = self.successors[0];
                let while_body = Tag::WhileBody(format!("{}", reg));
                self.tags.entry(body).or_default().push(while_body);
                asms.insert(0, X64::Tag(format!("{}Start", reg)));
                asms.extend(vec![X64::CmpNum(reg, 0), X64::Je(format!("{}End", reg))]);
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
            Expression::Identifier(var) => (Vec::new(), self.allocator.from_var(var)),
            Expression::Number(num) => {
                let reg = self.allocator.create_temp();
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
                    let ret_reg = self.allocator.create_temp();
                    asms.push(X64::Call(name, regs, ret_reg));
                    (asms, ret_reg)
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
                    let (mut asms, reg) = self.build_expr(*expression);
                    asms.push(X64::Neg(reg));
                    (asms, reg)
                }
                "!" => {
                    let (mut asms, reg) = self.build_expr(*expression);
                    let r = self.allocator.create_temp();
                    asms.extend(vec![
                        X64::MovNum(r, 1),
                        X64::CmpNum(reg, 0),
                        X64::Je(format!("{}", r)),
                        X64::MovNum(r, 0),
                        X64::Tag(format!("{}", r)),
                    ]);
                    (asms, r)
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
                    let reg = self.allocator.create_temp();
                    let asms = match operator {
                        "*" => vec![X64::MovReg(reg, left_reg), X64::Imul(reg, right_reg)],
                        "/" => vec![X64::MovReg(reg, left_reg), X64::Idiv(reg, right_reg)],
                        "+" => vec![X64::MovReg(reg, left_reg), X64::Add(reg, right_reg)],
                        "-" => vec![X64::MovReg(reg, left_reg), X64::Sub(reg, right_reg)],
                        "&&" => vec![X64::MovReg(reg, left_reg), X64::And(reg, right_reg)],
                        "||" => vec![X64::MovReg(reg, left_reg), X64::Or(reg, right_reg)],
                        op => {
                            let asm = match op {
                                "<" => X64::Jl(format!("{}", reg)),
                                ">" => X64::Jg(format!("{}", reg)),
                                "<=" => X64::Jle(format!("{}", reg)),
                                ">=" => X64::Jge(format!("{}", reg)),
                                "==" => X64::Je(format!("{}", reg)),
                                "!=" => X64::Jne(format!("{}", reg)),
                                _ => unreachable!(),
                            };
                            vec![
                                X64::MovNum(reg, 1),
                                X64::CmpReg(left_reg, right_reg),
                                asm,
                                X64::MovNum(reg, 0),
                                X64::Tag(format!("{}", reg)),
                            ]
                        }
                    };
                    (asms, reg)
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 0,
            body: vec![X64::MovNum(Register::Virtual(1), 1)],
        }];
        assert_eq!(asm, expected);
    }

    #[test]
    fn call() {
        let ast = parser::parse(
            "
            int f(int a) {
                return a;
            }

            int main(int a) {
                return f(a) + 1;
            }
        ",
        );
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![
            X64Function {
                name: String::from("f"),
                param_cnt: 1,
                body: vec![X64::Ret(Some(Register::Virtual(0)))],
            },
            X64Function {
                name: String::from("main"),
                param_cnt: 1,
                body: vec![
                    X64::Call(
                        String::from("f"),
                        vec![Register::Virtual(0)],
                        Register::Virtual(1),
                    ),
                    X64::MovNum(Register::Virtual(2), 1),
                    X64::MovReg(Register::Virtual(3), Register::Virtual(1)),
                    X64::Add(Register::Virtual(3), Register::Virtual(2)),
                    X64::Ret(Some(Register::Virtual(3))),
                ],
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 1,
            body: vec![
                X64::Neg(Register::Virtual(1)),
                X64::MovNum(Register::Virtual(2), 0),
                X64::MovNum(Register::Virtual(3), 1),
                X64::CmpNum(Register::Virtual(2), 0),
                X64::Je(String::from("VR3")),
                X64::MovNum(Register::Virtual(3), 0),
                X64::Tag(String::from("VR3")),
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 2,
            body: vec![
                X64::MovNum(Register::Virtual(2), 0),
                X64::MovNum(Register::Virtual(3), 1),
                X64::MovReg(Register::Virtual(4), Register::Virtual(2)),
                X64::Imul(Register::Virtual(4), Register::Virtual(3)),
                X64::MovNum(Register::Virtual(5), 2),
                X64::MovReg(Register::Virtual(6), Register::Virtual(4)),
                X64::Idiv(Register::Virtual(6), Register::Virtual(5)),
                X64::MovNum(Register::Virtual(7), 3),
                X64::MovReg(Register::Virtual(8), Register::Virtual(6)),
                X64::Add(Register::Virtual(8), Register::Virtual(7)),
                X64::MovNum(Register::Virtual(9), 4),
                X64::MovReg(Register::Virtual(10), Register::Virtual(8)),
                X64::Sub(Register::Virtual(10), Register::Virtual(9)),
                X64::MovNum(Register::Virtual(11), 5),
                X64::MovReg(Register::Virtual(12), Register::Virtual(10)),
                X64::And(Register::Virtual(12), Register::Virtual(11)),
                X64::MovNum(Register::Virtual(13), 6),
                X64::MovReg(Register::Virtual(14), Register::Virtual(12)),
                X64::Or(Register::Virtual(14), Register::Virtual(13)),
                X64::MovReg(Register::Virtual(0), Register::Virtual(14)),
                X64::MovNum(Register::Virtual(15), 1),
                X64::CmpReg(Register::Virtual(0), Register::Virtual(0)),
                X64::Jl(String::from("VR15")),
                X64::MovNum(Register::Virtual(15), 0),
                X64::Tag(String::from("VR15")),
                X64::MovNum(Register::Virtual(16), 1),
                X64::CmpReg(Register::Virtual(15), Register::Virtual(0)),
                X64::Jg(String::from("VR16")),
                X64::MovNum(Register::Virtual(16), 0),
                X64::Tag(String::from("VR16")),
                X64::MovNum(Register::Virtual(17), 1),
                X64::CmpReg(Register::Virtual(16), Register::Virtual(0)),
                X64::Jle(String::from("VR17")),
                X64::MovNum(Register::Virtual(17), 0),
                X64::Tag(String::from("VR17")),
                X64::MovNum(Register::Virtual(18), 1),
                X64::CmpReg(Register::Virtual(17), Register::Virtual(0)),
                X64::Jge(String::from("VR18")),
                X64::MovNum(Register::Virtual(18), 0),
                X64::Tag(String::from("VR18")),
                X64::MovNum(Register::Virtual(19), 1),
                X64::CmpReg(Register::Virtual(18), Register::Virtual(0)),
                X64::Je(String::from("VR19")),
                X64::MovNum(Register::Virtual(19), 0),
                X64::Tag(String::from("VR19")),
                X64::MovNum(Register::Virtual(20), 1),
                X64::CmpReg(Register::Virtual(19), Register::Virtual(0)),
                X64::Jne(String::from("VR20")),
                X64::MovNum(Register::Virtual(20), 0),
                X64::Tag(String::from("VR20")),
                X64::MovReg(Register::Virtual(1), Register::Virtual(20)),
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 0,
            body: vec![
                X64::MovNum(Register::Virtual(0), 0),
                X64::CmpNum(Register::Virtual(0), 0),
                X64::Je(String::from("VR0Start")),
                X64::MovNum(Register::Virtual(1), 1),
                X64::Jmp(String::from("VR0End")),
                X64::Tag(String::from("VR0Start")),
                X64::MovNum(Register::Virtual(2), 2),
                X64::Tag(String::from("VR0End")),
                X64::MovNum(Register::Virtual(3), 3),
                X64::CmpNum(Register::Virtual(3), 0),
                X64::Je(String::from("VR3End")),
                X64::MovNum(Register::Virtual(4), 4),
                X64::Tag(String::from("VR3End")),
                X64::MovNum(Register::Virtual(5), 5),
                X64::MovNum(Register::Virtual(6), 6),
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 0,
            body: vec![
                X64::Tag(String::from("VR0Start")),
                X64::MovNum(Register::Virtual(0), 0),
                X64::CmpNum(Register::Virtual(0), 0),
                X64::Je(String::from("VR0End")),
                X64::MovNum(Register::Virtual(1), 1),
                X64::Jmp(String::from("VR0Start")),
                X64::Tag(String::from("VR0End")),
                X64::Tag(String::from("VR2Start")),
                X64::MovNum(Register::Virtual(2), 2),
                X64::CmpNum(Register::Virtual(2), 0),
                X64::Je(String::from("VR2End")),
                X64::Jmp(String::from("VR2Start")),
                X64::Tag(String::from("VR2End")),
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
        let (ssa, prog_leaves) = ssa::construct(ast);
        let cfg = ssa::destruct(ssa, prog_leaves);
        let asm = X64Builder::new().build(cfg);
        let expected = vec![X64Function {
            name: String::from("main"),
            param_cnt: 0,
            body: vec![
                X64::MovNum(Register::Virtual(0), 0),
                X64::CmpNum(Register::Virtual(0), 0),
                X64::Je(String::from("VR0End")),
                X64::MovNum(Register::Virtual(1), 1),
                X64::Ret(Some(Register::Virtual(1))),
                X64::Tag(String::from("VR0End")),
                X64::Ret(None),
            ],
        }];
        assert_eq!(asm, expected);
    }
}
