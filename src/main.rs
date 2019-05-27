/// Demonstrates the simultaneous mixing of music and sound effects.

extern crate sdl2;

use std::env;
use std::path::Path;
use sdl2::mixer::{InitFlag, DEFAULT_CHANNELS, AUDIO_S16LSB};

fn main() -> Result<(), String> {
    let args: Vec<_> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: ./demo music.[mp3|wav|ogg] [sound-effect.[mp3|wav|ogg]]")
    } else {
        let sound_file = args.get(2).map(|sound_file| Path::new(sound_file));
        demo(Path::new(&args[1]), sound_file)?;
    }

    Ok(())
}

fn demo(music_file: &Path, sound_file: Option<&Path>) -> Result<(), String> {
    println!("linked version: {}", sdl2::mixer::get_linked_version());

    let sdl = sdl2::init()?;
    let _audio = sdl.audio()?;
    let mut timer = sdl.timer()?;

    let frequency = 44_100;
    let format = AUDIO_S16LSB; // signed 16 bit samples, in little-endian byte order
    let channels = DEFAULT_CHANNELS; // Stereo
    let chunk_size = 1_024;
    sdl2::mixer::open_audio(frequency, format, channels, chunk_size)?;
    let _mixer_context = sdl2::mixer::init(
        InitFlag::MP3 | InitFlag::FLAC | InitFlag::MOD | InitFlag::OGG
    )?;

    // Number of mixing channels available for sound effect `Chunk`s to play
    // simultaneously.
    sdl2::mixer::allocate_channels(4);

    {
        let n = sdl2::mixer::get_chunk_decoders_number();
        println!("available chunk(sample) decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_chunk_decoder(i));
        }
    }

    {
        let n = sdl2::mixer::get_music_decoders_number();
        println!("available music decoders: {}", n);
        for i in 0..n {
            println!("  decoder {} => {}", i, sdl2::mixer::get_music_decoder(i));
        }
    }

    println!("query spec => {:?}", sdl2::mixer::query_spec());

    let music = sdl2::mixer::Music::from_file(music_file)?;

    fn hook_finished() {
        println!("play ends! from rust cb");
    }

    sdl2::mixer::Music::hook_finished(hook_finished);

    println!("music => {:?}", music);
    println!("music type => {:?}", music.get_type());
    println!("music volume => {:?}", sdl2::mixer::Music::get_volume());
    println!("play => {:?}", music.play(1));
    sdl2::mixer::Music::pause();

    let joystick_subsystem = sdl.joystick()?;
    let available  = joystick_subsystem.num_joysticks()
        .map_err(|e| format!("can't enumerate joysticks: {}", e))?;
    println!("{} joysticks available", available);

    let joystick = (0..available).find_map(|id| match joystick_subsystem.open(id) {
        Ok(c) => {
            println!("Success: opened \"{}\"", c.name());
            Some(c)
        },
        Err(e) => {
            println!("failed: {:?}", e);
            None
        },
    }).expect("Couldn't open any joystick");

    println!("\"{}\" power level: {:?}", joystick.name(), joystick.power_level()
             .map_err(|e| e.to_string())?);

    let mut cur_ch = (0..4).cycle();

    if let Ok(sounds) = load_sounds() {
        let mut cur_sound = (0..sounds.len()).cycle();

        for event in sdl.event_pump()?.wait_iter() {
            use sdl2::event::Event;

            match event {
                Event::JoyButtonDown{ button_idx, ..  } => {
                    if button_idx == 1 {
                        sdl2::mixer::Music::resume();
                    } else if let Some(ch) = cur_ch.next() {
                        if let Some(sound_idx) = cur_sound.next() {
                            sdl2::mixer::Channel(ch).halt();
                            sdl2::mixer::Channel(ch).play(&sounds[sound_idx], 0)?;
                        }
                    }
                }
                Event::JoyButtonUp{ button_idx, .. } => {
                    if button_idx == 1 {
                        sdl2::mixer::Music::pause();
                    }
                }
                Event::Quit{..} => break,
                _ => (),
            }
        }
    }

    println!("fading out ... {:?}", sdl2::mixer::Music::fade_out(4_000));

    timer.delay(5_000);

    println!("fading in from pos ... {:?}",
        music.fade_in_from_pos(1, 10_000, 100.0));

    timer.delay(5_000);
    sdl2::mixer::Music::halt();
    timer.delay(1_000);

    println!("quitting sdl");

    Ok(())
}

fn load_sounds() -> Result<Vec<sdl2::mixer::Chunk>, String> {
    let files = vec![
        "assets/Home Studio Stuff - Speak & Spell Vocal Sample Pack/Blips & Bloops 1_bip.wav",
        "assets/Home Studio Stuff - Speak & Spell Vocal Sample Pack/Blips & Bloops 3_bip.wav",
        "assets/Home Studio Stuff - Speak & Spell Vocal Sample Pack/Blips & Bloops 4_bip.wav",
    ];
    files
        .iter()
        .map(|sound_file| Path::new(sound_file))
        .map(|path| sdl2::mixer::Chunk::from_file(path))
        .collect()
}
