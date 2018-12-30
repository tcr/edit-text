# Markdown

Let's assume we have the following document in edit-text's document structure:

```
DocGroup({"tag": "bullet"}, [
    DocGroup({"tag": "p"}, [DocText("Item 1")]),
])
DocGroup({"tag": "bullet"}, [
    DocGroup({"tag": "p"}, [DocText("Item 2")]),
])
DocGroup({"tag": "bullet"}, [
    DocGroup({"tag": "p"}, [DocText("Item 3...")]),
])
```

We can trivially define a mapping from edit-text's document model to HTML. (Conversion from a doc group to HTML can be done with `doc_as_html` in `edit-common/lib.rs`. There's no inverse method.) The result is an unordered list:

```
<ul>
  <li><p>Item 1</p></li>
  <li><p>Item 2</p></li>
  <li><p>Item 3...</p></li>
</ul>
```

Some conversions are straightforward: aside from all non-significant whitespace, all text nodes are converted into the DocText(...) struct. To simplify other logic, there are some invariants that should be true about DocText: DocText(...) must not be empty, and there must not be two successive DocText(...) components. This isn't validated anywhere (yet) but is expected to be true in all operations.

For groups, the first argument is a hashmap of `String` => `String` containing the "attributes". These are similar to HTML attributes and can contain any data. The one attribute required by all groups is the "tag" attribute, which usually maps to the name of its HTML equivalent.

NOTE: The use of "tag" or even any HTML semantics are not required by the operational transform library, Oatie. References to the "tag" attribute are almost entirely contained in `schema.rs`. In theory, every document/transform schema could use its own way of distinguishing between group kinds.

Of interest in the above conversion is that quasi-`<bullet>` tag has different semantics than its HTML counterpart, `<ul><li>...</li></ul>`. This is a deliberate simplification for operational transform (that I should detail elsewhere), but essentially our goal is to better represent Markdown's semantics, not HTML's. Take the following example:

```md
* Item 1
* Item 2

Interstitial paragraph

* Item 3...
```

And the middle paragraph, `Interstitial paragraph` is deleted. The document then becomes:

```md
* Item 1
* Item 2
* Item 3...
```

Because edit-text converts directly from its document representation into Markdown, we can bypass the logic of joining common `<ul>` parents in this case and also lean more heavily on Markdown-to-html conversion to perform this for us.

## Markdown serialization + deserialization

The module that controls markdown lives at `edit-common/src/markdown`.

* [ser.rs](https://github.com/tcr/edit-text/blob/master/edit-common/src/markdown/ser.rs)
* [de.rs](https://github.com/tcr/edit-text/blob/master/edit-common/src/markdown/de.rs)

edit-text's document schema should allow conversion losslessly into Markdown, while the deserialization code takes into account (or should) that Markdown's possible output is a superset of what edit-text supports, and thus all non-supported content should be stripped out.

## Document Elements

This are the current elements supported by edit-text:

| Tag | Description |
|-----|-------------|
| bullet | Bulleted item
| p | Paragraph
| h[1-6] | Header
| pre | Code block
| html | Inline HTML content (a raw string, as it would appear in Markdown)
| caret | Caret position
| hr | Horizontal rule
