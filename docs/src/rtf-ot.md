# Operational Transform

When two clients A and B make an operation concurrently, one way to get them back in sync is to determine what would operation B look like if operation A had happened first, and vice versa. If we can guarantee that this result on both clients produce the same output, meaning if we can guarantee the following (where ":" means "composed with"):

```
 operation A : (operation B as if A had happened first)

        the above composition is equivalent to

 operation B : (operation A as if B had happened first)
```

Then we can guarantee both clients, which had different operations occur to their documents before this moment in time, can get back in sync. The algorithm used by `oatie` guarantees this operation will be commutative, which is makes other properties of this system simpler.

## Rules for Transform

Operational transform is commutative, which is to say that a result of transforming A × B should result in A' and B', and the property A composed with A' == B composed with B'. Or put another way, the transform function when given two operations will return two subsequent operations to bring both clients into an identical document state.

Starting with a simple example, what happens when two documents type the same character simultaneously:

```
doc:       ed

          bed
client A: ^

          red
client B: ^
```

We can determine a hueristic, say, that when transforming we always know client A goes first and B second, and come up with the transformed operations:

```
           bred
client A':  ^

           bred
client B': ^
```

Now both clients are synchronized again, and we can repeat this at any time as long as we use a stable client ordering. If we look at deleting characters, we actually discover we don't even need an order:

```
doc:      creditor

            editor
client A: XX

          credit
client B:       XX

new doc:   edit
```

Deletions, as we'll call them, are commutative, and the result of transforming them should update both clients to have deleted their union. This holds true in even more complex scenarios.

A harder consideration is when we introduce groups, which are similar to HTML elements. `oatie` doesn't operate on HTML tags, but acts more like the DOM: groups are strictly nested and can only contain text or other groups. Each group has a "tag" (like an HTML tag), but also can contain other attributes (for example, each user's cursor contains the attribute of its originating client).

Transforming two operations which operate on groups in distinct ways offers some difficulty.

## Transform In Depth

**NOTE: The remainder of this document is not necessary for developing on edit-text.
If you're interested in more about how the transform step works, read on.**

### Race Condition

Let's look at a race condition. Client A sends an operation to the server, 

Next, we need to work around the undesirable constraint each operation made by a client has to be transformed against another client's. We actually can generalize up to more than two clients—by transforming the results of the operation A x B with the results of B' &mul; C, etc. But in practice, we are not going to have each operation have a corresponding and concurrent operation on each client at the same instant. Instead, we version the document, and use this to tell what operations an incoming operation should be transformed against. For instance, if operations A and B happen simultaneously, the server can do the following:

1. Start with doc version 100.
1. Apply operation A. The doc version is now 101.
2. See that operation B says its version was set to 100. We need to bring it up to date. We store a history of all previous operations, and so we transform operation B &mul; the operation to transition from version 100 &rarr; 101. The result is operation B as though it operated on version 101, which we can directly apply to our document and send out to all clients to perform.

Client A becomes in sync easily:

1. Start with doc version 100.
2. Apply operation A. Also send operation A to the server.
3. Receive an ACK from the server, and learn our version is now 101.
4. Receive operation B, which transitions from version 101 &rarr; 102; our document is now at version 102.

For client B, this is somewhat tricker; we already applied our operation! So we perform a transform locally against *our* history.

1. Start with doc verison 100.
2. Apply operation B. Also send operation B to the server. The network queue is synchronous; it only sends one operation at a time, waiting until an ACK to send the next.
3. Receive operation A, which transitions from version 100 &rarr; 101; we didn't expect this. So we transform this incoming operation A &mul; operation B, for which an ACK from the server is outstanding. We result in A' (operation A if operation B had happened already) which we apply to the document. We also result in B', which we transform against to operations which have accumulated behind operation B in the network queue.
4. Receive an ACK from the server, and learn our version is now 102. Note that at this point the server state and the state of our local client are equivalent (A &mul; A' == B &mul; B').
5. Because we received an ACK, we can send the next operation in our network queue (if any).

Note that the strategies for server and client code to bring operations "up to date" differ in that the client only needs to transform against one operation, while the server needs to transform against all intervening operations in its history. The server only needs to store the history up to the client that is the most out of date, however, and can always boot off clients which are off too old a version.

### Split Block Model

The entire document editing process is built on splitting a single origin block
element into a series of sub-elements.

All editing steps you can take in the frontend editor should follow this constaint.
