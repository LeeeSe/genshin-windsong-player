#![cfg_attr(
    debug_assertions,
    allow(dead_code, unused_imports, unused_variables, unused_mut)
)]

use genshin_windsong_player::{txt_player, Midiplayer};
use std::env;
fn main() {
    let args: Vec<String> = env::args().collect();
    let midi_file_path = &args[1];
    let midi_file = std::fs::read(midi_file_path).unwrap();

    let mut midi = Midiplayer::build(&midi_file).unwrap_or_else(|e| {
        println!("Error when build midiplayer: {}", e);
        std::process::exit(1);
    });
    midi.tracks_to_txt_for_play(&midi.tracks, "play_tracks.txt");

    let start = std::time::Instant::now();

    println!("初始化完成,请切屏到原神并打开想要使用的乐器,10s后开始弹奏");
    std::thread::sleep(std::time::Duration::from_secs(10));

    txt_player("play_tracks.txt");
    println!("Time of player is: {:?}", start.elapsed());
    // let track_merged = merge_tracks(&midi.tracks);
    // tracks_to_txt(&midi.tracks, "./");
}
