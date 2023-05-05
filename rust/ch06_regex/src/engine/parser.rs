use crate::helper::DynError;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::mem;

#[derive(Debug)]
pub enum AST {
    Char(char),
    Plus(Box<AST>),
    Star(Box<AST>),
    Question(Box<AST>),
    Or(Box<AST>, Box<AST>),
    Seq(Vec<AST>),
}

#[derive(Debug)]
pub enum ParseError {
    InvalidEscape(usize, char),
    InvalidRightParen(usize),
    NoPrev(usize),
    NoRightParen,
    Empty,
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::InvalidEscape(pos, c) => {
                write!(f, "ParseError: invalid escape: pos = {pos}, char = '{c}'")
            }
            ParseError::InvalidRightParen(pos) => {
                write!(f, "ParseError: invalid right parenthesis: pos = {pos}")
            }
            ParseError::NoPrev(pos) => write!(f, "ParseError: no previous expression: pos = {pos}"),
            ParseError::NoRightParen => {
                write!(f, "ParseError: no right parenthesis")
            }
            ParseError::Empty => write!(f, "ParseError: empty expression"),
        }
    }
}

fn parse_escape(pos: usize, c: char) -> Result<AST, ParseError> {
    match c {
        '\\' | '(' | ')' | '|' | '+' | '*' | '?' => Ok(AST::Char(c)),
        _ => {
            let err = ParseError::InvalidEscape(pos, c);
            Err(err)
        }
    }
}

enum PSQ {
    Plus,
    Star,
    Question,
}

pub fn parse(expr: &str) -> Result<AST, DynError> {
    enum ParseState {
        Char,
        Escape,
    }

    let mut seq = Vec::new();
    let mut seq_or = Vec::new();
    let mut stack = Vec::new();
    let mut state = ParseState::Char;

    for (i, c) in expr.chars().enumerate() {
        match &state {
            ParseState::Char => match c {
                '+' => parse_plus_question(&mut seq, PSQ::Plus, i)?,
                '*' => parse_plus_question(&mut seq, PSQ::Star, i)?,
                '?' => parse_plus_question(&mut seq, PSQ::Question, i)?,
                '(' => {
                    let prev = mem::take(&mut seq);
                    let prev_or = mem::take(&mut seq_or);
                    stack.push((prev, prev_or));
                }
                ')' => {
                    if let Some((mut prev, prev_or)) = stack.pop() {
                        if !seq.is_empty() {
                            seq_or.push(AST::Seq(seq));
                        }

                        if let Some(ast) = fold_or(seq_or) {
                            prev.push(ast);
                        }
                        seq = prev;
                        seq_or = prev_or;
                    } else {
                        return Err(Box::new(ParseError::InvalidRightParen(i)));
                    }
                }
                '|' => {}
                '\\' => {}
                _ => {
                    seq.push(AST::Char(c));
                }
            },
            ParseState::Escape => {}
        }
    }

    if !stack.is_empty() {
        return Err(Box::new(ParseError::NoRightParen));
    }

    if !seq.is_empty() {
        seq_or.push(AST::Seq(seq));
    }

    if let Some(ast) = fold_or(seq_or) {
        Ok(ast)
    } else {
        Err(Box::new(ParseError::Empty))
    }
}

fn parse_plus_question(seq: &mut Vec<AST>, ast_type: PSQ, pos: usize) -> Result<(), ParseError> {
    if let Some(prev) = seq.pop() {
        let ast = match ast_type {
            PSQ::Plus => AST::Plus(Box::new(prev)),
            PSQ::Star => AST::Star(Box::new(prev)),
            PSQ::Question => AST::Question(Box::new(prev)),
        };
        seq.push(ast);
        Ok(())
    } else {
        Err(ParseError::NoPrev(pos))
    }
}

fn fold_or(mut seq_or: Vec<AST>) -> Option<AST> {
    if seq_or.len() > 1 {
        let mut ast = seq_or.pop().unwrap();
        seq_or.reverse();
        for s in seq_or {
            ast = AST::Or(Box::new(s), Box::new(ast));
        }
        Some(ast)
    } else {
        seq_or.pop()
    }
}
