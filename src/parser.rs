use crate::ir::{Expression, Function, Program, Statement};
use libc::{c_char, size_t};
use std::ffi::CString;

static mut EVENTS: Vec<(String, String)> = Vec::new();

pub fn parse(source: &str) -> Program {
    let source = CString::new(source).unwrap().into_raw();
    unsafe {
        EVENTS.clear();
        CString::from_raw(_parse(source, rs_get_str, rs_emit_event));
    }
    build_ast()
}

extern "C" fn rs_get_str(len: size_t) -> *mut c_char {
    let mut v = Vec::new();
    v.resize(len, 1);
    CString::new(v).unwrap().into_raw()
}

extern "C" fn rs_emit_event(tag: *mut c_char, text: *mut c_char) {
    unsafe {
        let tag = CString::from_raw(tag).into_string().unwrap();
        let text = CString::from_raw(text).into_string().unwrap();
        EVENTS.push((tag, text));
    }
}

#[link(name = "parser")]
extern "C" {
    fn _parse(
        path: *const c_char,
        rs_get_str: extern "C" fn(size_t) -> *mut c_char,
        rs_emit_event: extern "C" fn(*mut c_char, *mut c_char),
    ) -> *mut c_char;
}

fn build_ast() -> Program {
    let mut program = Program::new();
    let mut expr_stack = Vec::new();
    let mut stmt_stack = Vec::new();
    let mut compound_stmt_stack = Vec::new();
    unsafe {
        for (tag, text) in &EVENTS {
            match tag.as_str() {
                "ExitPrimaryExpression" => {
                    let expr = match text.parse::<i32>() {
                        Ok(num) => Expression::Number(num),
                        Err(_) => Expression::Identifier(text.to_string()),
                    };
                    expr_stack.push(expr);
                }
                "ExitPostfixExpression" => {
                    let args = match expr_stack.last() {
                        Some(Expression::Arguments(_)) => expr_stack.pop().unwrap(),
                        _ => Expression::Arguments(Vec::new()),
                    };
                    let func = expr_stack.pop().unwrap();
                    let call = Expression::Call {
                        function: Box::new(func),
                        arguments: Box::new(args),
                    };
                    expr_stack.push(call);
                }
                "ExitArgumentExpressionList" => {
                    let arg = expr_stack.pop().unwrap();
                    let args = match expr_stack.last_mut() {
                        Some(Expression::Arguments(args)) => {
                            args.push(arg);
                            expr_stack.pop().unwrap()
                        }
                        _ => Expression::Arguments(vec![arg]),
                    };
                    expr_stack.push(args);
                }
                "ExitUnaryExpression" => {
                    let expr = expr_stack.pop().unwrap();
                    let expr = Expression::Prefix {
                        operator: text,
                        expression: Box::new(expr),
                    };
                    expr_stack.push(expr);
                }
                "ExitMultiplicativeExpression"
                | "ExitAdditiveExpression"
                | "ExitRelationalExpression"
                | "ExitEqualityExpression"
                | "ExitLogicalAndExpression"
                | "ExitLogicalOrExpression" => {
                    let right = expr_stack.pop().unwrap();
                    let left = expr_stack.pop().unwrap();
                    let expr = Expression::Infix {
                        left: Box::new(left),
                        operator: text,
                        right: Box::new(right),
                    };
                    expr_stack.push(expr);
                }
                "ExitDeclaration" => {
                    let stmt = Statement::Declaration(Expression::Identifier(text.to_string()));
                    stmt_stack.push(stmt);
                }
                "EnterCompoundStatement" => {
                    compound_stmt_stack.push(stmt_stack.len());
                }
                "ExitCompoundStatement" => {
                    let compound_stmt_ptr = compound_stmt_stack.pop().unwrap();
                    let mut stmts = Vec::new();
                    while stmt_stack.len() != compound_stmt_ptr {
                        stmts.push(stmt_stack.pop().unwrap());
                    }
                    stmts.reverse();
                    let stmt = Statement::Compound(stmts);
                    stmt_stack.push(stmt);
                }
                "ExitExpressionStatement" => {
                    let expr = expr_stack.pop().unwrap();
                    let stmt = Statement::Expression(expr);
                    stmt_stack.push(stmt);
                }
                "ExitSelectionStatement" => {
                    let condition = expr_stack.pop().unwrap();
                    let (body, alternative) = match text.is_empty() {
                        true => {
                            let body = stmt_stack.pop().unwrap();
                            (body, None)
                        }
                        false => {
                            let alternative = stmt_stack.pop().unwrap();
                            let body = stmt_stack.pop().unwrap();
                            (body, Some(Box::new(alternative)))
                        }
                    };
                    let stmt = Statement::If {
                        condition,
                        body: Box::new(body),
                        alternative,
                    };
                    stmt_stack.push(stmt);
                }
                "ExitIterationStatement" => {
                    let condition = expr_stack.pop().unwrap();
                    let body = stmt_stack.pop().unwrap();
                    let stmt = Statement::While {
                        condition,
                        body: Box::new(body),
                    };
                    stmt_stack.push(stmt);
                }
                "ExitJumpStatement" => {
                    let expr = match text.is_empty() {
                        true => None,
                        false => Some(expr_stack.pop().unwrap()),
                    };
                    let stmt = Statement::Return(expr);
                    stmt_stack.push(stmt);
                }
                "ExitFunctionDefinition" => {
                    let mut sig = text.split(' ');
                    let void = matches!(sig.next().unwrap(), "void");
                    let name = sig.next().unwrap().to_string();
                    let parameters = sig.map(String::from).rev().collect();
                    let body = stmt_stack.pop().unwrap();
                    let func = Function {
                        void,
                        name,
                        parameters,
                        body,
                    };
                    program.push(func);
                }
                s => panic!("Invalid event: {}", s),
            }
        }
    }
    program
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expression_identifier() {
        let program = parse(
            "
            int main() {
                a;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Identifier(
                String::from("a"),
            ))]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_number() {
        let program = parse(
            "
            int main() {
                1;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Number(1))]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_call() {
        let program = parse(
            "
            int main() {
                f_1();
                f_2(1);
                f_3(1, 2);
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![
                Statement::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier(String::from("f_1"))),
                    arguments: Box::new(Expression::Arguments(vec![])),
                }),
                Statement::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier(String::from("f_2"))),
                    arguments: Box::new(Expression::Arguments(vec![Expression::Number(1)])),
                }),
                Statement::Expression(Expression::Call {
                    function: Box::new(Expression::Identifier(String::from("f_3"))),
                    arguments: Box::new(Expression::Arguments(vec![
                        Expression::Number(1),
                        Expression::Number(2),
                    ])),
                }),
            ]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_prefix() {
        let program = parse(
            "
            int main() {
                !1;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Prefix {
                operator: "!",
                expression: Box::new(Expression::Number(1)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_multiplicative() {
        let program = parse(
            "
            int main() {
                1 * 2 / 3;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Number(1)),
                    operator: "*",
                    right: Box::new(Expression::Number(2)),
                }),
                operator: "/",
                right: Box::new(Expression::Number(3)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_additive() {
        let program = parse(
            "
            int main() {
                1 + 2 - 3;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Number(1)),
                    operator: "+",
                    right: Box::new(Expression::Number(2)),
                }),
                operator: "-",
                right: Box::new(Expression::Number(3)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_relational() {
        let program = parse(
            "
            int main() {
                1 < 2 > 3 <= 4 >= 5;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Infix {
                        left: Box::new(Expression::Infix {
                            left: Box::new(Expression::Number(1)),
                            operator: "<",
                            right: Box::new(Expression::Number(2)),
                        }),
                        operator: ">",
                        right: Box::new(Expression::Number(3)),
                    }),
                    operator: "<=",
                    right: Box::new(Expression::Number(4)),
                }),
                operator: ">=",
                right: Box::new(Expression::Number(5)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_equality() {
        let program = parse(
            "
            int main() {
                1 == 2 != 3;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Number(1)),
                    operator: "==",
                    right: Box::new(Expression::Number(2)),
                }),
                operator: "!=",
                right: Box::new(Expression::Number(3)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_logical_and() {
        let program = parse(
            "
            int main() {
                1 && 2;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Number(1)),
                operator: "&&",
                right: Box::new(Expression::Number(2)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_logical_or() {
        let program = parse(
            "
            int main() {
                1 || 2;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Number(1)),
                operator: "||",
                right: Box::new(Expression::Number(2)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_precedence() {
        let program = parse(
            "
            int main() {
                1 || 2 && 3 == 4 < 5 + 6 * !f();
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Number(1)),
                operator: "||",
                right: Box::new(Expression::Infix {
                    left: Box::new(Expression::Number(2)),
                    operator: "&&",
                    right: Box::new(Expression::Infix {
                        left: Box::new(Expression::Number(3)),
                        operator: "==",
                        right: Box::new(Expression::Infix {
                            left: Box::new(Expression::Number(4)),
                            operator: "<",
                            right: Box::new(Expression::Infix {
                                left: Box::new(Expression::Number(5)),
                                operator: "+",
                                right: Box::new(Expression::Infix {
                                    left: Box::new(Expression::Number(6)),
                                    operator: "*",
                                    right: Box::new(Expression::Prefix {
                                        operator: "!",
                                        expression: Box::new(Expression::Call {
                                            function: Box::new(Expression::Identifier(
                                                String::from("f"),
                                            )),
                                            arguments: Box::new(Expression::Arguments(vec![])),
                                        }),
                                    }),
                                }),
                            }),
                        }),
                    }),
                }),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn expression_group() {
        let program = parse(
            "
            int main() {
                (1 + 2) * 3;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Expression(Expression::Infix {
                left: Box::new(Expression::Infix {
                    left: Box::new(Expression::Number(1)),
                    operator: "+",
                    right: Box::new(Expression::Number(2)),
                }),
                operator: "*",
                right: Box::new(Expression::Number(3)),
            })]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn statement_declaration() {
        let program = parse(
            "
            int main() {
                int a;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::Declaration(Expression::Identifier(
                String::from("a"),
            ))]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn statement_if() {
        let program = parse(
            "
            int main() {
                if (1) {
                    2;
                }
                if (3) {
                    4;
                } else {
                    5;
                }
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![
                Statement::If {
                    condition: Expression::Number(1),
                    body: Box::new(Statement::Compound(vec![Statement::Expression(
                        Expression::Number(2),
                    )])),
                    alternative: None,
                },
                Statement::If {
                    condition: Expression::Number(3),
                    body: Box::new(Statement::Compound(vec![Statement::Expression(
                        Expression::Number(4),
                    )])),
                    alternative: Some(Box::new(Statement::Compound(vec![Statement::Expression(
                        Expression::Number(5),
                    )]))),
                },
            ]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn statement_while() {
        let program = parse(
            "
            int main() {
                while (1) {
                    2;
                }
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![Statement::While {
                condition: Expression::Number(1),
                body: Box::new(Statement::Compound(vec![Statement::Expression(
                    Expression::Number(2),
                )])),
            }]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn statement_return() {
        let program = parse(
            "
            int main() {
                return;
                return 1;
            }
        ",
        );
        let expected = vec![Function {
            void: false,
            name: String::from("main"),
            parameters: vec![],
            body: Statement::Compound(vec![
                Statement::Return(None),
                Statement::Return(Some(Expression::Number(1))),
            ]),
        }];
        assert_eq!(program, expected);
    }

    #[test]
    fn function() {
        let program = parse(
            "
            int f_1() {}
            void f_2(int a) {}
            void f_3(int a, int b) {}
        ",
        );
        let expected = vec![
            Function {
                void: false,
                name: String::from("f_1"),
                parameters: vec![],
                body: Statement::Compound(vec![]),
            },
            Function {
                void: true,
                name: String::from("f_2"),
                parameters: vec![String::from("a")],
                body: Statement::Compound(vec![]),
            },
            Function {
                void: true,
                name: String::from("f_3"),
                parameters: vec![String::from("a"), String::from("b")],
                body: Statement::Compound(vec![]),
            },
        ];
        assert_eq!(program, expected);
    }
}
