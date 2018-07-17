# Documents and Markdown

A document in Mercutio is built with *groups* and *characters*. The document model that Mercutio uses is similar to HTML. We can trivially define a mapping from Mercutio's document model to HTML:

```
<ul>
  <li><p>Item 1</p></li>
  <li><p>Item 2</p></li>
  <li><p>Item 3...</p></li>
</ul>
```

Becomes when converted to Mercutio's document structure (expressed in RON):

```
DocGroup({"tag": "bullet", [
    DocGroup({"tag": "p"}, [DocChars("Item 1")]),
]})
DocGroup({"tag": "bullet", [
    DocGroup({"tag": "p"}, [DocChars("Item 2")]),
]})
DocGroup({"tag": "bullet", [
    DocGroup({"tag": "p"}, [DocChars("Item 3...")]),
]})
```

(Conversion from a doc group to HTML can be done with `doc_as_html` in `edit-common/lib.rs`. There's no inverse method.)

Some conversions are straightforward: aside from all non-significant whitespace, all text nodes are converted into the DocChars(...) struct. To simplify other logic, there are some invariants that should be true about DocChars: DocChars(...) must not be empty, and there must not be two successive DocChars(...) components. This isn't validated anywhere (yet) but is expected to be true in all operations.

For groups, the first argument is a hashmap of `String` => `String` containing the "attributes". These are similar to HTML attributes and can contain any data. The one attribute required by all groups is the "tag" attribute, which usually maps to the name of its HTML equivalent.

**O/T:** The use of "tag" or even any HTML semantics are not required by the operational transform library, Oatie. References to the "tag" attribute are almost entirely contained in `schema.rs`. In theory, every document/transform schema could use its own way of distinguishing between group kinds.

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

Because Mercutio converts directly from its document representation into Markdown, we can bypass the logic of joining common `<ul>` parents in this case and also lean more heavily on Markdown-to-html conversion to perform this for us.

## Markdown serialization + deserialization

The module that controls markdown lives at `edit-common/src/markdown`.

* [ser.rs](https://github.com/tcr/edit-text/blob/master/edit-common/src/markdown/ser.rs)
* [de.rs](https://github.com/tcr/edit-text/blob/master/edit-common/src/markdown/de.rs)

Mercutio's document schema should allow conversion losslessly into Markdown, while the deserialization code takes into account (or should) that Markdown's possible output is a superset of what Mercutio supports, and thus all non-supported content should be stripped out.

## Document Elements

This are the current elements supported by Mercutio:

```
bullet => Bulleted item
p => Paragraph
h1/h2/h3/h4/h5/h6 => Header
pre => Code block
html => Inline HTML content (a raw string, as it would appear in Markdown)
caret => Caret position
hr => Horizontal rule
```

# Splitting Image

The entire document editing process is built on splitting a single origin block element into a series of sub-elements.

All editing steps you can take in the frontend editor preserve this quality.