# Editor Basics

This section describes the basics of how editors are stored and modified.
The content of the editor is a "document" composed of text and
groups (which can contain other text or groups). Operations are data types that
can modify the document by adding or deleting content. These operations are
composable, and also can occur concurrently—conflicts are resolved with the
operational transform algorithm.

{{#toc}}
