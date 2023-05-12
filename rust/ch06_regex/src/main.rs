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
        if engine::match_line(expr, &line)? {
            println!("{line}");
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::helper::{safe_add, SafeAdd};

    #[test]
    fn test_safe_add() {
        let n: usize = 10;
        assert_eq!(Some(30), n.safe_add(&20));

        let n: usize = !0; // 2^64 - 1 (64 bits CPU)
        assert_eq!(None, n.safe_add(&1));

        let mut n: usize = 10;
        assert!(safe_add(&mut n, &20, || ()).is_ok());

        let mut n: usize = !0;
        assert!(safe_add(&mut n, &1, || ()).is_err());
    }
}
