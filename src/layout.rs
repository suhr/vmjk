use sfml::window::Key;

#[derive(Debug)]
pub struct Layout(Vec<(Key, u8)>);

impl Layout {
    pub fn new() -> Self {
        let mut map = Vec::new();

        map.push((Key::Unknown, 0));
        map.extend(
            b"azsxdcfvgbhnjmk,l.;/'".into_iter()
                .map(|&b| into_key(b)).zip(1..22)
        );
        map.push((Key::Quote, 21));
        map.push((Key::RShift, 22));
        map.push((Key::BackSlash, 23));
        map.push((Key::RControl, 24));

        map.extend(
            b"q2w3e4r5t6y7u8i9o0p-[=]".into_iter()
                .map(|&b| into_key(b))
            .zip(0..23)
        );
        map.push((Key::BackSpace, 23));

        Layout(map)
    }

    pub fn note(&self, base: u8, key: Key) -> Option<u8> {
        self.0.iter().find(|&&(k, _)| k == key)
            .map(|&(_, n)| base + n)
    }
}

fn into_key(c: u8) -> Key {
    match c {
        0x00 => Key::Unknown,
        0x08 => Key::BackSpace,
        0x09 => Key::Tab,
        0x0D => Key::Return,
        0x1B => Key::Escape,
        0x20 => Key::Space,
        0x21 => Key::Num1,
        0x22 => Key::Quote,
        0x23 => Key::Num3,
        0x24 => Key::Num4,
        0x25 => Key::Num5,
        0x26 => Key::Num7,
        0x27 => Key::Quote,
        0x28 => Key::Num9,
        0x29 => Key::Num0,
        0x2A => Key::Num8,
        0x2B => Key::Equal, // +
        0x2C => Key::Comma,
        0x2D => Key::Dash, //Dash
        0x2E => Key::Period,
        0x2F => Key::Slash,
        0x30 => Key::Num0,
        0x31 => Key::Num1,
        0x32 => Key::Num2,
        0x33 => Key::Num3,
        0x34 => Key::Num4,
        0x35 => Key::Num5,
        0x36 => Key::Num6,
        0x37 => Key::Num7,
        0x38 => Key::Num8,
        0x39 => Key::Num9,
        0x3A => Key::SemiColon,
        0x3B => Key::SemiColon,
        0x3C => Key::Comma,
        0x3D => Key::Equal, // =
        0x3E => Key::Comma,
        0x3F => Key::Slash,
        0x40 => Key::Num2,
        0x5B => Key::LBracket,
        0x5C => Key::BackSlash,
        0x5D => Key::RBracket,
        0x5E => Key::Num6,
        0x5F => Key::Dash,
        0x60 => Key::Tilde,
        0x61 => Key::A,
        0x62 => Key::B,
        0x63 => Key::C,
        0x64 => Key::D,
        0x65 => Key::E,
        0x66 => Key::F,
        0x67 => Key::G,
        0x68 => Key::H,
        0x69 => Key::I,
        0x6A => Key::J,
        0x6B => Key::K,
        0x6C => Key::L,
        0x6D => Key::M,
        0x6E => Key::N,
        0x6F => Key::O,
        0x70 => Key::P,
        0x71 => Key::Q,
        0x72 => Key::R,
        0x73 => Key::S,
        0x74 => Key::T,
        0x75 => Key::U,
        0x76 => Key::V,
        0x77 => Key::W,
        0x78 => Key::X,
        0x79 => Key::Y,
        0x7A => Key::Z,
        0x7F => Key::Delete,
        _ => Key::Unknown,
    }
}