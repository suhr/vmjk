extern crate getopts;
extern crate portmidi;
extern crate sfml;

use portmidi::{PortMidi, MidiMessage, OutputPort, Result as PmResult};
use sfml::window::*;
use sfml::graphics::{Drawable, RenderWindow, RenderTarget, RenderStates, Color, View, Rect, Text, Font};

use layout::*;
use ui::*;
use cmd::*;

mod layout;
mod ui;
mod cmd;

pub static FONT: &'static [u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/PTM55F.ttf"));

struct MusicBox<'a> {
    cmd: Commander<'a>,
    view: (f32, f32),
    pub hexes: Hexes,
    pub font: Box<Font>,
    font_size: u32,
    map: Layout,
    port: OutputPort,
    chan: u8,
    patches: [(Option<u8>, Option<u16>); 16],
    low: bool,
    cmd_mode: bool,
}

impl<'a> MusicBox<'a> {
    fn new(midi: &'a PortMidi, port: OutputPort, view: (f32, f32), font_size: u32) -> Self {
        let font = Box::new(Font::new_from_memory(FONT).unwrap());

        MusicBox {
            cmd: Commander::new(midi),
            view: view,
            hexes: Hexes::new(1.5 * font_size as f32),
            font: font,
            font_size: font_size,
            map: Layout::new(),
            port: port,
            chan: 0,
            patches: [(None, None); 16],
            low: false,
            cmd_mode: false,
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

        self.hexes.release_all();
    }

    fn set_patch(&mut self, patch: u8) {
        let msg = MidiMessage {
            status: 0xC0 + self.chan,
            data1: patch,
            data2: 0,
        };

        drop(self.port.write_message(msg));
        self.patches[self.chan as usize].0 = Some(patch);
    }

    fn set_bank(&mut self, bank: u16) {
        let (msb, lsb) = ((bank >> 7) as u8, (bank & 0x7F) as u8);
        let msg = MidiMessage {
            status: 0xD0 + self.chan,
            data1: 0,
            data2: msb,
        };

        drop(self.port.write_message(msg));

        let msg = MidiMessage {
            status: 0xD0 + self.chan,
            data1: 0x20,
            data2: lsb,
        };

        drop(self.port.write_message(msg));
        self.patches[self.chan as usize].1 = Some(bank);
    }

    fn execute(&mut self, act: Action) {
        match act {
            Action::Device(dev) => {
                if let Ok(p) = self.cmd.midi.output_port(dev, 1024) {
                    self.port = p
                }
            },
            Action::Chan(n) => self.chan = n,
            Action::Patch(patch, bank) => {
                if let Some(b) = bank { self.set_bank(b) }
                if let Some(p) = patch { self.set_patch(p) }
            }
        }
    }

    fn text(&mut self, ch: char) {
        if self.cmd_mode == true {
            self.cmd.feed(ch)
        }
    }

    fn press(&mut self, key: Key, ctrl: bool) {
        if self.cmd_mode == true {
            match key {
                Key::Escape => { self.cmd_mode = false },
                Key::Return => if let Some(act) = self.cmd.emit() {
                    self.execute(act);
                    self.cmd_mode = false
                },
                _ => (),
            }
            return
        }

        match key {
            Key::Add if self.chan != 15 => 
                self.chan += 1,
            Key::Subtract if self.chan != 0 => 
                self.chan -= 1,
            Key::Space => {
                if !self.low { self.low = true } else { self.low = false };

                let shift = if self.low == false { 0 } else { 12 };
                self.hexes.base_note(60 - shift);
                self.all_notes_off();
            },
            Key::PageUp if !ctrl => {
                let num = self.patches[self.chan as usize].0
                    .map(|n| if n != 127 { n + 1 } else { n }).unwrap_or(0);
                self.set_patch(num);
            },
            Key::PageDown if !ctrl => {
                let num = self.patches[self.chan as usize].0
                    .map(|n| if n != 0 { n - 1 } else { n }).unwrap_or(0);
                self.set_patch(num);
            },
            Key::PageUp => {
                let num = self.patches[self.chan as usize].1
                    .map(|n| if n != 16383 { n + 1 } else { n }).unwrap_or(0);
                self.set_bank(num);
            },
            Key::PageDown => {
                let num = self.patches[self.chan as usize].1
                    .map(|n| if n != 0 { n - 1 } else { n }).unwrap_or(0);
                self.set_bank(num);
            },
            Key::Return => {
                self.all_notes_off();
            },
            Key::Escape => {
                self.cmd_mode = true
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

    fn status(&self) -> String {
        let level = if self.low { "Low" } else { "High" };
        let patch = self.patches[self.chan as usize].0.map(|p| p.to_string()).unwrap_or("?".to_string());
        let bank = self.patches[self.chan as usize].1.map(|p| p.to_string()).unwrap_or("?".to_string());
        let dev = self.port.device().name().to_string();

        format!(" [{}], channel {}, program {} from bank {} [{}]", level, self.chan, patch, bank, dev)
    }

    fn resize(&mut self, w: f32, h: f32) {
        self.view = (w, h);
        self.hexes.resize(w as f32, h as f32 - 1.5 * self.font_size as f32);
    }
}

impl<'a> Drawable for MusicBox<'a> {
    fn draw<RT: RenderTarget>(&self, target: &mut RT, rs: &mut RenderStates) {
        let status = self.status();
        let text = Text::new_init(&status, &self.font, self.font_size).unwrap();

        text.draw(target, rs);
        self.hexes.draw(target, rs);

        if self.cmd_mode {
            let frame = CmdFrame::new(self.cmd.text(), self.view, &self.font, self.font_size);
            frame.draw(target, rs);
        }
    }
}


fn proceed(midi: PortMidi, port: OutputPort) {
    let view = (980.0, 310.0);
    let mut the_box = MusicBox::new(&midi, port, view, 20);

    let mut window = RenderWindow::new(
        VideoMode::new_init(view.0 as u32, view.1 as u32, 32),
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
                    the_box.resize(w as f32, h as f32);
                },
                Some(Event::KeyPressed {code, ctrl, ..}) => the_box.press(code, ctrl),
                Some(Event::TextEntered {code}) => the_box.text(code),
                Some(Event::KeyReleased {code, ..}) => the_box.release(code),
                None => break,
                _ => (),
            }
        }

        window.clear(&Color::new_rgb(0x21, 0x21, 0x21));
        window.draw(&the_box);

        window.display();
        ::std::thread::sleep(::std::time::Duration::from_millis(25));
    }
}

fn get_port(midi: &PortMidi, id: Option<i32>) -> PmResult<OutputPort> {
    match id {
        Some(id) =>
            midi.device(id).and_then(|dev| midi.output_port(dev, 1024)),
        None => midi.default_output_port(1024),
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
    let port_id;

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
        if let Ok(id) = id.parse::<i32>() {
            port_id = Some(id)
        } else {
            println!("Not an integer: {}", id);
            return
        }
    } else { port_id = None }

    let port = if midi.device_count() != 0 {
        match get_port(&midi, port_id) {
            Ok(p) => p,
            Err(e) => {
                println!("{}", e);
                return
            }
        }
    } else { panic!("No midi devices in the system") };

    proceed(midi, port);
}
