#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
use enigo::{Enigo, Key, KeyboardControllable};
use midi_reader_writer::{
    midly_0_5::{
        exports::{
            num::{u28, u7},
            Header, MidiMessage, Smf, Timing, Track, TrackEvent, TrackEventKind,
        },
        merge_tracks, TrackSeparator,
    },
    ConvertMicroSecondsToTicks, ConvertTicksToMicroseconds,
};
use std::io::Read;
use std::time::Instant;
use std::{collections::HashMap, thread, time::Duration};
use std::{fmt::format, fs::File, io::Write};

pub struct Midiplayer<'a> {
    pub tempo: u32,
    pub tqpn: u32,
    pub header: Header,
    pub tracks: Vec<Vec<TrackEvent<'a>>>,
    pub keybind: HashMap<u8, char>,
}

impl<'a> Midiplayer<'a> {
    pub fn build(midi_file: &'a [u8]) -> Result<Midiplayer<'a>, Box<dyn std::error::Error>> {
        let smf = Smf::parse(&midi_file)?;
        let header = smf.header;
        let tracks = smf.tracks;
        let mut tqpn = 0;
        match header {
            Header { format, timing } => match timing {
                Timing::Metrical(tqdn) => {
                    tqpn = tqdn.as_int() as u32;
                }
                _ => {}
            },
        }
        let keybind = key_value_genshin();

        Ok(Midiplayer {
            tempo: 500000, // default tempo, it will change when parsing the midi event
            tqpn,
            header,
            tracks,
            keybind,
        })
    }

    pub fn delta_to_ms(&self, delta: u28) -> u32 {
        delta.as_int() * self.tempo / (self.tqpn * 1000)
    }

    pub fn key_adjust(
        &self,
        key: u7,
        tone_adjust: i8,
        blow_limit: u8,
        above_limit: u8,
        black_key: u8,
    ) -> u7 {
        let mut key = key.as_int();

        key += (tone_adjust * 12) as u8;
        let key_min = self.keybind.keys().min().unwrap();
        let key_max = self.keybind.keys().max().unwrap();
        if key < *key_min {
            match blow_limit {
                0 => black_key_adjusted(div_plus(key, *key_min), black_key), // 上调八度直到可播放
                1 => black_key_adjusted(u7::from(*key_min), black_key),      // 上调直到最低音
                _ => 0.into(),                                               // 跳过
            }
        } else if key > *key_max {
            match above_limit {
                0 => black_key_adjusted(div_plus(key, *key_max), black_key), // 下调八度直到可播放
                1 => black_key_adjusted(u7::from(*key_max), black_key),      // 下调直到最高音
                _ => 0.into(),                                               // 跳过
            }
        } else if key > *key_min && key < *key_max && !self.keybind.contains_key(&key) {
            black_key_adjusted(u7::from(key), black_key)
        } else {
            key.into()
        }
    }

    pub fn tracks_to_txt_for_play<'b>(&self, tracks: &[Track<'a>], save_path: &str) {
        let merged = merge_tracks(&tracks);
        let mut prev_ms = 0;
        let mut file = File::create(save_path).unwrap();
        let mut track_bytes = Vec::new();
        let mut ticks_to_microseconds =
            ConvertTicksToMicroseconds::try_from(self.header).expect("trans tick to ms failed");
        let mut microseconds_to_ticks = ConvertMicroSecondsToTicks::from(self.header);
        let mut ket_count = 0;
        for (i, (tick, track_index, kind)) in merged.enumerate() {
            // println!("{i}, {tick}, {track_index}, {kind:?}");
            // jump meta and controller event
            match kind {
                TrackEventKind::Midi { channel, message } => match message {
                    MidiMessage::NoteOn { key, vel } => {
                        let ms = ticks_to_microseconds.convert(tick, &kind);
                        let delta = ms - prev_ms;
                        ket_count += 1;
                        let key_txt = self.key_adjust(key, 0, 0, 0, 3);
                        // println!("{i}, {tick}, {track_index}, {key}, {key_txt}");
                        if delta == 0 {
                            track_bytes.extend_from_slice(format!(",{}", key_txt).as_bytes());
                        } else {
                            track_bytes
                                .extend_from_slice(format!(" {} {}", delta, key_txt).as_bytes());
                        }
                        prev_ms = ms
                    }
                    _ => {}
                },
                _ => {}
            }
        }
        println!("key count: {}", ket_count);
        file.write_all(&track_bytes).unwrap();
    }
}

pub fn key_value_genshin() -> HashMap<u8, char> {
    let mut key_value_genshin: HashMap<u8, char> = HashMap::new();
    key_value_genshin.insert(48, 'z');
    key_value_genshin.insert(50, 'x');
    key_value_genshin.insert(52, 'c');
    key_value_genshin.insert(53, 'v');
    key_value_genshin.insert(55, 'b');
    key_value_genshin.insert(57, 'n');
    key_value_genshin.insert(59, 'm');
    key_value_genshin.insert(60, 'a');
    key_value_genshin.insert(62, 's');
    key_value_genshin.insert(64, 'd');
    key_value_genshin.insert(65, 'f');
    key_value_genshin.insert(67, 'g');
    key_value_genshin.insert(69, 'h');
    key_value_genshin.insert(71, 'j');
    key_value_genshin.insert(72, 'q');
    key_value_genshin.insert(74, 'w');
    key_value_genshin.insert(76, 'e');
    key_value_genshin.insert(77, 'r');
    key_value_genshin.insert(79, 't');
    key_value_genshin.insert(81, 'y');
    key_value_genshin.insert(83, 'u');
    key_value_genshin
}

pub fn key_value_genshin_ascii() -> HashMap<char, u8> {
    let mut key_value_genshin: HashMap<char, u8> = HashMap::new();
    key_value_genshin.insert('z', 90);
    key_value_genshin.insert('x', 88);
    key_value_genshin.insert('c', 67);
    key_value_genshin.insert('v', 86);
    key_value_genshin.insert('b', 66);
    key_value_genshin.insert('n', 78);
    key_value_genshin.insert('m', 77);
    key_value_genshin.insert('a', 65);
    key_value_genshin.insert('s', 83);
    key_value_genshin.insert('d', 68);
    key_value_genshin.insert('f', 70);
    key_value_genshin.insert('g', 71);
    key_value_genshin.insert('h', 72);
    key_value_genshin.insert('j', 74);
    key_value_genshin.insert('q', 81);
    key_value_genshin.insert('w', 87);
    key_value_genshin.insert('e', 69);
    key_value_genshin.insert('r', 82);
    key_value_genshin.insert('t', 84);
    key_value_genshin.insert('y', 89);
    key_value_genshin.insert('u', 85);
    key_value_genshin
}
pub struct BlackKeyGenshin {
    c3: [u8; 5],
    c4: [u8; 5],
    c5: [u8; 5],
}

pub fn black_key_c3_to_c5() -> BlackKeyGenshin {
    let c3 = [49, 51, 54, 56, 58];
    let c4 = [61, 63, 66, 68, 70];
    let c5 = [73, 75, 78, 80, 82];
    BlackKeyGenshin { c3, c4, c5 }
}

pub fn tracks_to_txt_for_read<'a>(tracks: &[Track<'a>], save_path: &str) {
    for (i, track) in tracks.iter().enumerate() {
        let mut file = File::create(format!("{}/track{}.txt", save_path, i)).unwrap();
        let mut track_bytes = Vec::new();
        track_bytes.extend_from_slice(format!("{:#?}", track).as_bytes());
        file.write_all(&track_bytes).unwrap();
    }
}

pub fn black_key_adjusted(key: u7, mode: u8) -> u7 {
    let black_key_list = black_key_c3_to_c5();
    let key = key.as_int();
    // 如果key在黑键c3 c4 c5中
    if black_key_list.c3.contains(&key)
        || black_key_list.c4.contains(&key)
        || black_key_list.c5.contains(&key)
    {
        match mode {
            0 => 0.into(),         // 不演奏
            1 => (key - 1).into(), // 调整到前一个白键
            2 => (key + 1).into(), // 调整到后一个白键
            3 => {
                // 根据黑键所处的位置调整
                let mut key = key;
                // println! {"source black {key}"};
                if black_key_list.c3.contains(&key) {
                    key = 0; // 不演奏
                } else if black_key_list.c4.contains(&key) {
                    key = key + 1;
                } else if black_key_list.c5.contains(&key) {
                    key = key + 1;
                }
                // println! {"after black {key}"};
                key.into()
            }
            _ => key.into(),
        }
    } else {
        key.into()
    }
}

pub fn div_plus(val: u8, min_or_max: u8) -> u7 {
    // let val = val.as_int();
    if val > min_or_max {
        let flag = (val - min_or_max) / 12 + 1;
        u7::from(val - flag * 12)
    } else {
        let flag = (min_or_max - val) / 12 + 1;
        u7::from(val + flag * 12)
    }
}

pub fn txt_player(file_path: &str) {
    let mut file = File::open(file_path).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    let mut content = content
        .split(" ")
        .filter(|s| !s.is_empty())
        .collect::<Vec<&str>>();
    let mut enigo = Enigo::new();
    let event_len = content.len() / 2;
    println!("event_len: {}", event_len);
    for delta_keys in content.chunks(2) {
        let delta = delta_keys[0];
        let keys = delta_keys[1];
        // println!("{delta}, {keys}");
        let wait: u64 = delta.parse().unwrap();
        // wait
        thread::sleep(Duration::from_micros(wait));
        // test press_key 's time
        let start = Instant::now();
        press_key(keys, &mut enigo);
        let end = Instant::now();
        println!("press_key time: {:?}", end - start);
    }
}

fn press_key(keys: &str, enigo: &mut Enigo) {
    let mut keys_vec: Vec<&str> = keys.split(",").filter(|&x| x != "0").collect();
    keys_vec.dedup();
    let kvg = key_value_genshin();
    let keys_in_genshin: Vec<char> = keys_vec
        .iter()
        .map(|x| kvg.get(&x.parse::<u8>().unwrap()).cloned())
        .flatten()
        .collect();
    for key in keys_in_genshin {
        let key = Key::Layout(key);
        let start2 = Instant::now();
        enigo.key_click(key);
        let end2 = Instant::now();
        // println!("press time: {:?}", end2 - start2);
    }
    // let keys_str: String = keys_in_genshin.iter().collect();
    // enigo.key_sequence_parse_try(&keys_str).unwrap();
}
