# Delall Hack

Grepping the codebase for "Delall" turns up this comment in `transform.rs`:

```
// "Delall" transform hack to avoid fully deleted elements that
// leave their content unwrapped. Because one side deletes the
// group, we can't recreate it (because we have no knowledge of
// the document). Instead, we just delete all the newly added
// content of the group.
```

This is an explainer on what this means.

## Operations only know the document structure implied by the operation

You do a DelGroup on a Group, but when transformed against any of these elements, we can respond consistently even without knowing the other type of document.

TODO: This section

* DelGroup x DelText
* DelGroup x DelWithGroup
* DelGroup x DelGroup

...

## Example Transform

**NOTE:** This is a long, but realistic, example. Imagine every client connected that has a ton of lag, accuring operations until it waits for a server response.

When transforming two operations, oatie (so far) does not need to know anything about the document the operation will eventually operate on in order to compose or transform two operations. In order to maintain this property, some pedantic cases arise where we need to think carefully what the result of a transform will be.

Say two clients have the following document:

```rust,noplaypen
<h1><caret client=B" />Hello </h1>
<p><caret client="A" /> world!</p>
```

### The journey of Client A

Client A hits backspace, which will collapse the two block-level elements (`h1` and `p`) into one. The document there will now look like this:

```rust,noplaypen
// This operation:
Op([
    DelGroup([DelSkip(7)]), DelGroup([DelSkip(8)]),
], [
    AddGroup({"tag": "h1"}, [AddSkip(15)]),
])

// New Document
<h1><caret client=B" />Hello <caret client="A" /> world!</p>
```

Next, Client A forward-deletes everything that existed in the second paragraph (`" world!"`). So the document now looks like this:

```rust,noplaypen
// This operation:
Op([
    DelWithGroup([DelSkip(8), DelText(7)]),
], [
])

// Composed operations:
Op([
    DelGroup([DelSkip(7)]), DelGroup([DelSkip(1), DelText(7)]),
], [
    AddGroup({"tag": "h1"}, [AddSkip(15)]),
])

// New document
<h1><caret client=B" />Hello <caret client="A" /></p>
```

Lastly (for good measure) Client A clicks at the beginning of the first paragraph to move its caret:

```rust,noplaypen
// This operation:
Op([
    DelWithGroup([DelSkip(7), DelGroup([])]),
], [
    AddWithGroup([AddGroup({"tag": "caret", "client": "A"})])
])

// Composed operations:
Op([
    DelGroup([DelSkip(7)]), DelGroup([DelGroup([]), DelText(7)]),
], [
    AddGroup({"tag": "h1"}, [AddGroup({"tag": "caret", "client": "A"}), AddSkip(14)]),
])

// New document
<h1><caret client="A" /><caret client=B" />Hello </p>
```

### The journey of Client B

Client B is less active, has less lag, or just less to contribute. It simply moves its cursor to the second block...:


```rust,noplaypen
// This operation:
Op([
    DelGroup([DelGroup([]), DelSkip(7)]),
], [
    AddSkip(1), AddWithGroup([AddSkip(8), AddGroup({"tag": "caret", "client": "B"}, [])]),
])

// New Document
<h1>Hello </h1>
<p><caret client="A" /> world!<caret client=B" /></p>
```

Then Client B hits backspace:

```rust,noplaypen
// This operation:
Op([
    DelSkip(1), DelWithGroup([DelSkip(7), DelText(1)]),
], [
])

// Cumulative operation:
Op([
    DelGroup([DelGroup([]), DelSkip(7)]), DelWithGroup([DelSkip(7), DelText(1)]),
], [
    AddSkip(1), AddWithGroup([AddSkip(8), AddGroup({"tag": "caret", "client": "B"}, [])]),
])

// New Document
<h1>Hello </h1>
<p><caret client="A" /> world<caret client=B" /></p>
```

### Transform

In the end of this hypothetical, we are now transforming these two operations:

```rust,noplaypen
// Client A
Op([
    DelGroup([DelSkip(7)]), DelGroup([DelGroup([]), DelText(7)]),
], [
    AddGroup({"tag": "h1"}, [AddGroup({"tag": "caret", "client": "A"}), AddSkip(14)]),
])

// Client B
Op([
    DelGroup([DelGroup([]), DelSkip(7)]), DelWithGroup([DelSkip(7), DelText(1)]),
], [
    AddSkip(1), AddWithGroup([AddSkip(8), AddGroup({"tag": "caret", "client": "B"}, [])]),
])
```

And we know exactly what each result is generated from applying each operation to our original document (first shown at the beginning of this document):

```rust,noplaypen
// Original document
<h1><caret client=B" />Hello </h1>
<p><caret client="A" /> world!</p>


// Client A
<h1><caret client="A" /><caret client=B" />Hello </p>

// Client B
<h1>Hello </h1>
<p><caret client="A" /> world<caret client=B" /></p>
```

But when transforming, by design, we avoid needing knowledge of what the document looks like. So let's evaluate these operations as though we didn't know what the result was going to look like. 

In particular, we want to look at how the paragraph, `" world!"`, is modified. Client A has deleted it entirely, whereas Client B deleted a character and inserted its caret:

```
Op([DelGroup([DelGroup([]), DelText(7)])], [])
Op([DelWithGroup([DelSkip(7), DelText(1)])], [AddWithGroup([AddSkip(8), AddGroup({"tag": "caret", "client": "B"}, [])])])
```

We can take the union of the deletions and since there is only one addition component, select it. When we transform the two, our result looks like this:

```
Op([DelGroup([DelGroup([]), DelText(7)])], [AddGroup({"tag": "caret", "client": "B"}, [])])
```

If you imagine a document consisting of only this element, and we see that one client has deleted the entire element, we accidentally wind up with a client's caret being in the root element (instead of block element). See [Splitting image](./diary-markdown.md].

To avoid this problem, the Delall hack delets alls newly inserted elements from an insertion that is transformed against a deletion that delets all content. We can verify any arbitrary `DelGroup` delets all of its inner contents using a recursive check.
