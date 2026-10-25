//! What the bytecode reader rebuilds: expressions and the statements made of them.

use swf::avm1::types::Action;

/// A value computed by the original's code.
#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Number(f64),
    Text(String),
    Boolean(bool),
    Undefined,
    Null,
    /// A function's register (a numbered local in the decompiler's output).
    Register(u8),
    Variable(String),
    /// `object.member` or `object[member]`.
    Member(Box<Expression>, Box<Expression>),
    Binary(Operator, Box<Expression>, Box<Expression>),
    Not(Box<Expression>),
    /// `name(arguments)`, including Flash's random-number instruction.
    Call(String, Vec<Expression>),
    /// `object.method(arguments)`.
    MethodCall(Box<Expression>, Box<Expression>, Vec<Expression>),
    /// `[a, b, c]`.
    Array(Vec<Expression>),
    /// Anything the reader does not model yet.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Less,
    Greater,
    Equals,
    StrictEquals,
    LogicalAnd,
    LogicalOr,
    BitAnd,
    BitOr,
}

/// One step of a function (or of the frame script) in the order the bytecode runs it; branches
/// keep their condition, but jumps are not followed.
#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    /// `target = value` (also how `x *= 2` arrives: `x = x * 2`).
    Assign {
        target: Expression,
        value: Expression,
    },
    /// A call or other expression whose result is dropped.
    Evaluate(Expression),
    /// `if (!condition) jump`: the code after it runs when `condition` is false.
    BranchUnless(Expression),
    Return(Expression),
}

impl Expression {
    pub fn number(&self) -> Option<f64> {
        match self {
            Self::Number(value) => Some(*value),
            _ => None,
        }
    }

    /// The name of a variable or of a member with a constant name.
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "for reading gameplay values; tests use it already")
    )]
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Variable(name) => Some(name),
            Self::Member(_, member) => match member.as_ref() {
                Self::Text(name) => Some(name),
                _ => None,
            },
            _ => None,
        }
    }
}

/// The binary operator an action applies, if it is one.
pub fn operator(action: &Action) -> Option<Operator> {
    let operator = match action {
        Action::Add | Action::Add2 | Action::StringAdd => Operator::Add,
        Action::Subtract => Operator::Subtract,
        Action::Multiply => Operator::Multiply,
        Action::Divide => Operator::Divide,
        Action::Modulo => Operator::Modulo,
        Action::Less => Operator::Less,
        Action::Greater => Operator::Greater,
        Action::Equals | Action::Equals2 => Operator::Equals,
        Action::StrictEquals => Operator::StrictEquals,
        Action::And => Operator::LogicalAnd,
        Action::Or => Operator::LogicalOr,
        Action::BitAnd => Operator::BitAnd,
        Action::BitOr => Operator::BitOr,
        _ => return None,
    };
    Some(operator)
}
