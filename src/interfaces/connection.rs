use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(ConnectionService, (Close, close, [conn_id: Uuid]));
