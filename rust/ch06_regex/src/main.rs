use crate::helper::DynError;

mod engine;
mod helper;

fn main() -> Result<(), DynError> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        eprintln!("usage: {} regex file", args[0]);
        return Err("invalid arguments".into());
    }

    match_file(&args[1], &args[2])?;

    Ok(())
}

fn match_file(_expr: &str, _file: &str) -> Result<(), DynError> {
    todo!()
}
