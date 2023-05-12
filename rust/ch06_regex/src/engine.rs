use std::fmt::{Display, Formatter};

use crate::helper::DynError;

use self::evaluator::eval;

mod codegen;
mod evaluator;
mod parser;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum Instruction {
    Char(char),
    Match,
    Jump(usize),
    Split(usize, usize),
    Head,
}

impl Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruction::Char(c) => write!(f, "char {}", c),
            Instruction::Match => write!(f, "match"),
            Instruction::Jump(addr) => write!(f, "jump {:>04}", addr),
            Instruction::Split(addr1, addr2) => write!(f, "split {:>04}, {:>04}", addr1, addr2),
            Instruction::Head => write!(f, "head"),
        }
    }
}

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
struct EvalResult {
    matched: bool,
    should_be_head: bool,
}

impl EvalResult {
    fn matched() -> Self {
        Self {
            matched: true,
            should_be_head: false,
        }
    }
    fn unmatched() -> Self {
        Self {
            matched: false,
            should_be_head: false,
        }
    }
    fn matched_if_head() -> Self {
        Self {
            matched: true,
            should_be_head: true,
        }
    }

    fn merge(&self, other: &Self) -> Self {
        if self.matched {
            if other.matched {
                return Self {
                    matched: true,
                    should_be_head: self.should_be_head && other.should_be_head,
                };
            } else {
                return Self {
                    matched: true,
                    should_be_head: self.should_be_head,
                };
            }
        } else {
            return Self {
                matched: other.matched,
                should_be_head: other.should_be_head,
            };
        }
    }
}

pub fn print(expr: &str) -> Result<(), DynError> {
    println!("expr: {expr}");
    let ast = parser::parse(expr)?;
    println!("AST: {:?}", ast);

    println!();
    println!("code:");
    let code = codegen::get_code(&ast)?;
    for (n, c) in code.iter().enumerate() {
        println!("{:>04}: {c}", n);
    }

    Ok(())
}

pub fn do_matching(expr: &str, line: &str, is_depth: bool) -> Result<bool, DynError> {
    let ast = parser::parse(expr)?;
    let code = codegen::get_code(&ast)?;
    let line = line.chars().collect::<Vec<_>>();

    Ok(evaluator::eval(&code, &line, is_depth)?.matched)
}

pub(crate) fn match_line(expr: &str, line: &str) -> Result<bool, DynError> {
    let ast = parser::parse(expr)?;
    let code = codegen::get_code(&ast)?;

    for (i, _) in line.char_indices() {
        let partial_line = line[i..].chars().collect::<Vec<_>>();

        let result = eval(&code, &partial_line, true)?;
        if result.matched {
            if !result.should_be_head || i == 0 {
                return Ok(true);
            } else {
                continue;
            }
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_do_matching() {
        // パースエラー
        assert!(do_matching("+b", "bbb", true).is_err());
        assert!(do_matching("*b", "bbb", true).is_err());
        assert!(do_matching("|b", "bbb", true).is_err());
        assert!(do_matching("?b", "bbb", true).is_err());

        // パース成功、マッチ成功
        assert!(do_matching("abc|def", "def", true).unwrap());
        assert!(do_matching("(abc)*", "abcabc", true).unwrap());
        assert!(do_matching("(ab|cd)+", "abcdcd", true).unwrap());
        assert!(do_matching("abc?", "ab", true).unwrap());
        assert!(do_matching("((((a*)*)*)*)", "aaaaaaaaa", true).unwrap());
        assert!(do_matching("(a*)*b", "aaaaaaaaab", true).unwrap());
        assert!(do_matching("(a*)*b", "b", true).unwrap());
        assert!(do_matching("a**b", "aaaaaaaaab", true).unwrap());
        assert!(do_matching("a**b", "b", true).unwrap());

        // パース成功、マッチ失敗
        assert!(!do_matching("abc|def", "efa", true).unwrap());
        assert!(!do_matching("(ab|cd)+", "", true).unwrap());
        assert!(!do_matching("abc?", "acb", true).unwrap());
    }

    #[test]
    fn test_match_line() -> Result<(), DynError> {
        assert_eq!(match_line("abc|def", "abc")?, true);
        assert_eq!(match_line("abc|def", "def")?, true);
        assert_eq!(match_line("abc|def", "123def")?, true);

        assert_eq!(match_line("^abc", "abcdef")?, true);
        assert_eq!(match_line("^abc", "123abc")?, false);

        assert_eq!(match_line("^^abc", "abc")?, true);
        assert_eq!(match_line("^^abc", "123abc")?, false);

        assert_eq!(match_line("(a|^b)c", "ac")?, true);
        assert_eq!(match_line("(a|^b)c", "bc")?, true);
        assert_eq!(match_line("(a|^b)c", "123ac")?, true);
        assert_eq!(match_line("(a|^b)c", "123bc")?, false);

        assert_eq!(match_line("x(a|^b)c", "xac")?, true);
        assert_eq!(match_line("x(a|^b)c", "xbc")?, false);
        assert_eq!(match_line("x(a|^b)c", "bc")?, false);
        assert_eq!(match_line("x(a|^b)c", "123xac")?, true);
        assert_eq!(match_line("x(a|^b)c", "123xbc")?, false);

        assert_eq!(match_line("(^ab)?c", "c")?, true);
        assert_eq!(match_line("(^ab)?c", "abc")?, true);
        assert_eq!(match_line("(^ab)?c", "123c")?, true);
        assert_eq!(match_line("(^ab)?c", "123abc")?, true);

        Ok(())
    }
}
