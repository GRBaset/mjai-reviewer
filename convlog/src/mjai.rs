use crate::Tile;

use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none};

/// Describes an event in mjai format.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum Event {
    None,

    StartGame {
        names: [String; 4],

        // akochan specific
        kyoku_first: u8,
        aka_flag: bool,
    },
    StartKyoku {
        bakaze: Tile,
        dora_marker: Tile,
        kyoku: u8, // counts from 1
        honba: u8,
        kyotaku: u8,
        oya: u8,
        scores: [i32; 4],
        tehais: [[Tile; 13]; 4],
    },

    Tsumo {
        actor: u8,
        pai: Tile,
    },
    Dahai {
        actor: u8,
        pai: Tile,
        tsumogiri: bool,
    },

    Chi {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Pon {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 2],
    },
    Daiminkan {
        actor: u8,
        target: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Kakan {
        actor: u8,
        pai: Tile,
        consumed: [Tile; 3],
    },
    Ankan {
        actor: u8,
        consumed: [Tile; 4],
    },
    Dora {
        dora_marker: Tile,
    },

    Reach {
        actor: u8,
    },
    ReachAccepted {
        actor: u8,
    },

    Hora {
        actor: u8,
        target: u8,

        deltas: Option<[i32; 4]>,
        ura_markers: Option<Vec<Tile>>,
    },
    Ryukyoku {
        deltas: Option<[i32; 4]>,
    },

    EndKyoku,
    EndGame,
}

impl Event {
    pub fn exchange_bakaze(self, kyoku_bakaze: Tile) -> Self {
        match self {
            Event::StartKyoku {
                bakaze,
                dora_marker,
                kyoku,
                honba,
                kyotaku,
                oya,
                scores,
                tehais,
            } => {
                let new_tehais = tehais.map(|tehai| tehai.map(|pai| pai.exchange_bakaze(bakaze)));

                Event::StartKyoku {
                    bakaze: bakaze.next(),
                    dora_marker,
                    kyoku,
                    honba,
                    kyotaku,
                    oya,
                    scores,
                    tehais: new_tehais,
                }
            }

            Event::Tsumo { actor, pai } => Event::Tsumo {
                actor,
                pai: pai.exchange_bakaze(kyoku_bakaze),
            },

            Event::Dahai {
                actor,
                pai,
                tsumogiri,
            } => Event::Dahai {
                actor,
                pai: pai.exchange_bakaze(kyoku_bakaze),
                tsumogiri,
            },

            Event::Chi {
                actor,
                target,
                pai,
                consumed,
            } => Event::Chi {
                actor,
                target,
                pai: pai.exchange_bakaze(kyoku_bakaze),
                consumed: consumed.map(|pai| pai.exchange_bakaze(kyoku_bakaze)),
            },

            Event::Pon {
                actor,
                target,
                pai,
                consumed,
            } => Event::Pon {
                actor,
                target,
                pai: pai.exchange_bakaze(kyoku_bakaze),
                consumed: consumed.map(|pai| pai.exchange_bakaze(kyoku_bakaze)),
            },

            Event::Daiminkan {
                actor,
                target,
                pai,
                consumed,
            } => Event::Daiminkan {
                actor,
                target,
                pai: pai.exchange_bakaze(kyoku_bakaze),
                consumed: consumed.map(|pai| pai.exchange_bakaze(kyoku_bakaze)),
            },

            Event::Kakan {
                actor,
                pai,
                consumed,
            } => Event::Kakan {
                actor,
                pai: pai.exchange_bakaze(kyoku_bakaze),
                consumed: consumed.map(|pai| pai.exchange_bakaze(kyoku_bakaze)),
            },

            Event::Ankan { actor, consumed } => Event::Ankan {
                actor,
                consumed: consumed.map(|pai| pai.exchange_bakaze(kyoku_bakaze)),
            },

            Event::Dora { dora_marker } => Event::Dora {
                dora_marker: dora_marker.exchange_bakaze(kyoku_bakaze),
            },

            Event::Hora {
                actor,
                target,
                deltas,
                ura_markers,
            } => Event::Hora {
                actor,
                target,
                deltas,
                ura_markers: ura_markers.map(|markers| {
                    markers
                        .iter()
                        .map(|pai| pai.exchange_bakaze(kyoku_bakaze))
                        .collect()
                }),
            },

            _ => self,
        }
    }

    #[inline]
    #[must_use]
    pub const fn actor(&self) -> Option<u8> {
        match *self {
            Self::Tsumo { actor, .. }
            | Self::Dahai { actor, .. }
            | Self::Chi { actor, .. }
            | Self::Pon { actor, .. }
            | Self::Daiminkan { actor, .. }
            | Self::Kakan { actor, .. }
            | Self::Ankan { actor, .. }
            | Self::Reach { actor, .. }
            | Self::ReachAccepted { actor, .. }
            | Self::Hora { actor, .. } => Some(actor),
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn naki_info(&self) -> Option<(u8, Tile)> {
        match *self {
            Self::Chi { target, pai, .. }
            | Self::Pon { target, pai, .. }
            | Self::Daiminkan { target, pai, .. } => Some((target, pai)),
            _ => None,
        }
    }

    #[inline]
    pub(crate) const fn naki_to_ord(&self) -> i8 {
        match *self {
            Self::Chi { .. } => 0,
            Self::Pon { .. } => 1,
            _ => -1,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn optional_field_deser() {
        let a = r#"{"type":"hora","actor":0,"target":0}"#;
        serde_json::from_str::<Event>(a).unwrap();
    }
}
