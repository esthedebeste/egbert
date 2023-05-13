use arrayvec::ArrayVec;
use rand::{rngs::StdRng, Rng};

use crate::wfc::{Collapse, WaveFunctionCollapse};

#[derive(PartialEq, Clone, Copy)]
pub struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Rgb { r, g, b }
    }

    pub fn to_hex(self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Direction {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Top,
    Bottom,
    Left,
    Right,
}

impl Direction {
    fn random(rng: &mut StdRng) -> Self {
        match rng.gen_range(0..8) {
            0 => Direction::TopLeft,
            1 => Direction::TopRight,
            2 => Direction::BottomLeft,
            3 => Direction::BottomRight,
            4 => Direction::Top,
            5 => Direction::Bottom,
            6 => Direction::Left,
            7 => Direction::Right,
            _ => unreachable!(),
        }
    }
    fn get_offset(&self) -> (i32, i32) {
        match self {
            Direction::TopLeft => (-1, -1),
            Direction::TopRight => (1, -1),
            Direction::BottomLeft => (-1, 1),
            Direction::BottomRight => (1, 1),
            Direction::Top => (0, -1),
            Direction::Bottom => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }
    fn emoji(&self) -> &'static str {
        match self {
            Direction::Right => "➡️",
            Direction::Left => "⬅️",
            Direction::Top => "⬆️",
            Direction::Bottom => "⬇️",
            Direction::TopLeft => "↖️",
            Direction::TopRight => "↗️",
            Direction::BottomLeft => "↙️",
            Direction::BottomRight => "↘️",
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Color {
    pub name: &'static str,
    pub rgb: Rgb,
}

pub const COLORS: &[Color] = &[
    Color {
        name: "◻️",
        rgb: Rgb::new(210, 210, 210),
    },
    Color {
        name: "🟥",
        rgb: Rgb::new(180, 60, 60),
    },
    Color {
        name: "🟩",
        rgb: Rgb::new(60, 180, 60),
    },
    Color {
        name: "🟦",
        rgb: Rgb::new(60, 60, 180),
    },
    Color {
        name: "🟨",
        rgb: Rgb::new(180, 180, 60),
    },
    // Color {
    //     name: "🟧",
    //     rgb: Rgb::new(180, 120, 60),
    // },
    Color {
        name: "🟪",
        rgb: Rgb::new(180, 60, 180),
    },
];
const COLOR_COUNT: usize = 3; // limit to 3 colors
const RULE_COUNT: usize = 3;

#[derive(PartialEq, Clone)]
pub struct Rule {
    pub cols: (usize, usize),
    pub dir: Direction,
}

impl Rule {
    pub fn to_emoji(&self, colors: &[&Color]) -> String {
        format!(
            "{} ❌{} {}",
            colors[self.cols.0].name,
            self.dir.emoji(),
            colors[self.cols.1].name
        )
    }
}

pub fn generate(
    width: usize,
    height: usize,
    rng: &mut StdRng,
) -> (Vec<Rule>, Vec<&'static Color>, Vec<Rgb>) {
    let mut colors = vec![];
    for _ in 0..COLOR_COUNT {
        let mut col;
        loop {
            col = rng.gen_range(0..COLORS.len());
            if !colors.contains(&&COLORS[col]) {
                break;
            }
        }
        colors.push(&COLORS[col]);
    }
    let mut rules = vec![];
    for _ in 0..RULE_COUNT {
        rules.push(Rule {
            cols: (rng.gen_range(0..COLOR_COUNT), rng.gen_range(0..COLOR_COUNT)),
            dir: Direction::random(rng),
        });
    }
    let mut wfc = WaveFunctionCollapse::<COLOR_COUNT, { COLOR_COUNT * RULE_COUNT }, _>::new(
        width,
        height,
        rng,
        |x: usize, y: usize, picked: usize| {
            let mut collapse = ArrayVec::new();
            for rule in &rules {
                if picked == rule.cols.0 {
                    collapse.push(Collapse {
                        x: (x as i32 - rule.dir.get_offset().0) as usize,
                        y: (y as i32 - rule.dir.get_offset().1) as usize,
                        opt: rule.cols.1,
                    });
                }
                if picked == rule.cols.1 {
                    collapse.push(Collapse {
                        x: (x as i32 - rule.dir.get_offset().0) as usize,
                        y: (y as i32 - rule.dir.get_offset().1) as usize,
                        opt: rule.cols.0,
                    });
                }
            }
            collapse
        },
    );

    let mut count = 0;
    loop {
        wfc.init();
        while !wfc.is_done() {
            wfc.step();
        }
        // keep going until we find a valid image.
        count += 1;
        if count > 2000 {
            return generate(width, height, rng); // give up on this ruleset
        }
        if !wfc.broken() {
            break;
        }
    }
    let mut image = vec![];
    for y in 0..height {
        for x in 0..width {
            let cell = wfc.at(x, y);
            image.push(if let Some(i) = cell.first_active() {
                colors[i].rgb
            } else {
                Rgb::new(255, 0, 0)
            });
        }
    }
    (rules, colors, image)
}
