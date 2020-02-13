use super::map::{Map, Position};

#[derive(Debug)]
pub enum TranslationUpdates {
    State(i32),
    EntityIdBlock(i32),
    XOrigin(i32),
    ZOrigin(i32),
    NoChange,
}

#[derive(Debug, Clone)]
pub struct TranslationInfo {
    pub state: i32,
    pub map: Map,
}

impl TranslationInfo {
    pub fn new() -> TranslationInfo {
        TranslationInfo {
            state: 0,
            map: Map::new(Position { x: 0, z: 0 }, 0),
        }
    }

    pub fn update(&mut self, param: &TranslationUpdates) {
        match param {
            TranslationUpdates::State(state) => {
                self.state = *state;
            }
            TranslationUpdates::EntityIdBlock(block) => {
                self.map.entity_id_block = *block;
            }
            TranslationUpdates::XOrigin(x) => {
                self.map.position.x = *x;
            }
            TranslationUpdates::ZOrigin(z) => {
                self.map.position.z = *z;
            }
            TranslationUpdates::NoChange => {}
        }
    }
}
