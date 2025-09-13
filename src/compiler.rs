use crate::{ast, virtual_machine};

pub fn compile(ast: &Vec<ast::Declaration>) -> virtual_machine::VirtualMachine {
    let mut functions = vec![];

    let function_names: Vec<String> = ast
        .iter()
        .flat_map(|decl| match decl {
            ast::Declaration::Function { name, .. } => Some(name.clone()),
        })
        .collect();

    for declaration in ast {
        match declaration {
            ast::Declaration::Function {
                name,
                arguments,
                return_type,
                body,
            } => {
                let mut local_vars = Vec::new();
                let mut operations = Vec::new();
                for statement in body {
                    compile_statement(
                        statement,
                        &mut operations,
                        &mut local_vars,
                        arguments,
                        &function_names,
                    );
                }
                functions.push(virtual_machine::Function::from_operations(
                    name.clone(),
                    operations,
                ))
            }
        }
    }

    virtual_machine::VirtualMachine::from_functions(functions)
}

fn size_of(t: &ast::Type) -> u32 {
    match t {
        ast::Type::Fun(items, _) => 1,
        ast::Type::U32 => 1,
    }
}

fn compile_statement(
    statement: &ast::Statement,
    operations: &mut Vec<virtual_machine::Operation>,
    local_vars: &mut Vec<Option<String>>,
    arguments: &Vec<(String, ast::Type)>,
    function_names: &Vec<String>,
) {
    match statement {
        ast::Statement::Let(name, var_type, expression) => {
            let var_size = size_of(var_type.as_ref().unwrap());
            if var_size > 0 {
                local_vars.push(Some(name.clone()));
                for _ in 1..var_size {
                    local_vars.push(None);
                }
            }
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            local_vars.pop(); // Expressions add their own None instance, so we must remove it.
        }
        ast::Statement::Expr(expression) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Pop);
            local_vars.pop();
        }
        ast::Statement::Return(expression) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            local_vars.pop();
            operations.push(virtual_machine::Operation::Put(local_vars.len() as u32 + 2));
            for _ in 0..local_vars.len() {
                operations.push(virtual_machine::Operation::Pop);
            }
            operations.push(virtual_machine::Operation::Return);
        }
    }
}

fn compile_expression(
    expression: &ast::Expression,
    operations: &mut Vec<virtual_machine::Operation>,
    local_vars: &mut Vec<Option<String>>,
    arguments: &Vec<(String, ast::Type)>,
    function_names: &Vec<String>,
) {
    match expression {
        ast::Expression::Call(fn_name, expressions) => {
            for expression in expressions {
                compile_expression(
                    expression,
                    operations,
                    local_vars,
                    arguments,
                    function_names,
                );
            }
            operations.push(virtual_machine::Operation::Call(
                function_names.iter().position(|s| s == fn_name).unwrap() as u32,
            ))
        }
        ast::Expression::NumLiteral(n) => {
            operations.push(virtual_machine::Operation::Push(*n));
            local_vars.push(None);
        }
        ast::Expression::Variable(name) => {
            let index = local_vars
                .iter()
                .rev()
                .position(|var_name| var_name.as_ref() == Some(name))
                .or_else(|| {
                    arguments
                        .iter()
                        .rev()
                        .position(|var_name| &var_name.0 == name)
                        .map(|i| i + local_vars.len() + 2)
                })
                .unwrap();
            operations.push(virtual_machine::Operation::Get(index as u32));
            local_vars.push(None);
        }
        ast::Expression::Add(expression, expression1) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            compile_expression(
                expression1,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Add);
            local_vars.pop();
        }
        ast::Expression::Sub(expression, expression1) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            compile_expression(
                expression1,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Sub);
            local_vars.pop();
        }
        ast::Expression::Mul(expression, expression1) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            compile_expression(
                expression1,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Mul);
            local_vars.pop();
        }
        ast::Expression::Div(expression, expression1) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            compile_expression(
                expression1,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Div);
            local_vars.pop();
        }
        ast::Expression::Mod(expression, expression1) => {
            compile_expression(
                expression,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            compile_expression(
                expression1,
                operations,
                local_vars,
                arguments,
                function_names,
            );
            operations.push(virtual_machine::Operation::Mod);
            local_vars.pop();
        }
    }
}
