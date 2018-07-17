# Working with Operations

Operations are defined by a "deletion" and an "addition" pair. This covers the whole space of modifications that edit-text can perform, and the distinction allows operational transform logic to be simplified.

The `Op` type is a tuple of an `AddSpan` and a `DelSpan`, each composed of `AddElement` and `DelElement` enums respectively. These `???Element` enums have some obvious `DocElement` counterparts, like `DocChars`, `DelChars`, and `AddChars`.

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