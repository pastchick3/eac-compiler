use crate::ast::{Expression, Function, Program, Statement};
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
    let mut s = String::new();
    for _ in 0..len {
        s.push(' ');
    }
    CString::new(s).unwrap().into_raw()
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
    let mut compound_stmt_ptr = 0;
    unsafe {
        for (tag, text) in &EVENTS {
            println!("{:?} - {:?} - {:?}", tag, text, expr_stack);
            match tag.as_str() {
                "ExitPrimaryExpression" => {
                    let expr = match text.parse::<i32>() {
                        Ok(num) => Expression::Number(num),
                        Err(_) => Expression::Identifier(text.to_string()),
                    };
                    expr_stack.push(expr);
                }
                "ExitUnaryExpression" => {
                    let expr = match text.as_str() {
                        "!" => {
                            let expr = expr_stack.pop().unwrap();
                            Expression::Prefix {
                                operator: "!",
                                expression: Box::new(expr),
                            }
                        }
                        op => panic!("Invalid prefix operator: {}", op),
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
                    let args = match expr_stack.pop().unwrap() {
                        Expression::Arguments(mut args) => {
                            args.push(arg);
                            args
                        }
                        expr => {
                            expr_stack.push(expr);
                            vec![arg]
                        }
                    };
                    expr_stack.push(Expression::Arguments(args));
                }
                "EnterCompoundStatement" => {
                    compound_stmt_ptr = stmt_stack.len();
                }
                "ExitCompoundStatement" => {
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
                "ExitFunctionDefinition" => {
                    let mut sig = text.split(' ');
                    let void = match sig.next().unwrap() {
                        "void" => true,
                        _ => false,
                    };
                    let name = sig.next().unwrap().to_string();
                    let parameters = sig.map(|p| String::from(p)).rev().collect();
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
