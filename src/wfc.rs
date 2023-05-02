// largely a rewrite of https://github.com/esthedebeste/WaveFunctionCollapse/blob/main/src/wfc.cpp

use rand::{rngs::StdRng, Rng};

#[derive(Clone, Copy)]
pub struct Cell {
    pub options: u32, // bitfield
}

impl Cell {
    pub fn new_all_active(count: usize) -> Cell {
        Cell {
            options: (1 << count) - 1,
        }
    }
    pub fn activate(&mut self, opt: usize) {
        self.options |= 1 << opt;
    }
    pub fn deactivate(&mut self, opt: usize) {
        self.options &= !(1 << opt);
    }
    pub fn active_options(&self) -> u32 {
        self.options.count_ones()
    }
    pub fn first_active(&self) -> Option<usize> {
        if self.options == 0 {
            return None;
        }
        Some(self.options.trailing_zeros() as usize)
    }
    pub fn enabled(&self, opt: usize) -> bool {
        self.options & (1 << opt) != 0
    }
    pub fn random_active(&self, rng: &mut StdRng) -> usize {
        let mut active = self.options;
        let mut count = active.count_ones();
        let mut opt = rng.gen_range(0..count);
        while opt > 0 {
            active &= active - 1;
            count -= 1;
            opt -= 1;
        }
        active.trailing_zeros() as usize
    }
}

pub struct Collapse {
    pub x: usize,
    pub y: usize,
    pub opt: usize,
}

pub struct WaveFunctionCollapse<'a, Collapser: Fn(usize, usize, usize) -> Vec<Collapse>> {
    pub width: usize,
    pub height: usize,
    pub opt_count: usize,
    pub cells: Vec<Cell>,
    pub rng: &'a mut StdRng,
    pub collapser: Collapser,
}
impl<'a, C: Fn(usize, usize, usize) -> Vec<Collapse>> WaveFunctionCollapse<'a, C> {
    pub fn new(
        width: usize,
        height: usize,
        opt_count: usize,
        rng: &'a mut StdRng,
        collapser: C,
    ) -> Self {
        if opt_count > 32 {
            panic!("Too many options! (max 32)");
        }
        WaveFunctionCollapse {
            width,
            height,
            opt_count,
            cells: vec![Cell::new_all_active(opt_count); width * height],
            rng,
            collapser,
        }
    }
    pub fn init(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                self.cells[y * self.width + x] = Cell::new_all_active(self.opt_count);
            }
        }
    }
    pub fn at(&mut self, x: usize, y: usize) -> &mut Cell {
        &mut self.cells[y * self.width + x]
    }
    pub fn remove_option(&mut self, x: usize, y: usize, opt: usize) {
        if x >= self.width || y >= self.height {
            return;
        }
        let cell = &mut self.cells[y * self.width + x];
        if cell.enabled(opt) {
            cell.deactivate(opt);
            if cell.active_options() == 1 {
                for collapse in (self.collapser)(x, y, cell.first_active().unwrap()) {
                    self.remove_option(collapse.x, collapse.y, collapse.opt);
                }
            }
        }
    }
    pub fn is_done(&self) -> bool {
        for cell in &self.cells {
            if cell.active_options() > 1 {
                return false;
            }
        }
        true
    }
    pub fn broken(&self) -> bool {
        for cell in &self.cells {
            if cell.active_options() == 0 {
                return true;
            }
        }
        false
    }
    pub fn step(&mut self) {
        let mut lowest_cell: Option<usize> = None;
        let mut min_opt = self.opt_count + 1;
        for (i, cell) in &mut self.cells.iter().enumerate() {
            let opts = cell.active_options() as usize;
            if opts <= 1 {
                continue;
            }
            if opts < min_opt {
                min_opt = opts;
                lowest_cell = Some(i);
            }
        }
        if lowest_cell.is_none() {
            return;
        }

        let cell_i = lowest_cell.unwrap();
        let cell = &mut self.cells[cell_i];
        let picked = cell.random_active(self.rng);
        cell.options = 0;
        cell.activate(picked);

        let r = (self.collapser)(cell_i % self.width, cell_i / self.width, picked);
        for collapse in r {
            self.remove_option(collapse.x, collapse.y, collapse.opt);
        }
    }
}
