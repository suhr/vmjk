use sfml::graphics::{Drawable, RenderTarget, RenderStates, CircleShape, Color, Shape, Transformable};

const GAP: f32 = 0.175;

const HORIZ_X: f32 = 0.866 * (2.0 + GAP);
const DIAG_X: f32 = 0.5 * HORIZ_X;
const DIAG_Y: f32 = 1.5 + 0.707 * GAP;

#[derive(Debug)]
pub struct Hexes {
    note: u8,
    radius: f32,
    width: f32,
    height: f32,
    pressed: Vec<u8>,
}

impl Hexes {
    pub fn new() -> Self {
        Hexes {
            note: 60,
            radius: 40.0,
            width: 0.0,
            height: 0.0,
            pressed: vec![],
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        let (w, h) = (width / (13.0 * HORIZ_X), height / (7.0));
        self.radius = if w <= h { w } else { h };
    }

    pub fn press(&mut self, note: u8) {
        self.pressed.push(note);
    }

    pub fn release(&mut self, note: u8) {
        self.pressed.retain(|n| *n != note);
    }

    pub fn release_all(&mut self) {
        self.pressed.clear();
    }

    pub fn base_note(&mut self, note: u8) {
        self.note = note;
    }


    fn note_pressed(&self, note: u8) -> bool {
        self.pressed.contains(&(self.note + note))
    }
}

impl Drawable for Hexes {
    fn draw<RT: RenderTarget>(&self, target: &mut RT, rs: &mut RenderStates) {
        let r = self.radius;
        let mut cs = CircleShape::new_init(r, 6).unwrap();
        cs.set_fill_color(&Color::new_rgb(0,0,0));
        cs.set_outline_thickness(2.5);
        cs.set_outline_color(&Color::new_rgb(0x26,0x32,0x38));
        cs.move2f(r * DIAG_X, 0.0);

        for _ in 0..2 {
            for i in 0..12 {
                let color = match (i % 6 > 1, self.note_pressed(2 * i + 1)) {
                    (false, false) => Color::new_rgb(0,0,0),
                    (true, false) => Color::new_rgb(0xFF,0xFF,0xFF),
                    (false, true) => Color::new_rgb(0x37,0x47,0x4F),
                    (true, true) => Color::new_rgb(0x78,0x90,0x9C),
                };

                cs.set_fill_color(&color);

                cs.draw(target, rs);
                cs.move2f(r * HORIZ_X, 0.0);
            }

            cs.move2f(r * (-12.0 * HORIZ_X - DIAG_X), r * DIAG_Y);

            for i in 0..13 {
                let color = match (i % 6 < 3, self.note_pressed(2 * i)) {
                    (false, false) => Color::new_rgb(0,0,0),
                    (true, false) => Color::new_rgb(0xFF,0xFF,0xFF),
                    (false, true) => Color::new_rgb(0x37,0x47,0x4F),
                    (true, true) => Color::new_rgb(0x78,0x90,0x9C),
                };

                cs.set_fill_color(&color);

                cs.draw(target, rs);
                cs.move2f(r * HORIZ_X, 0.0);
            }
            cs.move2f(r * (-13.0 * HORIZ_X + DIAG_X), r * DIAG_Y);
        }
    }
}
