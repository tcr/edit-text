# Operations

Operations are sequences of steps that when applied to a document, modifies the document by adding or removing content. Operations are
broken down into two stages: deleting content, then adding content. This pair of "delete" and "add" steps makes up an operation, which is performed atomically on a document, ensuring that the result is always valid.

Operations can be "applied" to a document, producing a new document:

```rust,noplaypen
let updated_doc = Op::apply(&doc, &operation);
```

When applying multiple operation pairs in a row, you can also "compose" them to
produce a single operation that would produce the same result:

```rust,noplaypen
let updated_doc_1 = Op::apply(&doc, &operation_1);
let updated_doc_2 = Op::apply(&updated_doc_1, &operation_2);

// Same result
let updated_doc_2 = Op::apply(
    &doc,
    &Op::compose(&operation_1, &operation_2), // Composed operation
);
```

You can also transform two concurrent operation; see the section on [operational transform](rtf-ot.html).


## Example: Combining Two Groups

Let's start with a simple document composed of a title and a paragraph. The document
looks like the following in Markdown:

```markdown
# Title

Body
```

And would have the following document representation:

```rust,noplaypen
let doc = doc_span![
    DocGroup({"tag": "h1"}, [DocText("Title")]),
    DocGroup({"tag": "p"}, [DocText("Body")]),
];
```

Imagine the user starts editing. The user places their caret to the left of the word "Body", then hits backspace. What will happen is that the paragraph containing "Body" and the header containing the text "Title" will become combined into one header with the text "TitleBody".

There are a few ways to can imagine combining these two elements. The way in which edit-text implements this is by first deleting
the current and preceding blocks while preserving their content, then creating a new element
which spans the content of both blocks.

First, we would delete each block group:

```rust,noplaypen
let deletion = del_span![
    DelGroup([DelSkip(5)]), // Five characters in "Title"
    DelGroup([DelSkip(4)]), // Four characters in "Body"
];

assert_eq!(
    Op::apply_deletion(&doc, &deletion),
    doc_span![DocText("TitleBody")],
);
```

The result is just the characters "TitleBody". In edit-text, you are not allowed to have top-level textual content that is not contained inside of a group. So in order to produce a valid document, we now have to wrap the contents of both groups inside of a new group:

```rust,noplaypen
let addition = del_span![
    AddGroup({"tag": "h1"}, [
        AddSkip(9) // Nine characters in "Titlebody"
    ])
];

assert_eq!(
    Op::apply_addition(&Op::apply_deletion(&doc, &deletion), &addition),
    doc_span![
        DocGroup({"tag": "h1"}, [DocText("TitleBody")]),
    ];
);
```

A deletion followed by an addition is common enough in edit-text that you can work with it as a single datatype. The method `Op::apply` takes a document and a `&(DelSpan, AddSpan)` type, and returns a modified document.

```rust,noplaypen
type Op = (DelSpan, AddSpan);

let valid_op: Op = (
    del_span![
        DelGroup([DelSkip(5)]),
        DelGroup([DelSkip(4)]),
    ],
    add_span![
        AddGroup({"tag": "h1"}, [AddSkip(9)]),
    ],
);

assert_eq!(
    Op::apply(&doc, &valid_op),
    doc_span![
        DocGroup({"tag": "h1"}, [DocText("TitleBody")]),
    ];
);
```

## Constaints

Applying operations can fail. Usually, an operation is written to modify a known
document state, and applying that operation to the document it was intended for
should never fail.

NOTE: At the moment, if your apply an operation improperly to document and it
fails, it will likely panic!() rather than returning an Error object.

## Deletion and Addition Elements

These are all the steps a Deletion or Addition can perform.

```rust,noplaypen
enum DelElement {
    /// ...
    DelSkip(usize),
    DelWithGroup(DelSpan),
    DelText(usize),
    DelGroup(DelSpan),
}
```

```rust,noplaypen
enum AddElement {
    AddSkip(usize),
    AddWithGroup(AddSpan),
    AddText(DocString),
    AddGroup(Attrs, AddSpan),
}
```
