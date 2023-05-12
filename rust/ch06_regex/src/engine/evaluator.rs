use std::{error::Error, fmt::Display};

use super::Instruction;
use crate::helper::safe_add;

#[derive(Debug)]
pub enum EvalError {
    PCOverFlow,
    SPOverFlow,
    InvalidPC,
    // InvalidContext,
}

impl Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EvalError: {:?}", self)
    }
}

impl Error for EvalError {}

fn eval_depth(
    inst: &[Instruction],
    line: &[char],
    mut pc: usize,
    mut sp: usize,
) -> Result<bool, EvalError> {
    loop {
        let next = if let Some(i) = inst.get(pc) {
            i
        } else {
            return Err(EvalError::InvalidPC);
        };

        match next {
            Instruction::Char(c) => {
                if let Some(sp_c) = line.get(sp) {
                    if c == sp_c {
                        safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                        safe_add(&mut sp, &1, || EvalError::SPOverFlow)?;
                    } else {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }
            Instruction::Match => {
                return Ok(true);
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                if eval_depth(inst, line, *addr1, sp)? || eval_depth(inst, line, *addr2, sp)? {
                    return Ok(true);
                } else {
                    return Ok(false);
                }
            }
        }
    }
}

fn eval_width(_inst: &[Instruction], _line: &[char]) -> Result<bool, EvalError> {
    todo!()
}

pub fn eval(inst: &[Instruction], line: &[char], is_depth: bool) -> Result<bool, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0)
    } else {
        eval_width(inst, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::Instruction::*;

    #[test]
    fn test_eval_depth() -> Result<(), EvalError> {
        assert_eq!(
            eval_depth(
                &[Char('a'), Char('b'), Char('c'), Match,],
                &['a', 'b', 'c'],
                0,
                0
            )?,
            true
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Char('b'), Char('c'), Match,],
                &['a', 'b', 'c', 'd'],
                0,
                0
            )?,
            true
        );
        assert_eq!(eval_depth(&[Match], &[], 0, 0)?, true);
        assert_eq!(eval_depth(&[Char('b')], &['a'], 0, 0)?, false);
        assert_eq!(
            eval_depth(&[Jump(2), Char('a'), Match], &['b'], 0, 0)?,
            true
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Split(2, 4), Char('b'), Char('c'), Match,],
                &['a', 'b', 'c'],
                0,
                0
            )?,
            true
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Split(2, 4), Char('b'), Char('c'), Match,],
                &['a'],
                0,
                0
            )?,
            true
        );

        Ok(())
    }
}
