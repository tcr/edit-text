# Documents

The contents of a page is stored as a **Document** object. For example, the following "Hello world!" document:

<img width="1189" alt="image" src="https://user-images.githubusercontent.com/80639/50059231-1737f500-0152-11e9-8704-d133d6b19e66.png">

May be created and manipulated in Rust:

```rust,noplaypen
let document = doc![
    DocGroup({"tag": "h1"}, ["Hello world!"]),
    DocGroup({"tag": "p"}, ["This is a document."]),
];

// Convert this document to Markdown.
let markdown = edit_common::markdown::doc_to_markdown(&document)?;
```

## Text and Groups

Documents are composed of **text** and **groups**. Text is a unicode string modeled by the `DocString` type. Groups are similar to HTML elements, having a set of "attributes" and then a vector of children which can be either text or other groups.

In Rust, `enum DocElement` models these two types:

```rust,noplaypen
type DocSpan = Vec<DocElement>;

type Attrs = HashMap<String, String>;

enum DocElement {
    DocText(DocString),
    DocGroup(Attrs, DocSpan),
}
```

You can use the `oatie::macros::doc!` macro to conveniently create `DocSpan` objects:

```rust,noplaypen
doc![
    DocGroup({"tag": "h1"}, ["Title"]),
    DocGroup({"tag": "bullet"}, [
        DocGroup({"tag": "p"}, ["Groups can be nested."]),
        DocGroup({"tag": "p"}, ["With multiple elements."]),
    ]),
]
```

## Documents and Operations

Documents can [have operations performed on them](working-with-operations.html) to result in a new, modified document.

## Schema

Documents have no enforced structure other than being a collection of text and groups. To define which types in a document are valid and what they are allowed to contain, the `oatie::schema::RtfSchema` defines which groups can contain what other groups, which can contain text content, and which can be found in the root. See `oatie::validate::validate_doc_span` for how to validate a document tree.
