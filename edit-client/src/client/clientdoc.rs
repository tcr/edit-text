//! Document + versioning state that talks to a synchronization server.

use oatie::doc::*;
use oatie::rtf::RtfSchema;
use oatie::validate::validate_doc;
use oatie::OT;
use std::mem;

#[derive(Debug)]
pub struct ClientDoc {
    pub doc: Doc<RtfSchema>,
    pub version: usize,
    pub client_id: String,

    pub original_doc: Doc<RtfSchema>,
    pub pending_op: Option<Op<RtfSchema>>,
    pub local_op: Op<RtfSchema>,
}

impl ClientDoc {
    // Default
    pub fn new(client_id: String) -> ClientDoc {
        ClientDoc {
            doc: Doc(vec![]),
            version: 100,
            client_id,

            original_doc: Doc(vec![]),
            pending_op: None,
            local_op: Op::empty(),
        }
    }

    /// Overwrite current state
    pub fn init(&mut self, new_doc: &Doc<RtfSchema>, version: usize) {
        self.doc = new_doc.clone();
        self.version = version;

        self.original_doc = new_doc.clone();
        self.pending_op = None;
        self.local_op = Op::empty();
    }

    /// Sync ACK'd our pending operation.
    /// Returns the next op to send to sync, if any.
    // TODO we can determine new_doc without needing it passed in
    pub fn sync_confirmed_pending_op(
        &mut self,
        new_doc: &Doc<RtfSchema>,
        version: usize,
    ) -> Option<Op<RtfSchema>> {
        log_wasm!(SyncNew("confirmed_pending_op".into()));

        // Server can't acknowledge an operation that wasn't pending.
        assert!(self.pending_op.is_some());
        // Likewise, the new doc update should be equivalent to original_doc : pending_op
        // or otherwise server acknowledged something improper.
        println!("Sync confirmed our pending op.");
        println!("pending_op: {:?}", self.pending_op);
        assert_eq!(
            new_doc,
            &Op::apply(&self.original_doc, self.pending_op.as_ref().unwrap()),
            "invalid ack from Sync"
        );

        self.original_doc = new_doc.clone();

        // Reassemble local document.
        self.doc = Op::apply(new_doc, &self.local_op);
        self.version = version;

        validate_doc(&self.doc).expect("Validation error after pending op");

        // Now that we have an ack, we can send up our new ops.
        self.pending_op = None;
        self.next_payload()
    }

    /// Sync gave us an operation not originating from us.
    // TODO we can determine new_doc without needing it passed in
    pub fn sync_sent_new_version(
        &mut self,
        new_doc: &Doc<RtfSchema>,
        version: usize,
        input_op: &Op<RtfSchema>,
    ) -> (Doc<RtfSchema>, Op<RtfSchema>) {
        // log_wasm!(SyncNew("new_op".into()));
        self.assert_compose_correctness(None);

        let current_doc = self.doc.clone();

        // Optimization
        if self.pending_op.is_none() && self.local_op == Op::empty() {
            // Skip ahead
            self.doc = new_doc.clone();
            self.version = version;
            self.original_doc = new_doc.clone();
            return (current_doc, Op::empty());
        }

        println!("\n----> TRANSFORMING");

        // Extract the pending op.
        assert!(self.pending_op.is_some());
        let pending_op = self.pending_op.clone().unwrap();

        // Extract and compose all local ops.
        let local_op = self.local_op.clone();

        // Transform.
        println!();
        println!("<test>");
        println!("server: {:?}", input_op);
        println!();
        println!("pending: {:?}", pending_op);
        println!("client: {:?}", local_op);
        println!("</test>");
        println!();

        // I x P -> I', P'
        let (pending_transform, input_transform) = Op::transform(&input_op, &pending_op);

        // let pending_final = Op::compose(&pending_transform, &correction);
        // let input_final = Op::compose(&input_transform, &correction);

        // P' x L -> P'', L'
        let (local_transform, _) = Op::transform(&input_transform, &local_op);

        // let correction = correct_op(&local_transform).unwrap();
        // let input_correction = correct_op(&input_transform).unwrap();
        // let correction_transform = Op::transform_advance::<RtfSchema>(&local_correction, &input_correction);
        // let correction = Op::compose(&local_correction, &correction_transform);

        // let local_final = Op::compose(&local_transform, &correction);
        // Drop input_final

        // client_doc = input_doc : I' : P''
        // let client_op = Op::compose(&pending_op_transform, &local_op_transform);

        // Do each operation in order, because we are going to apply corrections
        // to each new doc.

        println!();
        println!("<test>");
        println!("pending_op: {:?}", pending_op);
        println!();
        println!("local_op: {:?}", local_op);
        println!();
        println!("input_op: {:?}", input_op);
        println!();
        println!("new_doc: {:?}", new_doc);
        println!();
        println!("pending_op_transform: {:?}", pending_transform);
        println!();
        println!(
            "new_doc_pending: {:?}",
            Op::apply(&new_doc, &pending_transform)
        );
        println!();
        println!("local_op_transform: {:?}", local_transform);
        println!();
        println!("doc: {:?}", self.doc);
        println!("</test>");
        println!();

        // Reattach to doc.
        self.doc = Op::apply(&new_doc, &pending_transform);
        validate_doc(&self.doc).expect("Validation error after pending_op transform");
        self.doc = Op::apply(&self.doc, &local_transform);
        validate_doc(&self.doc).expect("Validation error after local_op transform");

        // {
        // let mirror = Op::apply(&new_doc, &Op::compose(&pending_op_transform, &local_op_transform));
        // assert_eq!(self.doc, mirror);
        // }

        // Set pending and local ops.
        self.pending_op = Some(pending_transform);
        if self.local_op != Op::empty() {
            self.local_op = local_transform;
        }

        // Update other variables.
        self.version = version;
        self.original_doc = new_doc.clone();

        // println!("{}", format!("\n----> result {:?}\n{:?}\n{:?}\n\n{:?}\n\n", self.original_doc, self.pending_op, self.local_op, self.doc).red());

        self.assert_compose_correctness(None);

        (current_doc, input_transform)
    }

    /// When there are no payloads queued, queue a next one.
    pub fn next_payload(&mut self) -> Option<Op<RtfSchema>> {
        log_wasm!(Debug(format!("NEXT_PAYLOAD: {:?}", self.local_op)));
        if self.pending_op.is_none() && self.local_op != Op::empty() {
            // Take the contents of local_op.
            self.pending_op = Some(mem::replace(&mut self.local_op, Op::empty()));
            println!("~~~~~~~> {:?} \n {:?}\n\n", self.pending_op, self.local_op);
            self.pending_op.clone()
        } else {
            None
        }
    }

    #[allow(unused)]
    fn assert_compose_correctness(&self, op: Option<Op<RtfSchema>>) {
        // Reference for variable names:
        // self.original_doc + pending_op + local_op + op
        //                              ^ recreated_doc
        //                                         ^ recreated_doc2 (self.doc)
        //                                              ^ target_doc
        //                                  (combined_op)

        if cfg!(debug_assertions) {
            //              println!("---->
            // <apply_local_op>
            // original_doc={:?},
            // pending_op={:?},
            // local_op={:?},
            // {op}</apply_local_op>
            // ",
            //             self.original_doc,
            //             self.pending_op,
            //             self.local_op,
            //             op = op.as_ref().map(|x| format!("op = {:?},\n", x)).unwrap_or("".to_string()),
            //         );

            // Test matching against the local doc.
            let recreated_doc = Op::apply(
                &self.original_doc,
                self.pending_op.as_ref().unwrap_or(&Op::empty()),
            );
            // println!("\n\nrecreated_doc={:?}", recreated_doc);
            let recreated_doc2 = Op::apply(&recreated_doc, &self.local_op);
            // println!("\n\nrecreated_doc2={:?}", recreated_doc2);
            assert_eq!(self.doc, recreated_doc2);
            if let &Some(ref op) = &op {
                let target_doc = Op::apply(&self.doc, op);
                // println!("\n\ntarget_doc={:?}", target_doc);
            }

            if let &Some(ref op) = &op {
                let combined_op = Op::compose(&self.local_op, op);
                // println!("\n\ncombined_op={:?}", combined_op);
                let target_doc2 = Op::apply(&recreated_doc, &combined_op);
                // println!("\n\ntarget_doc2={:?}", target_doc2);
            }

            let total_op = Op::compose(
                self.pending_op.as_ref().unwrap_or(&Op::empty()),
                &self.local_op,
            );
            let recreated_doc = Op::apply(&self.original_doc, &total_op);
            assert_eq!(self.doc, recreated_doc);
        }
    }

    /// An operation was applied to the document locally.
    pub fn apply_local_op(&mut self, op: &Op<RtfSchema>) {
        self.assert_compose_correctness(Some(op.clone()));

        // TODO pending op should be none, but it's actually a value here.
        // why? check for when original_Doc gets modified (i ugess) and then
        // see when self.pending_op gts nulled out.

        use oatie::validate::*;
        validate_doc(&self.doc).expect("Validation error BEFORE op application");

        // Apply the new operation.
        self.doc = Op::apply(&self.doc, op);

        // TODO Generate an "undo" version of the operation and store it.
        // This should come from the Op::apply above.

        // Combine operation with previous queued operations.
        self.local_op = Op::compose(&self.local_op, &op);

        self.assert_compose_correctness(None);
    }
}
