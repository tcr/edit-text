//! Sync state. This is a candidate file to be moved into Oatie.

use extern::{
    bus::{Bus},
    failure::Error,
    mercutio_common::{
        SyncToUserCommand,
    },
    oatie::{
        OT,
        doc::*,
        schema::RtfSchema,
        validate::validate_doc,
    },
    std::{
        collections::{HashMap, VecDeque},
    },
};

pub struct SyncState {
    pub version: usize,
    pub clients: HashMap<String, usize>, // client_id -> version
    pub history: HashMap<usize, Op>,
    pub doc: Doc,

    pub ops: VecDeque<(String, usize, Op)>,
    pub client_bus: Bus<SyncToUserCommand>,
}

impl SyncState {
    fn prune_history(&mut self) {
        if let Some(min_version) = self.clients.iter().map(|(_, &v)| v).min() {
            for k in self.history.keys().cloned().collect::<Vec<usize>>() {
                if k < min_version {
                    // eprintln!("(^) evicted document version {}", k);
                    self.history.remove(&k);
                }
            }
        }
    }

    /// Transform an operation incrementally against each interim document operation.
    pub fn update_operation_to_current(
        &self,
        mut op: Op,
        mut input_version: usize,
        target_version: usize,
    ) -> Result<Op, Error> {
        // Transform against each interim operation.
        while input_version < target_version {
            // If the version exists (it should) transform against it.
            let version_op = self.history.get(&input_version)
                .ok_or(format_err!("Version missing from history"))?;
            let (updated_op, _) = Op::transform::<RtfSchema>(version_op, &op);
            op = updated_op;

            input_version += 1;
        }
        Ok(op)
    }

    pub fn commit(
        &mut self,
        client_id: &str,
        op: Op,
        input_version: usize,
    ) -> Result<Op, Error> {
        let target_version = self.version;

        // Update the operation so we can apply it to the document.
        let op = self.update_operation_to_current(
            op,
            input_version,
            target_version,
        )?;

        if let Some(version) = self.clients.get_mut(client_id) {
            *version = target_version;
        } else {
            // TODO what circumstances would it be missing? Client closed
            // and removed itself from list but operation used later?
        }

        // Prune history entries.
        self.prune_history();
        self.history.insert(target_version, op.clone());

        // Update the document with this operation.
        let new_doc = Op::apply(&self.doc, &op);

        // Gut check.
        validate_doc(&self.doc).map_err(|_| format_err!("Validation error"))?;
        
        // Commit chhanges.
        self.doc = new_doc;
        self.version = target_version + 1;

        Ok(op)
    }
}