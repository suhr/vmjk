extern crate getopts;
extern crate portmidi;
extern crate sfml;

use portmidi::{PortMidi, MidiMessage, OutputPort, Result as PmResult};
use sfml::window::*;
use sfml::graphics::{RenderWindow, RenderTarget, Color, View, Rect};

use layout::*;
use ui::*;

mod layout;
mod ui;

struct MusicBox {
    pub hexes: Hexes,
    map: Layout,
    port: OutputPort,
    chan: u8,
    low: bool,
}

impl MusicBox {
    fn new(port: OutputPort) -> Self {
        MusicBox {
            hexes: Hexes::new(),
            map: Layout::new(),
            port: port,
            chan: 0,
            low: false,
        }
    }

    fn note_on(&mut self, note: u8) -> PmResult<()> {
        let msg = MidiMessage {
            status: 0x90 + self.chan,
            data1: note,
            data2: 64,
        };

        self.port.write_message(msg)
    }

    fn note_off(&mut self, note: u8) -> PmResult<()> {
        let msg = MidiMessage {
            status: 0x80 + self.chan,
            data1: note,
            data2: 64,
        };

        self.port.write_message(msg)
    }

    fn all_notes_off(&mut self) {
        // Better to send ALL NOTES OFF, but there're some synths that don't understand it
        for n in 0..36 {
            drop(self.note_off(48 + n))
        }
    }

    fn press(&mut self, key: Key) {
        match key {
            Key::Add if self.chan != 15 => 
                self.chan += 1,
            Key::Subtract if self.chan != 0 => 
                self.chan -= 1,
            Key::Space => {
                if self.low == false { self.low = true } else { self.low = false };

                let shift = if self.low == false { 0 } else { 12 };
                self.hexes.base_note(60 - shift);
            },
            Key::Return => {
                self.all_notes_off();
                self.hexes.release_all();
            },

            _ => if let Some(note) = self.map.note(60, key) {
                let note = note - if self.low == false { 0 } else { 12 };
                drop(self.note_on(note));
                self.hexes.press(note);
            }
        }
    }

    fn release(&mut self, key: Key) {
        if let Some(note) = self.map.note(60, key) {
            let note =  note - if self.low == false { 0 } else { 12 };
            drop(self.note_off(note));
            self.hexes.release(note);
        }
    }
}


fn proceed(midi: PortMidi, port: OutputPort) {
    let mut the_box = MusicBox::new(port);

    let mut window = RenderWindow::new(
        VideoMode::new_init(980, 280, 32),
        "Virtual Midi Janko Keyboard",
        WindowStyle::default(),
        &ContextSettings::default().antialiasing(8),
    ).expect("Cannot create a new Render Window.");
    window.set_key_repeat_enabled(false);

    loop {
        loop {
            let event = window.poll_event();
            match event {
                Some(Event::Closed) => return,
                Some(Event::Resized {width: w, height: h}) => {
                    window.set_view(&View::new_from_rect(&Rect::new(0.0, 0.0, w as f32, h as f32)).unwrap());
                    the_box.hexes.resize(w as f32, h as f32);
                },
                Some(Event::KeyPressed {code, ..}) => the_box.press(code),
                Some(Event::KeyReleased {code, ..}) => the_box.release(code),
                None => break,
                _ => (),
            }
        }
        window.clear(&Color::new_rgb(0x21, 0x21, 0x21));

        window.draw(&the_box.hexes);
        window.display();
        ::std::thread::sleep(::std::time::Duration::from_millis(25));
    }
}

fn usage(prog: &str, opts: getopts::Options) {
    let usage = format!("Usage: {} [options]", prog);
    print!("{}", opts.usage(&usage));
}

fn main() {
    use getopts::*;
    use std::env::*;

    let midi = PortMidi::new().unwrap();
    let port;

    let mut args = args();
    let prog = args.next().unwrap();

    let mut opts = Options::new();
    opts.optflag("h", "help", "show this help message");
    opts.optflag("l", "list", "list midi ports available");
    opts.optopt("p", "port", "connect to midi port [id]", "id");

    let matches = opts.parse(args).unwrap();
    if matches.opt_present("h") {
        usage(&prog, opts);
        return
    }
    if matches.opt_present("l") {
        let devs = midi.devices().unwrap();
        for d in devs.iter().filter(|d| d.is_output()) {
            println!("{}: {}", d.id(), d.name());
        }
        return
    }
    if let Some(id) = matches.opt_str("p") {
        use portmidi::PmError::PmInvalidDeviceId;
        use portmidi::Error::PortMidi;

        let dev = id.parse::<i32>().map(|id| midi.device(id));
        let dev = match dev {
            Ok(Ok(dev)) =>
                if dev.is_output() {
                    dev
                } else {
                    println!("Not an output device: {}", id);
                    return
                },
            Err(_) | Ok(Err(PortMidi(PmInvalidDeviceId))) => {
                println!("Invalid device id: {}", id);
                return
            },
            Ok(Err(e)) => panic!("MIDI error: {}", e),
        };
        port = midi.output_port(dev, 1024)
            .unwrap_or_else(|e| panic!("Cannot connect to midi port {}: {}", id, e))
    } else {
        if midi.device_count() == 0 { panic!("No midi devices in the system") }

        port = midi.default_output_port(1024).expect("Cannot connect to default output port");
    }

    proceed(midi, port);
}
