fn main() -> std::io::Result<()> {
    hello(&mut std::io::stdout())?;
    Ok(())
}

fn hello(writer: &mut impl std::io::Write) -> std::io::Result<()> {
    writeln!(writer, "Hello, world!")?;
    writeln!(writer, "This is ykyki.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hello() -> std::io::Result<()> {
        let mut buf = Vec::new();

        hello(&mut buf)?;

        assert_eq!(buf, b"Hello, world!\nThis is ykyki.\n");
        Ok(())
    }
}
