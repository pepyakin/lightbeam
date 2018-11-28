use super::{translate, TranslatedModule};
use wabt;

fn translate_wat(wat: &str) -> TranslatedModule {
    let wasm = wabt::wat2wasm(wat).unwrap();
    let compiled = translate(&wasm).unwrap();
    compiled
}

/// Execute the first function in the module.
fn execute_wat(wat: &str, a: usize, b: usize) -> usize {
    let translated = translate_wat(wat);
    translated.execute_func(0, a, b)
}

#[test]
fn adds() {
    const CASES: &[(usize, usize, usize)] = &[
        (5, 3, 8),
        (0, 228, 228),
        (usize::max_value(), 1, 0),
    ];

    let code = r#"
(module
  (func (param i32) (param i32) (result i32) (i32.add (get_local 0) (get_local 1)))
)
    "#;
    for (a, b, expected) in CASES {
        assert_eq!(execute_wat(code, *a, *b), *expected);
    }
}

#[test]
fn relop_eq() {
    const CASES: &[(usize, usize, usize)] = &[
        (0, 0, 1),
        (0, 1, 0),
        (1, 0, 0),
        (1, 1, 1),
        (1312, 1, 0),
        (1312, 1312, 1),
    ];

    let code = r#"
(module
  (func (param i32) (param i32) (result i32) (i32.eq (get_local 0) (get_local 1)))
)
    "#;

    for (a, b, expected) in CASES {
        assert_eq!(execute_wat(code, *a, *b), *expected);
    }
}

#[test]
fn block() {
    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    (block (result i32)
        get_local 0
    )
  )
)
    "#;

    assert_eq!(execute_wat(code, 10, 20), 10);
}

#[test]
fn if_then_else() {
    const CASES: &[(usize, usize, usize)] = &[
        (0, 1, 1),
        (0, 0, 0),
        (1, 0, 0),
        (1, 1, 1),
        (1312, 1, 1),
        (1312, 1312, 1312),
    ];

    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    (if (result i32)
      (i32.eq
        (get_local 0)
        (get_local 1)
      )
      (then (get_local 0))
      (else (get_local 1))
    )
  )
)
    "#;

    for (a, b, expected) in CASES {
        assert_eq!(execute_wat(code, *a, *b), *expected, "{}, {}", a, b);
    }
}

#[test]
fn if_without_result() {
    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    (if
      (i32.eq
        (get_local 0)
        (get_local 1)
      )
      (then (unreachable))
    )

    (get_local 0)
  )
)
    "#;

    assert_eq!(execute_wat(code, 2, 3), 2);
}

#[test]
fn br_block() {
    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    get_local 1
    (block (result i32)
        get_local 0
        get_local 0
        br 0
        unreachable
    )
    i32.add
  )
)
    "#;

    assert_eq!(execute_wat(code, 5, 7), 12);
}

// Tests discarding values on the value stack, while
// carrying over the result using a conditional branch.
#[test]
fn brif_block() {
    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    get_local 1
    (block (result i32)
        get_local 0
        get_local 0
        br_if 0
        unreachable
    )
    i32.add
  )
)
    "#;

    assert_eq!(execute_wat(code, 5, 7), 12);
}

// Tests that br_if keeps values in the case if the branch
// hasn't been taken.
#[test]
fn brif_block_passthru() {
    let code = r#"
(module
  (func (param i32) (param i32) (result i32)
    (block (result i32)
        get_local 1
        get_local 0
        br_if 0
        get_local 1
        i32.add
    )
  )
)
    "#;

    assert_eq!(execute_wat(code, 0, 3), 6);
}

// TODO: Add a test that checks argument passing via the stack.
