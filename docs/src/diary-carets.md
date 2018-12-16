# Carets

Carets are the vertical line which indicates your editing position in a document. When there are multiple viewers of a document, each viewer's current editing position would be represented by different carets. You can click the mouse to reposition the caret in a new location, or use various OS-specific shortcuts to move the caret by increments (like by word or line).

When a document is loaded, the client inserts two carets for the user: an "anchor" caret, and a "focus" caret. Usually these carets are located at the same position. When the anchor and focus carets are one more caret positions apart, the editor treats this as a selection. For example, when you click and drag your mouse to create a selection, the anchor caret stays in place and the focus caret follows your mouse.

Carets in edit-text are stored as part of the document itself. The data structure for carets looks like the following:

```
enum Attrs {
    Caret {
        client_id: String,
        focus: boolean,
    }
    ...
}
```

The `client_id` property represents the client for whom the caret belongs. (To see your client ID, click the debug menu in the editor.) The `focus` property is `false` if this is the anchor caret, and is `true` if this is the focus caret.

Carets are represented as inline elements in HTML, and must exist only inside a block-level element.

All editing operations that delete carets must also recreate the carets (possibly in a new location) to preserve there being one anchor and one focus caret for each client at all times. 

## Cursors

A related concept in edit-text is a cursor, represented by the `CurSpan` type, which specifies a single discrete position inside of the document (`DocSpan` type). When a user clicks their mouse, this is translated to a specific cursor inside the document. This can then be converted into a position to insert a caret (see the `cur_to_caret` function). A caret can have fewer valid positions than there are cursor positions, so this conversion is lossy.
