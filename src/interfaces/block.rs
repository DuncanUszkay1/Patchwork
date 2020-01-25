use std::sync::mpsc::Sender;
use uuid::Uuid;

define_interface!(BlockState, (Report, report, [conn_id: Uuid]));
