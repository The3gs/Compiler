mod ast;
mod compiler;
mod parser;
mod typechecker;
mod virtual_machine;

fn main() {
    let Some(file_name) = std::env::args().skip(1).next() else {
        eprintln!("Usage: {} [filename]", std::env::args().next().unwrap());
        return;
    };

    let Ok(input) = std::fs::read_to_string(&file_name) else {
        eprintln!("Error opening file {:?}", file_name);
        return;
    };

    let program = match parser::parse(&input) {
        Ok(program) => program,
        Err(e) => {
            eprintln!("Error parsing file");
            eprintln!("{e:?}");
            return;
        }
    };

    if let Err(e) = typechecker::check(&program) {
        eprintln!("Typechecking error");
        eprintln!("{e:?}");
        return;
    };

    println!("{:?}", program);

    let mut vm = compiler::compile(&program);
    let result = vm.run();
    println!("Program exited with code {result}");
}
