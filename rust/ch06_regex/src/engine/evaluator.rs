use std::{error::Error, fmt::Display};

use super::EvalResult;
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
) -> Result<EvalResult, EvalError> {
    let mut should_be_head = false;

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
                        return Ok(EvalResult::unmatched());
                    }
                } else {
                    return Ok(EvalResult::unmatched());
                }
            }
            Instruction::Match => {
                return if should_be_head {
                    Ok(EvalResult::matched_if_head())
                } else {
                    Ok(EvalResult::matched())
                };
            }
            Instruction::MatchEnd => {
                let is_end = line.get(sp).is_none();

                if !is_end {
                    return Ok(EvalResult::unmatched());
                }

                return if should_be_head {
                    Ok(EvalResult::matched_if_head())
                } else {
                    Ok(EvalResult::matched())
                };
            }
            Instruction::Jump(addr) => {
                pc = *addr;
            }
            Instruction::Split(addr1, addr2) => {
                return Ok(
                    eval_depth(inst, line, *addr1, sp)?.merge(&eval_depth(inst, line, *addr2, sp)?)
                );
            }
            Instruction::Head => {
                if sp != 0 {
                    return Ok(EvalResult::unmatched());
                } else {
                    should_be_head = true;
                    safe_add(&mut pc, &1, || EvalError::PCOverFlow)?;
                }
            }
        }
    }
}

fn eval_width(_inst: &[Instruction], _line: &[char]) -> Result<EvalResult, EvalError> {
    todo!()
}

pub(super) fn eval(
    inst: &[Instruction],
    line: &[char],
    is_depth: bool,
) -> Result<EvalResult, EvalError> {
    if is_depth {
        eval_depth(inst, line, 0, 0)
    } else {
        eval_width(inst, line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EvalResult;
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
            EvalResult::matched(),
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Char('b'), Char('c'), Match,],
                &['a', 'b', 'c', 'd'],
                0,
                0
            )?,
            EvalResult::matched(),
        );
        assert_eq!(eval_depth(&[Match], &[], 0, 0)?, EvalResult::matched());
        assert_eq!(
            eval_depth(&[Char('b')], &['a'], 0, 0)?,
            EvalResult::unmatched()
        );
        assert_eq!(
            eval_depth(&[Jump(2), Char('a'), Match], &['b'], 0, 0)?,
            EvalResult::matched(),
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Split(2, 4), Char('b'), Char('c'), Match,],
                &['a', 'b', 'c'],
                0,
                0
            )?,
            EvalResult::matched(),
        );
        assert_eq!(
            eval_depth(
                &[Char('a'), Split(2, 4), Char('b'), Char('c'), Match,],
                &['a'],
                0,
                0
            )?,
            EvalResult::matched(),
        );
        assert_eq!(
            eval_depth(&[Head, Char('a'), Char('b'), Match], &['a', 'b'], 0, 0)?,
            EvalResult::matched_if_head(),
        );
        assert_eq!(
            eval_depth(&[Char('a'), Head, Char('b'), Match], &['a', 'b'], 0, 0)?,
            EvalResult::unmatched(),
        );
        assert_eq!(
            eval_depth(
                &[
                    Split(1, 1), // 0:
                    Head,        // 1:
                    Char('a'),   // 2:
                    Jump(6),     // 3:
                    Char('b'),   // 4:
                    Char('c'),   // 5:
                    Match,       // 6:
                ],
                &['a'],
                0,
                0
            )?,
            EvalResult::matched_if_head()
        );
        assert_eq!(
            eval_depth(
                &[
                    Split(1, 4), // 0:
                    Head,        // 1:
                    Char('a'),   // 2:
                    Jump(6),     // 3:
                    Char('b'),   // 4:
                    Char('c'),   // 5:
                    Match,       // 6:
                ],
                &['b', 'c'],
                0,
                0
            )?,
            EvalResult::matched()
        );
        assert_eq!(
            eval_depth(
                &[
                    Char('a'),   // 0:
                    Split(2, 5), // 1:
                    Head,        // 2:
                    Char('b'),   // 3:
                    Jump(6),     // 4:
                    Char('d'),   // 5:
                    Char('e'),   // 6:
                    Match,       // 7:
                ],
                &['a', 'b'],
                0,
                0
            )?,
            EvalResult::unmatched()
        );
        assert_eq!(
            eval_depth(
                &[
                    Char('a'),   // 0:
                    Split(2, 5), // 1:
                    Head,        // 2:
                    Char('b'),   // 3:
                    Jump(7),     // 4:
                    Char('d'),   // 5:
                    Char('e'),   // 6:
                    Match,       // 7:
                ],
                &['a', 'd', 'e'],
                0,
                0
            )?,
            EvalResult::matched()
        );
        assert_eq!(
            eval_depth(&[Char('a'), MatchEnd,], &['a'], 0, 0)?,
            EvalResult::matched()
        );
        assert_eq!(
            eval_depth(&[Char('a'), MatchEnd,], &['a', 'b'], 0, 0)?,
            EvalResult::unmatched()
        );
        assert_eq!(
            eval_depth(&[Char('a'), MatchEnd,], &['c'], 0, 0)?,
            EvalResult::unmatched()
        );
        assert_eq!(
            eval_depth(
                &[
                    Head,      // 0:
                    Char('a'), // 1:
                    MatchEnd,  // 2:
                ],
                &['a'],
                0,
                0
            )?,
            EvalResult::matched_if_head()
        );
        assert_eq!(
            eval_depth(
                &[
                    Char('a'),   // 0:
                    Split(2, 4), // 1:
                    Char('b'),   // 2:
                    Jump(6),     // 3:
                    Char('c'),   // 4:
                    MatchEnd,    // 5:
                    Match,       // 6:
                ],
                &['a', 'b'],
                0,
                0
            )?,
            EvalResult::matched()
        );
        assert_eq!(
            eval_depth(
                &[
                    Char('a'),   // 0:
                    Split(2, 4), // 1:
                    Char('b'),   // 2:
                    Jump(6),     // 3:
                    Char('c'),   // 4:
                    MatchEnd,    // 5:
                    Match,       // 6:
                ],
                &['a', 'c'],
                0,
                0
            )?,
            EvalResult::matched()
        );
        assert_eq!(
            eval_depth(
                &[
                    Char('a'),   // 0:
                    Split(2, 4), // 1:
                    Char('b'),   // 2:
                    Jump(6),     // 3:
                    Char('c'),   // 4:
                    MatchEnd,    // 5:
                    Match,       // 6:
                ],
                &['a', 'd'],
                0,
                0
            )?,
            EvalResult::unmatched()
        );

        Ok(())
    }
}
