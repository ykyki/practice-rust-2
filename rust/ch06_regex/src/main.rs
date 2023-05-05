use std::{
    fs::File,
    io::{BufRead, BufReader},
};

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

fn match_file(expr: &str, file: &str) -> Result<(), DynError> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);

    engine::print(expr)?;
    println!();

    for line in reader.lines() {
        let line = line?;
        for (i, _) in line.char_indices() {
            if engine::do_matching(expr, &line[i..], true)? {
                println!("{line}");
                break;
            }
        }
    }

    Ok(())
}
