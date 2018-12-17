# Glossary

* *client* &mdash; A client can connect to a server and synchronize its document
  content. It sends client-side modifications (in the form of operations) to the
  server, and receives updated content (in the form of operations) from the server
  after any client submits an update.

* *controller* &mdash; Receives UI-level event updates from the frontend
  and converts it into operations on the client document.

* *cursor* &mdash; All positions in which a Text or Group element can be
  inserted into a document can be represented by a cursor object.

* *frontend* &mdash; The editor UI. The current document is rendered
  as a component inside the frontend, and interactions with this component are
  forwarded to the controller. The frontend also manages the toolbar,
  notifications, and dialog boxes.

* *server* &mdash; Serves HTTP content, a GraphQL endpoint for performing
  page-level commands, and a WebSocket endpoint for synchronizing document
  content.
