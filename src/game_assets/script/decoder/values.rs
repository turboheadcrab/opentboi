//! Function definitions, literal values and register stores met while decoding.

use swf::avm1::types::{Action, Value};

use super::{Decoder, Expression, MAXIMUM_FUNCTION_NESTING, ScriptFunction, Statement, decode_nested};

impl Decoder {
    /// A value stored in a register and dropped right after was a statement (a local assigned), not
    /// a temporary: it becomes an assignment.
    pub(super) fn dropped_store(&mut self, action: &Action) -> bool {
        let just_stored = self.just_stored.take();
        if let (Action::Pop, Some((register, value))) = (action, just_stored)
            && self.stack.last() == Some(&value)
        {
            self.stack.pop();
            self.block.statements.push(Statement::Assign {
                target: Expression::Register(register),
                value,
            });
            return true;
        }
        false
    }

    pub(super) fn define(&mut self, name: String, body: &[u8]) {
        if self.nesting >= MAXIMUM_FUNCTION_NESTING {
            self.block.damaged = true;
            return;
        }
        let inner = decode_nested(body, self.swf_version, &self.constants, self.nesting + 1);
        self.block.damaged |= inner.damaged;
        self.block.functions.push(ScriptFunction {
            name,
            statements: inner.statements,
        });
        self.block.functions.extend(inner.functions);
    }

    pub(super) fn value(&self, value: &Value) -> Expression {
        match *value {
            Value::Undefined => Expression::Undefined,
            Value::Null => Expression::Null,
            Value::Bool(flag) => Expression::Boolean(flag),
            Value::Int(number) => Expression::Number(f64::from(number)),
            Value::Float(number) => Expression::Number(f64::from(number)),
            Value::Double(number) => Expression::Number(number),
            Value::Str(text) => Expression::Text(text.to_string_lossy(swf::UTF_8)),
            Value::ConstantPool(index) => self
                .constants
                .get(usize::from(index))
                .map_or(Expression::Unknown, |text| Expression::Text(text.clone())),
            Value::Register(register) => Expression::Register(register),
        }
    }
}
