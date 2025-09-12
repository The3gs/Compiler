use std::collections::HashMap;

use crate::ast;

#[derive(Debug)]
pub enum Error {
    UndeclaredVariable(String),
    CallingNonFunction(String, ast::Type),
    NonMatchingTypes(ast::Type, ast::Type),
}

pub fn check(ast: &Vec<ast::Declaration>) -> Result<(), Error> {
    let mut global_types: HashMap<&String, ast::Type> = HashMap::new();
    for declaration in ast {
        match declaration {
            ast::Declaration::Function {
                name,
                arguments,
                return_type,
                body: _,
            } => {
                global_types.insert(
                    name,
                    ast::Type::Fun(
                        arguments.iter().map(|(_, t)| t.clone()).collect(),
                        Box::new(return_type.clone()),
                    ),
                );
            }
        }
    }
    for declaration in ast {
        match declaration {
            ast::Declaration::Function {
                name: _,
                arguments,
                return_type,
                body,
            } => {
                let mut local_vars: HashMap<&String, ast::Type> = HashMap::new();

                for (name, typ) in global_types.iter() {
                    local_vars.insert(name, typ.clone());
                }
                for (name, typ) in arguments.iter() {
                    local_vars.insert(name, typ.clone());
                }

                for statement in body {
                    match statement {
                        ast::Statement::Let(name, typ, expression) => {
                            check_expression(expression, typ.as_ref().unwrap(), &local_vars)?;
                            local_vars.insert(name, typ.clone().unwrap());
                        }
                        ast::Statement::Expr(_) => {
                            todo!("Implement inference for standalone expressions")
                        }
                        ast::Statement::Return(expression) => {
                            check_expression(expression, return_type, &local_vars)?
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

pub fn check_expression(
    expression: &ast::Expression,
    typ: &ast::Type,
    env: &HashMap<&String, ast::Type>,
) -> Result<(), Error> {
    match expression {
        ast::Expression::Block(_, _) => todo!("Implement blocks"),
        ast::Expression::Call(function, expressions) => match env.get(function) {
            Some(ast::Type::Fun(arg_types, return_type)) => {
                for (expression, arg_type) in expressions.iter().zip(arg_types) {
                    check_expression(expression, arg_type, env)?
                }
                if typ != return_type.as_ref() {
                    return Err(Error::NonMatchingTypes(
                        typ.clone(),
                        return_type.as_ref().clone(),
                    ));
                }
            }
            Some(t) => return Err(Error::CallingNonFunction(function.clone(), t.clone())),
            None => return Err(Error::UndeclaredVariable(function.clone())),
        },
        ast::Expression::Variable(name) => match env.get(name) {
            Some(var_type) => {
                if var_type == typ {
                    return Ok(());
                } else {
                    return Err(Error::NonMatchingTypes(typ.clone(), var_type.clone()));
                }
            }
            None => return Err(Error::UndeclaredVariable(name.clone())),
        },
        ast::Expression::NumLiteral(_) => {
            if !matches!(typ, ast::Type::U32) {
                return Err(Error::NonMatchingTypes(typ.clone(), ast::Type::U32));
            }
        }
        ast::Expression::Add(expression, expression1)
        | ast::Expression::Sub(expression, expression1)
        | ast::Expression::Mul(expression, expression1)
        | ast::Expression::Div(expression, expression1)
        | ast::Expression::Mod(expression, expression1) => {
            if typ == &ast::Type::U32 {
                return check_expression(expression, typ, env).and(check_expression(
                    expression1,
                    typ,
                    env,
                ));
            } else {
                return Err(Error::NonMatchingTypes(typ.clone(), ast::Type::U32));
            }
        }
    }
    Ok(())
}
