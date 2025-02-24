use std::num::ParseIntError;

use lazy_static::lazy_static;

use crate::{
    t,
    tenhou::xml_scheme::{Agari, Dora, Go, Init, Meld, MeldEvent, Reach, Ryuukyoku, TileList, Un},
    Event, Tile,
};

use quick_xml::{de::Deserializer, events::Event as XMLEvent, Reader};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConvertError {
    #[error("Type {0:?} not implemented")]
    TypeNotImplemented(u8),
    #[error("XML parsing error: {0:?}")]
    XMLParseError(quick_xml::Error),
    #[error("XML deserializing error: {0:?}")]
    XMLDeError(quick_xml::DeError),
    #[error("Regex error: {0:?}")]
    RegexError(regex::Error),
    #[error("Int parse error: {0:?}")]
    IntParseError(ParseIntError),
    #[error("invalid reach step: {0:?}")]
    InvalidReachStep(u8),
    #[error("invalid naki string: {0:?}")]
    InvalidNaki(String),

    #[error("invalid tile string: {0:?}")]
    InvalidTile(String),

    #[error("insufficient dora indicators: at kyoku {kyoku} honba {honba}")]
    InsufficientDoraIndicators { kyoku: u8, honba: u8 },

    #[error(
        "insufficient take sequence size: \
        at kyoku {kyoku} honba {honba} for actor {actor}"
    )]
    InsufficientTakes { kyoku: u8, honba: u8, actor: u8 },

    #[error(
        "insufficient discard sequence size: \
        at kyoku {kyoku} honba {honba} for actor {actor}"
    )]
    InsufficientDiscards { kyoku: u8, honba: u8, actor: u8 },

    #[error(
        "unexpected naki: \
        at kyoku {kyoku} honba {honba} for actor {actor}: \
        action {action:?}, expected tile {last_discard} \
        from {last_actor:?}"
    )]
    UnexpectedNaki {
        action: Event,
        last_discard: Tile,
        last_actor: Option<u8>,
        kyoku: u8,
        honba: u8,
        actor: u8,
    },
}

impl From<ParseIntError> for ConvertError {
    fn from(value: ParseIntError) -> Self {
        Self::IntParseError(value)
    }
}

impl From<quick_xml::DeError> for ConvertError {
    fn from(value: quick_xml::DeError) -> Self {
        Self::XMLDeError(value)
    }
}

// impl From<Vec<std::string::String>>> for ConvertError {
//     fn from(value: ParseIntError) -> Self {
//         Self::IntParseError(value)
//     }
// }

lazy_static! {
    static ref tsumo_re: Regex = Regex::new(r"[TUVW](\d{1,3})").unwrap();
    static ref dahai_re: Regex = Regex::new(r"[DEFG](\d{1,3})").unwrap();
}

pub type Result<T> = std::result::Result<T, ConvertError>;

// Transform a tenhou.net/0 format log into mjai format.
pub fn tenhou_xml_to_mjai(xml: &str) -> Result<Vec<Event>> {
    let mut reader = Reader::from_str(xml);
    let mut buf = Vec::new();

    let mut names: [String; 4] = Default::default();
    let mut game_length: u8 = 0;
    let mut last_tsumo: u8 = 0;
    let mut un_decoded = false;
    let mut aka_ari = true; // TODO: Read aka flag
    let mut kyoku_events: Vec<Event> = vec![];

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(XMLEvent::Empty(e)) => {
                let owned_e = e.into_owned();
                let tag_str = format!("<{}>", std::str::from_utf8(&buf).unwrap());
                let mut deser = Deserializer::from_str(&tag_str);

                match owned_e.name().as_ref() {
                    b"GO" => {
                        let go = Go::deserialize(&mut deser)?;
                        if go.game_type & 0b11001 != 0b01001 {
                            return Err(ConvertError::TypeNotImplemented(go.game_type));
                        }

                        aka_ari = go.game_type & 0b110 == 0;
                    }

                    b"UN" => {
                        if !un_decoded {
                            let un = Un::deserialize(&mut deser)?;
                            names = [un.n0, un.n1, un.n2, un.n3];
                            un_decoded = true;
                        }
                    }

                    b"INIT" => {
                        game_length += 1;
                        let init = Init::deserialize(&mut deser)?;
                        let bakaze = match init.seed.kyoku_num / 4 {
                            0 => t!(E),
                            1 => t!(S),
                            2 => t!(W),
                            _ => t!(N),
                        };
                        let tehais = [
                            init.hai0.into_mjai_list(aka_ari).try_into().unwrap(),
                            init.hai1.into_mjai_list(aka_ari).try_into().unwrap(),
                            init.hai2.into_mjai_list(aka_ari).try_into().unwrap(),
                            init.hai3.into_mjai_list(aka_ari).try_into().unwrap(),
                        ];

                        if game_length > 1 {
                            kyoku_events.push(Event::EndKyoku);
                        }
                        kyoku_events.push(Event::StartKyoku {
                            bakaze,
                            dora_marker: Tile::from_tenhou0(init.seed.dora_marker, aka_ari),
                            kyoku: (init.seed.kyoku_num % 4) + 1,
                            honba: init.seed.honba,
                            kyotaku: init.seed.riichi_bets,
                            oya: init.oya,
                            scores: init.ten.to_arr(),
                            tehais,
                        });
                    }

                    b"AGARI" => {
                        let agari = Agari::deserialize(&mut deser)?;
                        let ura_markers = agari
                            .dora_hai_ura
                            .unwrap_or(TileList(Vec::new()))
                            .into_mjai_list(aka_ari);

                        kyoku_events.push(Event::Hora {
                            actor: agari.who,
                            target: agari.from_who,
                            deltas: Some(agari.sc.deltas.to_arr()),
                            ura_markers: Some(ura_markers),
                        });
                    }

                    b"RYUUKYOKU" => {
                        let ryuukyoku = Ryuukyoku::deserialize(&mut deser)?;
                        let deltas_arr = ryuukyoku.sc.deltas.to_arr();

                        kyoku_events.push(Event::Ryukyoku {
                            deltas: Some(deltas_arr),
                        });
                    }

                    b"N" => {
                        let meld_event = MeldEvent::deserialize(&mut deser)?;
                        let meld = Meld::from(meld_event.m);

                        kyoku_events.push(meld.into_mjai(meld_event.who, aka_ari));
                    }

                    b"REACH" => {
                        let reach = Reach::deserialize(&mut deser)?;
                        let event = match reach.step {
                            1 => Event::Reach { actor: reach.who },
                            2 => Event::ReachAccepted { actor: reach.who },
                            _ => return Err(ConvertError::InvalidReachStep(reach.step)),
                        };

                        kyoku_events.push(event);
                    }

                    b"DORA" => {
                        let dora = Dora::deserialize(&mut deser)?;

                        kyoku_events.push(Event::Dora {
                            dora_marker: Tile::from_tenhou0(dora.hai, aka_ari),
                        });
                    }
                    name => {
                        let tsumo_cap = tsumo_re
                            .captures(std::str::from_utf8(name).unwrap())
                            .map(|caps| caps.get(1));

                        let dahai_cap = dahai_re
                            .captures(std::str::from_utf8(name).unwrap())
                            .map(|caps| caps.get(1));

                        if let Some(Some(tile_match)) = tsumo_cap {
                            let tile_num: u8 = tile_match.as_str().parse()?;
                            let actor = name[0] - 0x54;

                            last_tsumo = tile_num;
                            kyoku_events.push(Event::Tsumo {
                                actor,
                                pai: Tile::from_tenhou0(tile_num, aka_ari),
                            });
                        } else if let Some(Some(tile_match)) = dahai_cap {
                            let tile_num: u8 = tile_match.as_str().parse()?;
                            let actor = name[0] - 0x44;
                            let tsumogiri = last_tsumo == tile_num;

                            kyoku_events.push(Event::Dahai {
                                actor,
                                pai: Tile::from_tenhou0(tile_num, aka_ari),
                                tsumogiri,
                            });
                        }
                    }
                }
            }
            Ok(XMLEvent::Eof) => break,
            Err(e) => return Err(ConvertError::XMLParseError(e)),
            _ => {}
        }

        buf.clear();
    }

    let mut events = vec![Event::StartGame {
        kyoku_first: game_length,
        aka_flag: aka_ari,
        names: names,
    }];

    events.extend(kyoku_events);
    events.push(Event::EndGame);

    Ok(events)
}
