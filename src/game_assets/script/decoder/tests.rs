use super::*;

const PUSH: u8 = 0x96;
const PUSH_STRING: u8 = 0;
const PUSH_INTEGER: u8 = 7;
const CALL_FUNCTION: u8 = 0x3d;
const DEFINE_FUNCTION: u8 = 0x9b;
const END: u8 = 0;
const SWF_VERSION: u8 = 8;

fn push_integer(value: i32) -> Vec<u8> {
    let mut bytes = vec![PUSH, 5, 0, PUSH_INTEGER];
    bytes.extend_from_slice(&value.to_le_bytes());
    bytes
}

fn push_string(text: &str) -> Vec<u8> {
    let length = u16::try_from(text.len() + 2).expect("a short text");
    let mut bytes = vec![PUSH];
    bytes.extend_from_slice(&length.to_le_bytes());
    bytes.push(PUSH_STRING);
    bytes.extend_from_slice(text.as_bytes());
    bytes.push(0);
    bytes
}

/// `function f() { body }` with the body's bytes following the action.
fn define_function(body: &[u8]) -> Vec<u8> {
    let mut bytes = vec![DEFINE_FUNCTION, 6, 0, b'f', 0, 0, 0];
    bytes.extend_from_slice(&u16::try_from(body.len()).expect("a short body").to_le_bytes());
    bytes.extend_from_slice(body);
    bytes
}

#[test]
fn a_huge_argument_count_takes_only_what_the_stack_holds() {
    let mut bytecode = push_integer(1);
    bytecode.extend(push_integer(i32::MAX));
    bytecode.extend(push_string("f"));
    bytecode.extend([CALL_FUNCTION, END]);
    let block = decode(&bytecode, SWF_VERSION, &[]);
    assert!(!block.damaged);
}

#[test]
fn a_truncated_action_marks_the_block_damaged() {
    let mut bytecode = push_string("value");
    bytecode.truncate(bytecode.len() - 3);
    assert!(decode(&bytecode, SWF_VERSION, &[]).damaged);
    assert!(
        !decode(&push_string("value"), SWF_VERSION, &[]).damaged,
        "a whole one is fine"
    );
}

#[test]
fn deeply_nested_functions_stop_at_the_limit() {
    let mut bytecode = vec![END];
    for _ in 0..MAXIMUM_FUNCTION_NESTING + 8 {
        bytecode = define_function(&bytecode);
    }
    let block = decode(&bytecode, SWF_VERSION, &[]);
    assert!(block.damaged);
    assert_eq!(block.functions.len(), MAXIMUM_FUNCTION_NESTING);
}

const ADD: u8 = 0x47;
const STORE_REGISTER: u8 = 0x87;
const POP: u8 = 0x17;
const PUSH_REGISTER: u8 = 4;
const COUNTER: u8 = 1;

fn store_and_drop(register: u8) -> [u8; 5] {
    [STORE_REGISTER, 1, 0, register, POP]
}

/// A local assigned a sum, then assigned itself plus one, as a compiler writes a local kept in a
/// register.
#[test]
fn a_local_in_a_register_is_assigned_and_read_by_name() {
    let mut bytecode = push_integer(3);
    bytecode.extend(push_integer(4));
    bytecode.push(ADD);
    bytecode.extend(store_and_drop(COUNTER));
    bytecode.extend([PUSH, 2, 0, PUSH_REGISTER, COUNTER]);
    bytecode.extend(push_integer(1));
    bytecode.push(ADD);
    bytecode.extend(store_and_drop(COUNTER));
    bytecode.push(END);
    let block = decode(&bytecode, SWF_VERSION, &[]);
    let number = |value: f64| Box::new(Expression::Number(value));
    let counter = Expression::Register(COUNTER);
    assert_eq!(
        block.statements,
        [
            Statement::Assign {
                target: counter.clone(),
                value: Expression::Binary(Operator::Add, number(3.0), number(4.0)),
            },
            Statement::Assign {
                target: counter.clone(),
                value: Expression::Binary(Operator::Add, Box::new(counter), number(1.0)),
            },
        ],
        "the second statement reads the register, not the first statement's value"
    );
}
