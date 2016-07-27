use sfml::graphics::{RectangleShape, Transformable};
use sfml::system::Vector2f;
use sfml::graphics::{RenderTarget, Color, Text, Font, Drawable, RenderStates, Shape};
use portmidi::{PortMidi, DeviceInfo};

const LINES: f32 = 4.0;
const CMDS: &'static [&'static str] = &[
    "dev",
    "chan",
    "prog",
];

#[derive(Debug)]
pub enum Action {
    Device(DeviceInfo),
    Chan(u8),
    Patch(Option<u8>, Option<u16>),
}

// Hello Commander. It is good to see you again.
pub struct Commander<'a> {
    pub midi: &'a PortMidi,
    input: String,
    devs: Option<Vec<DeviceInfo>>,
    chosen: Option<usize>,
    current: Option<String>,
}

impl<'a> Commander<'a> {
    pub fn new(midi: &'a PortMidi) -> Self {
        Commander {
            midi: midi,
            input: String::new(),
            devs: None,
            chosen: None,
            current: None,
        }
    }

    pub fn feed(&mut self, ch: char) {
        match ch {
            '\u{8}' => {
                if self.input.is_empty() && self.current.is_some() {
                    let mut current = None;
                    ::std::mem::swap(&mut current, &mut self.current);
                    self.input = current.unwrap();
                    self.devs = None;
                } else {
                    drop(self.input.pop());
                }
                return
            },
            ' ' if self.current.is_none() => {
                if CMDS.contains(&&*self.input) {
                    let mut current = String::new();
                    ::std::mem::swap(&mut current, &mut self.input);
                    self.current = Some(current);
                }
            },
            c if c > ' ' => self.input.push(ch),
            _ => return,
        }

        self.do_cmd()
    }

    fn do_cmd(&mut self) {
        match self.current.as_ref().map(|s| s.as_str()) {
            Some("dev") => {
                let input = self.input.clone();
                if let Some(ref mut devs) = self.devs {
                    devs.retain(|d| d.name().contains(&input));
                } else {
                    let mut devs = self.midi.devices().unwrap();
                    devs.retain(|d| d.is_output());
                    self.devs = Some(devs);
                }
            },
            _ => return,
        }
    }

    pub fn emit(&mut self) -> Option<Action> {
        let act;
        {
            let cmd = self.current.as_ref().map(|s| s.as_str());
            act = match (cmd, &self.devs) {
                (Some("dev"), &Some(ref devs)) => {
                    devs.get(0).map(|d| Action::Device(d.clone()))
                },
                (Some("chan"), _) => {
                    if let Ok(ch) = self.input.parse() {
                        Some(Action::Chan(ch))
                    } else { None }
                },
                (Some("prog"), _) => {
                    let mut split = self.input.split('/');
                    let patch: Option<u8> = split.next().and_then(|s| s.parse().ok());
                    let bank: Option<u16> = split.next().and_then(|s| s.parse().ok());

                    match (patch, bank) {
                        (Some(p), None) if p < 128 => Some(Action::Patch(Some(p), None)),
                        (None, Some(b)) if b < 16384 => Some(Action::Patch(None, Some(b))),
                        (Some(p), Some(b)) if p < 128 && b < 16384 =>
                            Some(Action::Patch(Some(p), Some(b))),
                        _ => None,
                    }
                }
                _ => None,
            };
        }

        self.input.clear();
        self.devs = None;
        self.current = None;

        act
    }

    pub fn text(&self) -> String {
        let mut text = String::new();

        let cmdline = match self.current {
            Some(ref cmd) => format!("{}> {}", cmd, self.input),
            None => format!("> {}", self.input),
        };

        if let Some(ref devs) = self.devs {
            use std::iter::repeat;
            for l in devs.iter().map(|d| d.name()).chain(repeat(&"".to_string())).take(3) {
                text.push_str(l);
                text.push('\n');
            }
        } else {
            text.push_str("\n\n\n");
        }

        text.push_str(&cmdline);
        text
    }
}

pub struct CmdFrame<'a> {
    text: String,
    view: (f32, f32),
    font: &'a Font,
    font_size: f32,
}

impl<'a> CmdFrame<'a> {
    pub fn new(text: String, view: (f32, f32), font: &'a Font, font_size: u32) -> Self {
        CmdFrame {
            text: text,
            view: view,
            font: font,
            font_size: font_size as f32,
        }
    }
}


impl<'a> Drawable for CmdFrame<'a> {
    fn draw<RT: RenderTarget>(&self, target: &mut RT, rs: &mut RenderStates) {
        let (width, y) = self.view;
        let height = self.font_size * 1.2 * LINES;

        let mut rect = RectangleShape::new_init(&Vector2f::new(width, height)).unwrap();
        rect.move2f(0.0, y - height);
        rect.set_fill_color(&Color::new_rgba(0, 0, 0, 0xC0));
        rect.draw(target, rs);

        let mut text = Text::new_init(&self.text, self.font, 20).unwrap();
        text.move2f(0.0, y - height);
        text.draw(target, rs);
    }
}
