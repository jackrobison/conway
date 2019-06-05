extern crate rand;
extern crate termion;

use std::io::{Write, stdout};
use std::{thread, time};
use rand::random;
use termion::terminal_size;


#[derive(Clone)]
pub struct LineLand {
    pub size: usize,
    pub particles: Vec<bool>,
}

#[derive(Clone)]
pub struct FlatLand {
    pub size: usize,
    pub row_length: usize,
    pub rows: Vec<LineLand>,
}

#[derive(Copy, Clone)]
pub struct Neighbor {
    pub x: usize,
    pub y: usize,
    pub live: bool,
}

#[derive(Copy, Clone)]
pub struct Neighborhood {
    pub neighbors: [[Neighbor; 3]; 3],
    pub live_neighbors: u8,
}

impl Neighbor {
    pub fn new(live: bool, x: usize, y: usize) -> Neighbor {
        return Neighbor { x, y, live };
    }
}

impl LineLand {
    pub fn new(size: usize) -> LineLand {
        let mut particles: Vec<bool> = Vec::with_capacity(size);
        for _ in 0..size {
            particles.push(false);
        }
        return LineLand { size, particles };
    }
}

impl FlatLand {
    pub fn new(y_size: usize, x_size: usize) -> FlatLand {
        let mut rows = Vec::with_capacity(y_size);
        for i in 0..y_size {
            rows.push(LineLand::new(x_size));
        }
        let mut new_world = FlatLand { size: y_size, rows, row_length: x_size };
        new_world.randomize();
        return new_world;
    }

    pub fn get_particle(&mut self, x: usize, y: usize) -> bool {
        return self.rows[y].particles[x];
    }

    pub fn set_particle(&mut self, x: usize, y: usize, value: bool) {
        self.rows[y].particles[x] = value;
    }

    pub fn randomize(&mut self) {
        for x in 0..self.row_length {
            for y in 0..self.size {
                let v = rand::random();
                self.set_particle(x, y, v);
            }
        }
    }

    fn get_neighborhood(&mut self, x: usize, y: usize) -> Neighborhood {
        let lxdim = if x > 0 { x - 1 } else { self.row_length - 1 };
        let uxdim = if x + 1 == self.row_length { 0 } else { x + 1 };
        let lydim = if y > 0 { y - 1 } else { self.size - 1 };
        let uydim = if y + 1 == self.size { 0 } else { y + 1 };

        let neighbors = [
            [Neighbor::new(self.get_particle(lxdim, lydim), lxdim, lydim), Neighbor::new(self.get_particle(x, lydim), x, lydim), Neighbor::new(self.get_particle(uxdim, lydim), uxdim, lydim)],
            [Neighbor::new(self.get_particle(lxdim, y), lxdim, y), Neighbor::new(self.get_particle(x, y), x, y), Neighbor::new(self.get_particle(uxdim, y), uxdim, y)],
            [Neighbor::new(self.get_particle(lxdim, uydim), lxdim, uydim), Neighbor::new(self.get_particle(x, uydim), x, uydim), Neighbor::new(self.get_particle(uxdim, uydim), uxdim, uydim)]
        ];

        let mut live_neighbors = 0;
        for row in neighbors.iter() {
            for neighbor in row.iter() {
                if neighbor.live {
                    if !(neighbor.x == x && neighbor.y == y) {
                        live_neighbors += 1;
                    }
                }
            }
        }
        return Neighborhood { neighbors, live_neighbors };
    }

    pub fn tick(&mut self) {
        let mut new_state = FlatLand::new(self.size, self.row_length);
        for x in 0..self.row_length {
            for y in 0..self.size {
                let neighbors = self.get_neighborhood(x, y);
                if neighbors.live_neighbors == 3
                {
                    new_state.set_particle(x, y, true);
                } else if neighbors.live_neighbors == 2 && self.get_particle(x, y) {
                    new_state.set_particle(x, y, true);
                } else {
                    new_state.set_particle(x, y, false);
                }
            }
        }
        self.rows = new_state.rows;
    }
}


fn main() {
    let thirty_ms = time::Duration::from_millis(30);
    let mut cont = true;
    let (x_size2, y_size2) = termion::terminal_size().unwrap();
    let y_size = y_size2 - 1;
    let x_size = x_size2 - 1;

    let mut world = FlatLand::new(y_size as usize, x_size as usize);
    let mut prev_world = FlatLand::new(y_size as usize, x_size as usize);
    let mut second_prev_world = FlatLand::new(y_size as usize, x_size as usize);

    prev_world.rows = world.rows.clone();

    let mut stdout = stdout();
    let mut count = 0;
    let mut converged = false;
    write!(stdout, "{}", termion::cursor::Hide).unwrap();

    while !converged {
        count += 1;
        write!(stdout, "{}{}", termion::cursor::Goto(1, 1), termion::clear::CurrentLine);
        write!(stdout, "{}generation #{}", termion::cursor::Goto(x_size / 2 - 5, 1), count);
        for y in 0..y_size {
            let mut row = String::with_capacity(x_size as usize);
            for x in 0..x_size {
                let c = if world.get_particle(x as usize, y as usize) { '█' } else { '░' };
                row.push(c);
            }
            write!(stdout, "{}{}", termion::cursor::Goto(1, y as u16 + 2), row);
        }

        stdout.flush().unwrap();
        world.tick();

        converged = true;
        for y in 0..y_size {
            for x in 0..x_size {
                if world.get_particle(x as usize, y as usize) != second_prev_world.get_particle(x as usize, y as usize) {
                    converged = false;
                    break;
                }
            }
            if !converged {
                break;
            }
        }
        second_prev_world.rows = prev_world.rows.clone();
        prev_world.rows = world.rows.clone();
        thread::sleep(thirty_ms);
    }
}
