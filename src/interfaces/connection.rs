use std::sync::mpsc::Sender;
use uuid::Uuid;

pub trait ConnectionService {
    fn close(&self, conn_id: Uuid);
}

impl ConnectionService for Sender<ConnectionOperations> {
    fn close(&self, conn_id: Uuid) {
        self.send(ConnectionOperations::Close(conn_id)).unwrap();
    }
}

#[derive(Debug, Clone)]
pub enum ConnectionOperations {
    Close(Uuid),
}
