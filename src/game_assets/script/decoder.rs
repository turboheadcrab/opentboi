//! Replays AVM1's stack machine over a block of bytecode to rebuild its statements.

use swf::avm1::read::Reader;
use swf::avm1::types::Action;

use super::expression::{Expression, Operator, Statement, operator};

/// Functions defined inside functions this deep are not read: real scripts nest a few levels,
/// and damaged data could otherwise recurse until the stack runs out.
const MAXIMUM_FUNCTION_NESTING: usize = 32;

/// A function defined in a block, with its body's statements.
#[derive(Clone, Debug)]
pub struct ScriptFunction {
    #[cfg_attr(
        not(test),
        allow(dead_code, reason = "for reading gameplay values; tests use it already")
    )]
    pub name: String,
    pub statements: Vec<Statement>,
}

/// What one block of bytecode holds.
#[derive(Clone, Debug, Default)]
pub struct Block {
    pub statements: Vec<Statement>,
    pub functions: Vec<ScriptFunction>,
    /// Some bytecode could not be read (damaged, or nested too deep): what was read is kept, but
    /// the block is not the whole script.
    pub damaged: bool,
}

/// Decodes `bytecode` (a DoAction tag or a function body); `constants` is the constant pool in
/// effect (a function inherits its definer's).
pub fn decode(bytecode: &[u8], swf_version: u8, constants: &[String]) -> Block {
    decode_nested(bytecode, swf_version, constants, 0)
}

fn decode_nested(bytecode: &[u8], swf_version: u8, constants: &[String], nesting: usize) -> Block {
    let mut decoder = Decoder {
        constants: constants.to_vec(),
        stack: Vec::new(),
        just_stored: None,
        block: Block::default(),
        swf_version,
        nesting,
    };
    let mut reader = Reader::new(bytecode, swf_version);
    loop {
        let remaining_before = reader.get_ref().len();
        match reader.read_action() {
            Ok(Action::End) => break,
            Ok(action) => decoder.step(action),
            // Running out of bytes ends a block without `End`; anything else is damage. The
            // reader skips past a bad action, so reading goes on while it makes progress.
            Err(_) => {
                if remaining_before == 0 {
                    break;
                }
                decoder.block.damaged = true;
                let remaining = reader.get_ref().len();
                if remaining == 0 || remaining >= remaining_before {
                    break;
                }
            }
        }
    }
    decoder.block
}

struct Decoder {
    constants: Vec<String>,
    stack: Vec<Expression>,
    /// The register the previous action stored the top of the stack in, and that value: popped
    /// right after, it was a statement (a local assigned), not a temporary.
    just_stored: Option<(u8, Expression)>,
    block: Block,
    swf_version: u8,
    /// How many functions this block is defined inside.
    nesting: usize,
}

impl Decoder {
    fn pop(&mut self) -> Expression {
        self.stack.pop().unwrap_or(Expression::Unknown)
    }

    fn pop_name(&mut self) -> String {
        match self.pop() {
            Expression::Text(name) => name,
            other => format!("{other:?}"),
        }
    }

    /// `count` arguments, first argument first; never more than the stack holds, whatever count
    /// damaged bytecode gives.
    fn pop_arguments(&mut self) -> Vec<Expression> {
        let count = crate::whole_numbers::clamped_usize(self.pop().number().unwrap_or(0.0));
        (0..count.min(self.stack.len())).map(|_| self.pop()).collect()
    }

    fn binary(&mut self, operator: Operator) {
        let right = self.pop();
        let left = self.pop();
        self.stack
            .push(Expression::Binary(operator, Box::new(left), Box::new(right)));
    }

    fn step(&mut self, action: Action) {
        if self.dropped_store(&action) {
            return;
        }
        if let Some(operator) = operator(&action) {
            self.binary(operator);
        } else if !self.stack_action(&action) && !self.assignment_action(&action) {
            self.flow_action(action);
        }
    }

    /// Pushing and reading values; `false` if `action` is not one of those.
    fn stack_action(&mut self, action: &Action) -> bool {
        match action {
            Action::ConstantPool(pool) => {
                self.constants = pool
                    .strings
                    .iter()
                    .map(|text| text.to_string_lossy(swf::UTF_8))
                    .collect();
            }
            Action::Push(push) => {
                for value in &push.values {
                    let expression = self.value(value);
                    self.stack.push(expression);
                }
            }
            Action::Less2 => self.binary(Operator::Less),
            Action::Not => {
                let operand = self.pop();
                self.stack.push(Expression::Not(Box::new(operand)));
            }
            Action::GetVariable => {
                let name = self.pop_name();
                self.stack.push(Expression::Variable(name));
            }
            Action::GetMember => {
                let member = self.pop();
                let object = self.pop();
                self.stack.push(Expression::Member(Box::new(object), Box::new(member)));
            }
            Action::PushDuplicate => {
                let top = self.stack.last().cloned().unwrap_or(Expression::Unknown);
                self.stack.push(top);
            }
            Action::Increment | Action::Decrement => {
                let operand = self.pop();
                let step = if matches!(action, Action::Increment) { 1.0 } else { -1.0 };
                let sum = Expression::Binary(Operator::Add, Box::new(operand), Box::new(Expression::Number(step)));
                self.stack.push(sum);
            }
            Action::InitArray => {
                let elements = self.pop_arguments();
                self.stack.push(Expression::Array(elements));
            }
            _ => return false,
        }
        true
    }

    /// Storing values; `false` if `action` is not one of those.
    fn assignment_action(&mut self, action: &Action) -> bool {
        match action {
            Action::SetVariable | Action::DefineLocal => {
                let value = self.pop();
                let name = self.pop_name();
                self.assign(Expression::Variable(name), value);
            }
            Action::SetMember => {
                let value = self.pop();
                let member = self.pop();
                let object = self.pop();
                self.assign(Expression::Member(Box::new(object), Box::new(member)), value);
            }
            // A register stays a name when read: locals and parameters live in registers, and a
            // value stored on one path is not the value on every path after it.
            Action::StoreRegister(store) => {
                let value = self.stack.last().cloned().unwrap_or(Expression::Unknown);
                self.just_stored = Some((store.register, value));
            }
            _ => return false,
        }
        true
    }

    /// Calls, branches, returns and function definitions.
    fn flow_action(&mut self, action: Action) {
        match action {
            Action::Pop => {
                let dropped = self.pop();
                if matches!(dropped, Expression::Call(..) | Expression::MethodCall(..)) {
                    self.block.statements.push(Statement::Evaluate(dropped));
                }
            }
            Action::CallFunction => {
                let name = self.pop_name();
                let arguments = self.pop_arguments();
                self.stack.push(Expression::Call(name, arguments));
            }
            Action::CallMethod => {
                let method = self.pop();
                let object = self.pop();
                let arguments = self.pop_arguments();
                self.stack
                    .push(Expression::MethodCall(Box::new(object), Box::new(method), arguments));
            }
            Action::RandomNumber => {
                let limit = self.pop();
                self.stack.push(Expression::Call("random".to_owned(), vec![limit]));
            }
            Action::If(_) => {
                let condition = self.pop();
                self.block.statements.push(Statement::BranchUnless(condition));
            }
            Action::Return => {
                let value = self.pop();
                self.block.statements.push(Statement::Return(value));
            }
            Action::DefineFunction(function) => {
                self.define(function.name.to_string_lossy(swf::UTF_8), function.actions);
            }
            Action::DefineFunction2(function) => {
                self.define(function.name.to_string_lossy(swf::UTF_8), function.actions);
            }
            _ => {}
        }
    }

    fn assign(&mut self, target: Expression, value: Expression) {
        self.block.statements.push(Statement::Assign { target, value });
    }
}

mod values;

#[cfg(test)]
mod tests;
