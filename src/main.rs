extern crate portmidi;
extern crate sfml;

use portmidi::{PortMidi, MidiMessage, OutputPort, Result as PmResult};
use sfml::window::*;
use sfml::graphics::{RenderWindow, RenderTarget, Color};

use layout::*;

mod layout;

struct MusicBox {
    map: Layout,
    port: OutputPort,
    chan: u8,
    low: bool,
}

impl MusicBox {
    fn new(port: OutputPort) -> Self {
        MusicBox {
            map: Layout::new(),
            port: port,
            chan: 0,
            low: false,
        }
    }

    fn note_on(&mut self, note: u8) -> PmResult<()> {
        let shift = if self.low == false { 0 } else { 12 };
        let msg = MidiMessage {
            status: 0x90 + self.chan,
            data1: note - shift,
            data2: 64,
        };

        self.port.write_message(msg)
    }

    fn note_off(&mut self, note: u8) -> PmResult<()> {
        let shift = if self.low == false { 0 } else { 12 };
        let msg = MidiMessage {
            status: 0x80 + self.chan,
            data1: note - shift,
            data2: 64,
        };

        self.port.write_message(msg)
    }

    fn press(&mut self, key: Key) {
        match key {
            Key::Add if self.chan != 15 => 
                self.chan += 1,
            Key::Subtract if self.chan != 0 => 
                self.chan -= 1,
            Key::Space =>
                if self.low == false { self.low = true } else { self.low = false },

            _ => if let Some(note) = self.map.note(60, key) {
                self.note_on(note);
            }
        }
    }

    fn release(&mut self, key: Key) {
        if let Some(note) = self.map.note(60, key) {
            self.note_off(note);
        }
    }
}

fn main() {
    let midi = PortMidi::new().unwrap();
    let mut the_box = MusicBox::new(
        midi.device(0).and_then(|d| midi.output_port(d, 1024)).unwrap()
    );

    let mut window = RenderWindow::new(
        VideoMode::new_init(800, 600, 32),
        "Virtual Midi Janko Keyboard",
        WindowStyle::default(),
        &ContextSettings::default()
    ).expect("Cannot create a new Render Window.");
    window.set_key_repeat_enabled(false);

    loop {
        for event in window.events() {
            match event {
                Event::Closed => return,
                Event::KeyPressed {code, ..} => the_box.press(code),
                Event::KeyReleased {code, ..} => the_box.release(code),
                _ => (),
            }
        }
        window.clear(&Color::new_rgb(0, 0, 0));
        window.display();

        ::std::thread::sleep_ms(50);
    }
}
