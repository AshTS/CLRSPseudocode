#![allow(dead_code)]

use std::{rc::Rc, cell::RefCell, io::{BufRead, Write}};

use pseudocode::{tokenizer::TokenStream, interpreter::{RunTime, RuntimeError}, compile_function, VMFunction, error::GenericError};
mod args;

fn execute() {
    
}

fn main()
{
    use clap::Parser;
    let args = args::Arguments::parse();

    if let args::SubCommand::Tokenize{ file } = args.sub_command {
        let name = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).expect("Unable to read file");

        let tokens = TokenStream::from_source(text.as_str(), &name);
        for token in tokens {
            println!("{}", token);
        }
    }
    else if let args::SubCommand::Parse{ file } = args.sub_command {
        let name = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).expect("Unable to read file");

        let mut tokens = TokenStream::from_source(text.as_str(), &name);
        let mut context = pseudocode::parser::ParserContext::new(&mut tokens);
        
        
        let parse_tree = match context.parse_document() {
            Ok((parse_tree, errors)) => 
            {
                for error in errors {
                    println!("{}", error);
                }

                parse_tree
            },
            Err(errors) => {
                println!("Parsing Failed");

                for error in errors {
                    println!("{}", error);
                }

                return;
            },
        };

        dbg!(parse_tree);
    }
    else if let args::SubCommand::Execute{ file } = args.sub_command {
        let name = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).expect("Unable to read file");

        let mut tokens = TokenStream::from_source(text.as_str(), &name);
        let mut context = pseudocode::parser::ParserContext::new(&mut tokens);
        
        
        let parse_tree = match context.parse_document() {
            Ok((parse_tree, errors)) => 
            {
                for error in errors {
                    println!("{}", error);
                }

                parse_tree
            },
            Err(errors) => {
                println!("Parsing Failed");

                for error in errors {
                    println!("{}", error);
                }

                return;
            },
        };

        let executor = Rc::new(RefCell::new(RunTime::new(parse_tree)));

        let result = RunTime::inner_execute_function(executor, "Test".to_string(), vec![]);
        
        if let Err(RuntimeError::FinishedError(e)) = &result {
            println!("{}", e);
        }
        else if let Err(e) = &result {
            println!("{:?}", e);
        }
        else if let Ok(Some(v)) = result {
            println!("{}", v);
        }
        else {
            println!("Function Test Not Defined");
        }
    }
    else if let args::SubCommand::Compile{ file } = args.sub_command {
        let name = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).expect("Unable to read file");

        let mut tokens = TokenStream::from_source(text.as_str(), &name);
        let mut context = pseudocode::parser::ParserContext::new(&mut tokens);
        
        
        let parse_tree = match context.parse_document() {
            Ok((parse_tree, errors)) => 
            {
                for error in errors {
                    println!("{}", error);
                }

                parse_tree
            },
            Err(errors) => {
                println!("Parsing Failed");

                for error in errors {
                    println!("{}", error);
                }

                return;
            },
        };

        let functions = parse_tree.into_iter().map(compile_function).collect::<Result<Vec<VMFunction>, GenericError>>();
        if let Err(e) = functions {
            println!("{}", e);
        }
        else if let Ok(functions) = functions {
            for f in functions {
                println!("{}\n", f);
            }
        }
    }
    else if let args::SubCommand::VMRun{ file, supress: hide, no_wait, instructions } = args.sub_command {
        let name = file.to_string_lossy().to_string();
        let text = std::fs::read_to_string(file).expect("Unable to read file");

        let mut tokens = TokenStream::from_source(text.as_str(), &name);
        let mut context = pseudocode::parser::ParserContext::new(&mut tokens);
        
        
        let parse_tree = match context.parse_document() {
            Ok((parse_tree, errors)) => 
            {
                for error in errors {
                    println!("{}", error);
                }

                parse_tree
            },
            Err(errors) => {
                println!("Parsing Failed");

                for error in errors {
                    println!("{}", error);
                }

                return;
            },
        };

        let functions = parse_tree.into_iter().map(compile_function).collect::<Result<Vec<VMFunction>, GenericError>>();
        let functions = if let Err(e) = functions {
            println!("{}", e);
            return;
        }
        else if let Ok(functions) = functions {
            functions
        }
        else {
            unimplemented!()
        };

        let mut runtime = pseudocode::virtualmachine::Runtime::load(functions);

        if let Err(e) = runtime.start_execution("Test") {
            println!("{}", e);
            return;
        }

        'outer: loop {
            if !hide {
                print!("{}", runtime);
                let _ = std::io::stdout().flush();
            }

            if !no_wait {
                let mut s = String::new();
            
                std::io::stdin().lock().read_line(&mut s).unwrap();
            }
            
            runtime.clear();
            loop {
                let v = runtime.single_step(instructions);
                if let Err(e) = v {
                    println!("{}", e);
                    break 'outer;
                }
                else if let Ok(v) = v {
                    if v { break; }
                }
            }

            if runtime.is_done() {
                break;
            }
        }
    }

}
