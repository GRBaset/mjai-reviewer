use std::ops::Deref;

use crate::{Event, Tile};

use serde::{Deserialize, Deserializer, Serialize};
use serde_with::{serde_as, FromInto};
use urlencoding::{decode, encode};
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Mjloggm {
    #[serde(rename = "@ver")]
    pub ver: String,
    #[serde(rename = "SHUFFLE")]
    pub shuffle: Shuffle,
    #[serde(rename = "GO")]
    pub go: Go,
    #[serde(rename = "UN")]
    pub un: Un,
    #[serde(rename = "TAIKYOKU")]
    pub taikyoku: Taikyoku,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MeldEvent {
    #[serde(rename = "@m")]
    pub m: u16,
    #[serde(rename = "@who")]
    pub who: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Reach {
    #[serde(rename = "@step")]
    pub step: u8,
    #[serde(rename = "@ten")]
    pub ten: Option<String>,
    #[serde(rename = "@who")]
    pub who: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Dora {
    #[serde(rename = "@hai")]
    pub hai: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Ryuukyoku {
    #[serde(rename = "@ba")]
    pub ba: String,
    #[serde(rename = "@hai0")]
    pub hai0: Option<TileList>,
    #[serde(rename = "@hai1")]
    pub hai1: Option<TileList>,
    #[serde(rename = "@hai2")]
    pub hai2: Option<TileList>,
    #[serde(rename = "@hai3")]
    pub hai3: Option<TileList>,
    #[serde(rename = "@sc")]
    pub sc: ScoreDelta,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Agari {
    #[serde(rename = "@ba")]
    pub ba: String,
    #[serde(rename = "@doraHai")]
    pub dora_hai: TileList,
    #[serde(rename = "@doraHaiUra")]
    pub dora_hai_ura: Option<TileList>,
    #[serde(rename = "@fromWho")]
    pub from_who: u8,
    #[serde(rename = "@hai")]
    pub hai: TileList,
    #[serde(rename = "@m")]
    pub melds: Option<MeldList>,
    #[serde(rename = "@machi")]
    pub machi: u8,
    #[serde(rename = "@owari")]
    pub owari: Option<String>,
    #[serde(rename = "@sc")]
    pub sc: ScoreDelta,
    #[serde(rename = "@ten")]
    pub ten: String,
    #[serde(rename = "@who")]
    pub who: u8,
    #[serde(rename = "@yaku")]
    pub yaku: Option<String>,
    #[serde(rename = "@yakuman")]
    pub yakuman: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Go {
    #[serde(rename = "@lobby")]
    pub lobby: u8,
    #[serde(rename = "@type")]
    pub game_type: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Shuffle {
    #[serde(rename = "@ref")]
    pub shuffle_ref: String,
    #[serde(rename = "@seed")]
    pub seed: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Taikyoku {
    #[serde(rename = "@oya")]
    pub oya: u8,
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Un {
    #[serde(rename = "@dan")]
    pub dan: Option<String>,
    #[serde(rename = "@n0")]
    #[serde_as(as = "FromInto<EncodedString>")]
    pub n0: String,
    #[serde(rename = "@n1")]
    #[serde_as(as = "FromInto<EncodedString>")]
    pub n1: String,
    #[serde(rename = "@n2")]
    #[serde_as(as = "FromInto<EncodedString>")]
    pub n2: String,
    #[serde(rename = "@n3")]
    #[serde_as(as = "FromInto<EncodedString>")]
    pub n3: String,
    #[serde(rename = "@rate")]
    pub rate: String,
    #[serde(rename = "@sx")]
    pub sx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Init {
    #[serde(rename = "@hai0")]
    pub hai0: TileList,
    #[serde(rename = "@hai1")]
    pub hai1: TileList,
    #[serde(rename = "@hai2")]
    pub hai2: TileList,
    #[serde(rename = "@hai3")]
    pub hai3: TileList,
    #[serde(rename = "@oya")]
    pub oya: u8,
    #[serde(rename = "@seed")]
    pub seed: Seed,
    #[serde(rename = "@ten")]
    pub ten: ScoreList,
}

#[derive(Debug, PartialEq)]
pub enum Meld {
    Chi {
        target_rel: u8,
        pai: u8,
        consumed: [u8; 2],
    },
    Pon {
        target_rel: u8,
        pai: u8,
        consumed: [u8; 2],
    },
    Daiminkan {
        target_rel: u8,
        pai: u8,
        consumed: [u8; 3],
    },
    Kakan {
        pai: u8,
        consumed: [u8; 3],
    },
    Ankan {
        consumed: [u8; 4],
    },
    Nuki(u8),
}

impl Meld {
    const ACTOR_MASK: u16 = 0x3;
    const CHI_MASK: u16 = 0x4;
    const PON_MASK: u16 = 0x18;
    const NUKI_MASK: u16 = 0x20;

    fn decode_chi(value: u16) -> Self {
        let target_rel = (value & Self::ACTOR_MASK) as u8;
        let offsets = [
            (value >> 3) & 0b11,
            (value >> 5) & 0b11,
            (value >> 7) & 0b11,
        ];

        let call_data = value >> 10;
        let called_i = (call_data % 3) as usize;
        let min_tile_num = call_data / 3;
        let min_tile = (min_tile_num / 7) * 9 + min_tile_num % 7;

        let mut meld: Vec<u8> = offsets
            .iter()
            .enumerate()
            .map(|(i, offset)| (offset + 4 * (min_tile + i as u16)) as u8)
            .collect();

        let pai = meld.remove(called_i);

        Self::Chi {
            target_rel,
            pai,
            consumed: meld.try_into().unwrap(),
        }
    }

    fn decode_pon(value: u16) -> Self {
        let target_rel = (value & Self::ACTOR_MASK) as u8;
        let fourth_offset = (value >> 5) & 0b11;

        let call_data = value >> 9;
        let called_i = (call_data % 3) as usize;
        let tile_num = call_data / 3;

        if value & 0x8 != 0 {
            let offsets: Vec<u16> = (0..4)
                .into_iter()
                .filter(|offset| *offset != fourth_offset)
                .collect();

            let mut meld: Vec<u8> = offsets
                .iter()
                .map(|offset| (offset + 4 * tile_num) as u8)
                .collect();

            let pai = meld.remove(called_i);

            Self::Pon {
                target_rel,
                pai,
                consumed: meld.try_into().unwrap(),
            }
        } else {
            let offsets: Vec<u16> = (0..4).into_iter().collect();

            let mut meld: Vec<u8> = offsets
                .iter()
                .map(|offset| (offset + 4 * tile_num) as u8)
                .collect();

            let pai = meld.remove(fourth_offset as usize);

            Self::Kakan {
                pai,
                consumed: meld.try_into().unwrap(),
            }
        }
    }

    fn decode_kan(value: u16) -> Self {
        let target_rel = (value & Self::ACTOR_MASK) as u8;
        let offsets: Vec<u16> = (0..4).into_iter().collect();

        let call_data = value >> 8;
        let called_i = (call_data % 4) as usize;
        let tile_num = call_data / 4;

        let mut meld: Vec<u8> = offsets
            .iter()
            .map(|offset| (offset + 4 * tile_num) as u8)
            .collect();

        if target_rel != 0 {
            let pai = meld.remove(called_i);

            Self::Daiminkan {
                target_rel,
                pai,
                consumed: meld.try_into().unwrap(),
            }
        } else {
            Self::Ankan {
                consumed: meld.try_into().unwrap(),
            }
        }
    }

    fn decode_nuki(value: u16) -> Self {
        let tile_num = (value >> 8) as u8;

        Self::Nuki(tile_num)
    }

    pub fn into_mjai(self, actor: u8, aka_ari: bool) -> Event {
        match self {
            Self::Chi {
                target_rel,
                pai,
                consumed,
            } => Event::Chi {
                actor,
                target: (target_rel + actor) % 4,
                pai: Tile::from_tenhou0(pai, aka_ari),
                consumed: consumed.map(|pai| Tile::from_tenhou0(pai, aka_ari)),
            },
            Self::Pon {
                target_rel,
                pai,
                consumed,
            } => Event::Pon {
                actor,
                target: (target_rel + actor) % 4,
                pai: Tile::from_tenhou0(pai, aka_ari),
                consumed: consumed.map(|pai| Tile::from_tenhou0(pai, aka_ari)),
            },
            Self::Kakan { pai, consumed } => Event::Kakan {
                actor,
                pai: Tile::from_tenhou0(pai, aka_ari),
                consumed: consumed.map(|pai| Tile::from_tenhou0(pai, aka_ari)),
            },
            Self::Daiminkan {
                target_rel,
                pai,
                consumed,
            } => Event::Daiminkan {
                actor,
                target: (target_rel + actor) % 4,
                pai: Tile::from_tenhou0(pai, aka_ari),
                consumed: consumed.map(|pai| Tile::from_tenhou0(pai, aka_ari)),
            },
            Self::Ankan { consumed } => Event::Ankan {
                actor,
                consumed: consumed.map(|pai| Tile::from_tenhou0(pai, aka_ari)),
            },
            _ => panic!("Meld not implemented in mjai"),
        }
    }
}

impl From<u16> for Meld {
    fn from(value: u16) -> Self {
        if value & Self::CHI_MASK != 0 {
            Self::decode_chi(value)
        } else if value & Self::PON_MASK != 0 {
            Self::decode_pon(value)
        } else if value & Self::NUKI_MASK != 0 {
            Self::decode_nuki(value)
        } else {
            Self::decode_kan(value)
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct EncodedString(String);

impl Into<String> for EncodedString {
    fn into(self) -> String {
        decode(&self.0).unwrap().into_owned()
    }
}

impl From<String> for EncodedString {
    fn from(value: String) -> Self {
        EncodedString(encode(&value).into_owned())
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct MeldList(Vec<u16>);

impl Deref for MeldList {
    type Target = Vec<u16>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for MeldList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_sequence = String::deserialize(deserializer)?;
        let melds: Vec<u16> = str_sequence
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        Ok(Self(melds))
    }
}
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScoreList(pub [i32; 4]);

impl Deref for ScoreList {
    type Target = [i32; 4];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl ScoreList {
    pub fn to_arr(&self) -> [i32; 4] {
        self.map(|score| score * 100)
    }
}

impl<'de> Deserialize<'de> for ScoreList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_sequence = String::deserialize(deserializer)?;
        let scores: Vec<i32> = str_sequence
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        Ok(Self(scores.try_into().unwrap()))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ScoreDelta {
    pub initial_scores: ScoreList,
    pub deltas: ScoreList,
}

impl<'de> Deserialize<'de> for ScoreDelta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_sequence = String::deserialize(deserializer)?;
        let score_sequence: Vec<i32> = str_sequence
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        let initial_scores = ScoreList([
            score_sequence[0],
            score_sequence[2],
            score_sequence[4],
            score_sequence[6],
        ]);
        let deltas = ScoreList([
            score_sequence[1],
            score_sequence[3],
            score_sequence[5],
            score_sequence[7],
        ]);

        Ok(Self {
            initial_scores,
            deltas,
        })
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct TileList(pub Vec<u8>);

impl Deref for TileList {
    type Target = Vec<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl TileList {
    pub fn into_mjai_list(self, aka_ari: bool) -> Vec<Tile> {
        self.iter()
            .map(|tile| Tile::from_tenhou0(*tile, aka_ari))
            .collect()
    }
}

impl<'de> Deserialize<'de> for TileList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_sequence = String::deserialize(deserializer)?;
        let tiles: Vec<u8> = str_sequence
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();

        Ok(Self(tiles))
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct Seed {
    pub kyoku_num: u8,
    pub honba: u8,
    pub riichi_bets: u8,
    pub dora_marker: u8,
}

impl<'de> Deserialize<'de> for Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let str_sequence = String::deserialize(deserializer)?;
        let seed_list: Vec<u8> = str_sequence
            .split(',')
            .map(|s| s.parse().unwrap())
            .collect();
        let dora_marker = seed_list[5];

        Ok(Self {
            kyoku_num: seed_list[0],
            honba: seed_list[1],
            riichi_bets: seed_list[2],
            dora_marker: dora_marker,
        })
    }
}

#[cfg(test)]
mod test {
    use quick_xml::de;

    use super::*;

    #[test]
    fn test_melds() {
        let chi_num: u16 = 13719;
        let chi = Meld::Chi {
            target_rel: 3,
            pai: 20,
            consumed: [18, 27],
        };
        let chi_parsed = Meld::from(chi_num);

        let pon_num: u16 = 42026;
        let pon = Meld::Pon {
            target_rel: 2,
            pai: 110,
            consumed: [108, 111],
        };
        let pon_parsed = Meld::from(pon_num);

        let pon_num_2: u16 = 6251;
        let pon_2 = Meld::Pon {
            target_rel: 3,
            pai: 16,
            consumed: [17, 18],
        };
        let pon_parsed_2 = Meld::from(pon_num_2);

        let pon_num_3: u16 = 33866;
        let pon_3 = Meld::Pon {
            target_rel: 2,
            pai: 88,
            consumed: [89, 91],
        };
        let pon_parsed_3 = Meld::from(pon_num_3);

        let kakan_num: u16 = 44113;
        let kakan = Meld::Kakan {
            pai: 114,
            consumed: [112, 113, 115],
        };
        let kakan_parsed = Meld::from(kakan_num);

        let kakan_num_2: u16 = 6259;
        let kakan_2 = Meld::Kakan {
            pai: 19,
            consumed: [16, 17, 18],
        };
        let kakan_parsed_2 = Meld::from(kakan_num_2);

        let kakan_num_3: u16 = 33874;
        let kakan_3 = Meld::Kakan {
            pai: 90,
            consumed: [88, 89, 91],
        };
        let kakan_parsed_3 = Meld::from(kakan_num_3);

        let daiminkan_num: u16 = 18945;
        let daiminkan = Meld::Daiminkan {
            target_rel: 1,
            pai: 74,
            consumed: [72, 73, 75],
        };
        let daiminkan_parsed = Meld::from(daiminkan_num);

        let ankan_num: u16 = 32768;
        let ankan = Meld::Ankan {
            consumed: [128, 129, 130, 131],
        };
        let ankan_parsed = Meld::from(ankan_num);

        let ankan_num_2: u16 = 33280;
        let ankan_2 = Meld::Ankan {
            consumed: [128, 129, 130, 131],
        };
        let ankan_parsed_2 = Meld::from(ankan_num_2);

        let nuki_num: u16 = 31520;
        let nuki = Meld::Nuki(123);
        let nuki_parsed = Meld::from(nuki_num);

        assert_eq!(chi, chi_parsed);

        assert_eq!(pon, pon_parsed);
        assert_eq!(pon_2, pon_parsed_2);
        assert_eq!(pon_3, pon_parsed_3);

        assert_eq!(kakan, kakan_parsed);
        assert_eq!(kakan_2, kakan_parsed_2);
        assert_eq!(kakan_3, kakan_parsed_3);

        assert_eq!(daiminkan, daiminkan_parsed);

        assert_eq!(ankan, ankan_parsed);
        assert_eq!(ankan_2, ankan_parsed_2);

        assert_eq!(nuki, nuki_parsed);
    }

    #[test]
    fn test_seed() {
        let seed_str = "7,3,2,0,4,69";
        let seed = Seed {
            kyoku_num: 7,
            honba: 3,
            riichi_bets: 2,
            dora_marker: 69,
        };

        let mut deser = de::Deserializer::from_str(&seed_str);
        let deser_seed = Seed::deserialize(&mut deser).unwrap();

        assert_eq!(deser_seed, seed)
    }

    #[test]
    fn test_meld_list() {
        let meld_str = "1610,36191";
        let meld_list = MeldList(Vec::from([1610, 36191]));

        let mut deser = de::Deserializer::from_str(&meld_str);
        let deser_meld_list = MeldList::deserialize(&mut deser).unwrap();

        assert_eq!(deser_meld_list, meld_list)
    }
    #[test]
    fn test_score_list() {
        let score_str = "273,164,332,211";
        let score_list = ScoreList([273, 164, 332, 211]);

        let mut deser = de::Deserializer::from_str(&score_str);
        let deser_score_list = ScoreList::deserialize(&mut deser).unwrap();

        assert_eq!(deser_score_list, score_list)
    }

    #[test]
    fn test_score_deltas() {
        let score_str = "200,93,168,-83,372,0,250,0";
        let score_delta = ScoreDelta {
            initial_scores: ScoreList([200, 168, 372, 250]),
            deltas: ScoreList([93, -83, 0, 0]),
        };

        let mut deser = de::Deserializer::from_str(&score_str);
        let deser_score_delta = ScoreDelta::deserialize(&mut deser).unwrap();

        assert_eq!(score_delta, deser_score_delta)
    }

    #[test]
    fn test_un() {
        let log_content = r#"<UN n0="%E3%81%93%E3%81%84%E3%82%8F%E3%81%84%E3%82%88%E3%81%A4%E3%81%B0" n1="%E3%82%B5%E3%83%B3%E3%83%9E%E9%81%93%E6%A5%BD%E4%BB%A3%E8%A1%A8" n2="%78%6C%44%61%69%6C%78" n3="%74%6F%72%69%30%36%31%32" dan="17,16,16,18" rate="2129.98,2113.02,2159.98,2231.89" sx="M,M,M,M"/>"#;

        let mut deser = de::Deserializer::from_str(&log_content);

        dbg!(Un::deserialize(&mut deser).unwrap());
    }
    #[test]
    fn test_init() {
        let log_content = r#"<INIT seed="7,3,2,0,4,69" ten="273,164,332,211" oya="3"
        hai0="28,88,124,17,22,135,20,76,55,49,67,1,4"
        hai1="32,30,21,115,50,41,131,7,93,40,100,62,91"
        hai2="120,51,45,58,9,61,44,111,12,48,46,33,126"
        hai3="70,119,54,101,66,92,118,56,134,2,60,5,39" />"#;

        let init_example = Init {
            // hai0: TileList(Vec::from(t![
            //     8m, 5sr, P, 5m, 6m, C, 6m, 2s, 5p, 4p, 8p, 1m, 2m
            // ])),
            // hai1: TileList(Vec::from(t![
            //     9m, 8m, 6m, S, 4p, 2p, F, 2m, 6s, 2p, 8s, 7p, 5s
            // ])),
            // hai2: TileList(Vec::from(t![
            //     N, 4p, 3p, 6p, 3m, 7p, 3p, E, 4m, 4p, 3p, 9m, P
            // ])),
            // hai3: TileList(Vec::from(t![
            //     9p, W, 5p, 8s, 8p, 6s, W, 6p, C, 1m, 7p, 2m, 1p
            // ])),
            hai0: TileList(Vec::from([
                28, 88, 124, 17, 22, 135, 20, 76, 55, 49, 67, 1, 4,
            ])),
            hai1: TileList(Vec::from([
                32, 30, 21, 115, 50, 41, 131, 7, 93, 40, 100, 62, 91,
            ])),
            hai2: TileList(Vec::from([
                120, 51, 45, 58, 9, 61, 44, 111, 12, 48, 46, 33, 126,
            ])),
            hai3: TileList(Vec::from([
                70, 119, 54, 101, 66, 92, 118, 56, 134, 2, 60, 5, 39,
            ])),
            oya: 3,
            seed: Seed {
                kyoku_num: 7,
                honba: 3,
                riichi_bets: 2,
                dora_marker: 69,
            },
            ten: ScoreList([273, 164, 332, 211]),
        };

        let mut deser = de::Deserializer::from_str(&log_content);

        assert_eq!(init_example, Init::deserialize(&mut deser).unwrap())
    }

    #[test]
    fn test_agari() {
        let log_content = r#"<AGARI ba="0,1" hai="24,25,26,83,86,91,92,94" m="1610,36191" machi="91" ten="30,2900,0"
        yaku="8,1,52,1" doraHai="87" doraHaiUra="32" who="1" fromWho="3" sc="273,0,125,39,362,0,230,-29" />"#;

        let agari_example = Agari {
            ba: String::from("0,1"),
            dora_hai: TileList(Vec::from([87])),
            dora_hai_ura: Some(TileList(Vec::from([32]))),
            from_who: 3,
            hai: TileList(Vec::from([24, 25, 26, 83, 86, 91, 92, 94])),
            melds: Some(MeldList(Vec::from([1610, 36191]))),
            machi: 91,
            owari: None,
            sc: ScoreDelta {
                initial_scores: ScoreList([273, 125, 362, 230]),
                deltas: ScoreList([0, 39, 0, -29]),
            },
            ten: String::from("30,2900,0"),
            who: 1,
            yaku: Some(String::from("8,1,52,1")),
            yakuman: None,
        };

        let mut deser = de::Deserializer::from_str(&log_content);

        assert_eq!(agari_example, Agari::deserialize(&mut deser).unwrap())
    }
    #[test]
    fn test_ryuukyoku() {
        let log_content = r#"<RYUUKYOKU ba="1,1" sc="273,15,164,-15,362,-15,191,15" hai0="13,14,15,37,38,49,59,94,96,100"
        hai3="0,1,2,9,10,11,19,73,77,81,82,85,89" />"#;

        let ryuukyoku_example = Ryuukyoku {
            ba: String::from("1,1"),
            hai0: Some(TileList(Vec::from([
                13, 14, 15, 37, 38, 49, 59, 94, 96, 100,
            ]))),
            hai1: None,
            hai2: None,
            hai3: Some(TileList(Vec::from([
                0, 1, 2, 9, 10, 11, 19, 73, 77, 81, 82, 85, 89,
            ]))),
            sc: ScoreDelta {
                initial_scores: ScoreList([273, 164, 362, 191]),
                deltas: ScoreList([15, -15, -15, 15]),
            },
        };

        let mut deser = de::Deserializer::from_str(&log_content);

        assert_eq!(
            ryuukyoku_example,
            Ryuukyoku::deserialize(&mut deser).unwrap()
        )
    }
}
