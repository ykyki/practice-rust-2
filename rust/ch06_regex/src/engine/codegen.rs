use std::{
    error::Error,
    fmt::{Display, Formatter},
};

use super::{parser::AST, Instruction};
use crate::helper::safe_add;

#[derive(Debug)]
pub enum CodeGenError {
    PCOverFlow,
    FailStar,
    FailOr,
    FailQuestion,
}

impl Display for CodeGenError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "CodeGenError: {:?}", self)
    }
}

impl Error for CodeGenError {}

#[derive(Debug, Default)]
struct Generator {
    pc: usize,
    insts: Vec<Instruction>,
}

impl Generator {
    fn inc_pc(&mut self) -> Result<(), CodeGenError> {
        safe_add(&mut self.pc, &1, || CodeGenError::PCOverFlow)
    }

    fn gen_code(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        self.gen_expr(ast)?;
        self.inc_pc()?;
        self.insts.push(Instruction::Match);
        Ok(())
    }

    fn gen_expr(&mut self, ast: &AST) -> Result<(), CodeGenError> {
        match ast {
            AST::Char(c) => self.gen_char(*c)?,
            AST::Period => self.gen_period()?,
            AST::Caret => self.gen_caret()?,
            AST::Dollar => self.gen_dollar()?,
            AST::Or(e1, e2) => self.gen_or(e1, e2)?,
            AST::Plus(e) => self.gen_plus(e)?,
            AST::Star(e) => {
                match &**e {
                    // `(a*)*`のように`Star`が二重となっている場合にスタックオーバーフローする問題を回避するため、
                    // このような`(((r*)*)*...*)*`を再帰的に処理して1つの`r*`へと変換する。
                    AST::Star(_) => self.gen_expr(&e)?,
                    AST::Seq(e2) if e2.len() == 1 => {
                        if let Some(e3 @ AST::Star(_)) = e2.get(0) {
                            self.gen_expr(e3)?
                        } else {
                            self.gen_star(e)?
                        }
                    }
                    e => self.gen_star(&e)?,
                }
            }
            AST::Question(e) => self.gen_question(e)?,
            AST::Seq(v) => self.gen_seq(v)?,
        }

        Ok(())
    }

    fn gen_char(&mut self, c: char) -> Result<(), CodeGenError> {
        let inst = Instruction::Char(c);
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_caret(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::Head;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_dollar(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::MatchEnd;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_period(&mut self) -> Result<(), CodeGenError> {
        let inst = Instruction::AnyChar;
        self.insts.push(inst);
        self.inc_pc()?;
        Ok(())
    }

    fn gen_seq(&mut self, exprs: &[AST]) -> Result<(), CodeGenError> {
        for e in exprs {
            self.gen_expr(e)?;
        }

        Ok(())
    }

    fn gen_or(&mut self, e1: &AST, e2: &AST) -> Result<(), CodeGenError> {
        let split_addr = self.pc;
        self.inc_pc()?;

        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        self.gen_expr(e1)?;

        let jmp_addr = self.pc;
        self.insts.push(Instruction::Jump(0));

        self.inc_pc()?;
        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
        } else {
            return Err(CodeGenError::FailOr);
        }

        self.gen_expr(e2)?;

        if let Some(Instruction::Jump(l3)) = self.insts.get_mut(jmp_addr) {
            *l3 = self.pc;
        } else {
            return Err(CodeGenError::FailOr);
        }

        Ok(())
    }

    fn gen_plus(&mut self, e: &AST) -> Result<(), CodeGenError> {
        let l1 = self.pc;
        self.gen_expr(e)?;

        self.inc_pc()?;
        let split = Instruction::Split(l1, self.pc);
        self.insts.push(split);

        Ok(())
    }

    fn gen_star(&mut self, e: &AST) -> Result<(), CodeGenError> {
        let l1 = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        self.gen_expr(e)?;

        self.inc_pc()?;
        self.insts.push(Instruction::Jump(l1));

        if let Some(Instruction::Split(_, l3)) = self.insts.get_mut(l1) {
            *l3 = self.pc;
            Ok(())
        } else {
            Err(CodeGenError::FailStar)
        }
    }

    fn gen_question(&mut self, e: &AST) -> Result<(), CodeGenError> {
        let split_addr = self.pc;
        self.inc_pc()?;
        let split = Instruction::Split(self.pc, 0);
        self.insts.push(split);

        self.gen_expr(e)?;

        if let Some(Instruction::Split(_, l2)) = self.insts.get_mut(split_addr) {
            *l2 = self.pc;
            Ok(())
        } else {
            Err(CodeGenError::FailQuestion)
        }
    }
}

pub fn get_code(ast: &AST) -> Result<Vec<Instruction>, CodeGenError> {
    let mut generator = Generator::default();
    generator.gen_code(ast)?;
    Ok(generator.insts)
}

#[cfg(test)]
mod tests {
    use crate::engine::parser::parse;
    use crate::engine::parser::AST;
    use crate::helper::DynError;

    use super::Instruction::*;
    use super::*;

    #[test]
    fn test_get_code() -> Result<(), DynError> {
        assert_eq!(get_code(&AST::Char('a'))?, vec![Char('a'), Match]);
        assert_eq!(
            get_code(&AST::Or(Box::new(AST::Char('a')), Box::new(AST::Char('b'))))?,
            vec![Split(1, 3), Char('a'), Jump(4), Char('b'), Match]
        );
        // parse関数を使うのは望ましくないがfixtureを作るのが面倒なので仕方なく使う
        assert_eq!(
            get_code(&parse("ab|bc")?)?,
            vec![
                Split(1, 4),
                Char('a'),
                Char('b'),
                Jump(6),
                Char('b'),
                Char('c'),
                Match
            ]
        );
        assert_eq!(
            get_code(&parse("a.b")?)?,
            vec![Char('a'), AnyChar, Char('b'), Match]
        );
        assert_eq!(
            get_code(&parse("ab(de)?")?)?,
            vec![
                Char('a'),
                Char('b'),
                Split(3, 5),
                Char('d'),
                Char('e'),
                Match
            ]
        );
        assert_eq!(
            get_code(&parse("a(bc|e+)*")?)?,
            vec![
                Char('a'),   // 0:
                Split(2, 9), // 1: *のsplit
                Split(3, 6), // 2: |のsplit
                Char('b'),   // 3:
                Char('c'),   // 4:
                Jump(8),     // 5: |のjump
                Char('e'),   // 6:
                Split(6, 8), // 7: +のsplit
                Jump(1),     // 8: *のjump
                Match
            ]
        );
        assert_eq!(get_code(&parse("^a")?)?, vec![Head, Char('a'), Match]);
        assert_eq!(
            get_code(&parse("a^a")?)?,
            vec![Char('a'), Head, Char('a'), Match]
        );
        assert_eq!(
            get_code(&parse("(a|^b)c")?)?,
            vec![
                Split(1, 3), // 0:
                Char('a'),   // 1:
                Jump(5),     // 2:
                Head,        // 3:
                Char('b'),   // 4:
                Char('c'),   // 5:
                Match,       // 6:
            ]
        );
        assert_eq!(get_code(&parse("a$")?)?, vec![Char('a'), MatchEnd, Match]);
        assert_eq!(
            get_code(&parse("a$b")?)?,
            vec![Char('a'), MatchEnd, Char('b'), Match]
        );
        assert_eq!(
            get_code(&parse("a(b|c$)")?)?,
            vec![
                Char('a'),   // 0:
                Split(2, 4), // 1:
                Char('b'),   // 2:
                Jump(6),     // 3:
                Char('c'),   // 4:
                MatchEnd,    // 5:
                Match,       // 6:
            ]
        );

        Ok(())
    }
}
