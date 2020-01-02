use std::sync::mpsc::Sender;
use uuid::Uuid;

pub trait BlockState {
    fn report(&self, conn_id: Uuid);
}

impl BlockState for Sender<BlockStateOperations> {
    fn report(&self, conn_id: Uuid) {
        self.send(BlockStateOperations::Report(ReportMessage { conn_id }))
            .unwrap();
    }
}

pub enum BlockStateOperations {
    Report(ReportMessage),
}

#[derive(Debug)]
pub struct ReportMessage {
    pub conn_id: Uuid,
}
