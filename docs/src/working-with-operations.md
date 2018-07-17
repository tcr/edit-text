# Working with Operations

Operations in edit-text are a sequence of a "deletion" followed by an "addition", grouped together in a pair. All modifications edit-text can perform are designed to perform a deletion followed by an addition, and this distinction between deleting and adding content allows operational transform logic to be simplified.

The `Op` type is a tuple of the `AddSpan` and `DelSpan` types, which just represent vectors of the `AddElement` and `DelElement` enums. The `DocElement` enums has some obvious counterparts here, like the trio of `DocChars`, `DelChars`, and `AddChars`.

```rust
type DelSpan = Vec<DelElement>;

enum DelElement {
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelChars(usize),
    DelGroup(DelSpan),
}
```

```rust
type AddSpan = Vec<AddElement>;

enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    AddChars(DocString),
    AddGroup(Attrs, AddSpan),
}
```

```rust
type Op = (DelSpan, AddSpan);
```

## Macros

When importing oatie, there are several convenience macros which make writing the above easier:

```rust
#[macro_use]
extern crate oatie;

let doc = doc_span![
    DocGroup({"tag": "p"}, [
        DocChars("Hello world!")
    ]),
];

let op = op_span!([
    DelGroup([DelSkip(12)]),
    AddGroup({"tag": "h1"}, [AddSkip(12)]),
]);
```

This is roughly equivalent to Rust code without needing to type `vec!` or use explicit `HashMap::new()` and `DocString(...)` invocations.

## Using Operations

An operation can be applied to a document.

```rust
#[macro_use]
extern crate oatie;

use oatie::doc::*;
use oatie::OT;

let doc = doc_span![
    DocGroup({"tag": "p"}, [DocChars("Hello world!")]),
];

let op = op_span!([
    DelGroup([DelSkip(12)]),
    AddGroup({"tag": "h1"}, [AddSkip(12)]),
]);

let doc2 = Op::apply(&doc, $op);

println!("{:?}", doc2);

// DocGroup({"tag": "h1"}, [DocChars("Hello world!")])
```