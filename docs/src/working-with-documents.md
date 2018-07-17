# Working with Documents

The basic structure that Oatie operates on is a Document:

```rust
type DocSpan = Vec<DocElement>;

struct Doc(DocSpan);

// Documents are made up of chars and groups.
enum DocElement {
    DocChars(DocString),
    DocGroup(Attrs, DocSpan),
}

// Convenience wrapper for opaquely operating on Strings.
struct DocString(String);
// Convenience wrapper for group "attributes" like HTML attrs.
type Attrs = HashMap<String, String>;
```

Documents can [have operations performed on them](working-with-operations.html) to result in a new, modified document.
