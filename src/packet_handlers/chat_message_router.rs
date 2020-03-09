use super::interfaces::patchwork::PatchworkState;
use super::packet::Packet;
use super::Peer;
use uuid::Uuid;

pub fn route_packet<PA: PatchworkState>(p: Packet, conn_id: Uuid, patchwork_state: PA) {
    match p {
        Packet::ChatMessage(chat_message) => {
            trace!(
                "Received chat message {:?} from {:?}",
                chat_message.message,
                conn_id
            );
            let mut token_stream = chat_message.message.as_str().split(' ');
            match token_stream.next() {
                Some("/connect") => {
                    connect(&mut token_stream, patchwork_state).ok();
                }
                _ => {}
            }
        }
        _ => {
            panic!("Chat Message Router received unexpected packet {:?}", p);
        }
    }
}

fn connect<'a, I: Iterator<Item = &'a str>, PA: PatchworkState>(
    token_stream: &mut I,
    patchwork_state: PA,
) -> Result<(), ()> {
    let addr = token_stream.next().ok_or(())?;
    let port_str = token_stream.next().ok_or(())?;
    let port = port_str.parse::<u16>().map_err(|_| ())?;

    patchwork_state.new_map(Peer {
        port,
        address: String::from(addr),
    });

    Ok(())
}
