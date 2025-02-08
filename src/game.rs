#![allow(dead_code)]

use std::{
    collections::{HashSet, VecDeque},
    ops::Add,
};

use anyhow::{Result, anyhow};
use ruscii::{drawing::Pencil, keyboard::Key, spatial::Vec2};
use thiserror::Error;

pub type Vec3 = (isize, isize, isize);

fn contains(coord: Vec3, dimensions: Vec3) -> bool {
    let (x, y, z) = coord;
    let (mx, my, mz) = dimensions;
    (0..mx).contains(&x) && (0..my).contains(&y) && (0..mz).contains(&z)
}
fn coord_to_screen(coord: Vec3) -> Vec2 {
    let (x, y, z) = coord;
    let screen_x = (x - y) * 2;
    let screen_y = (x + y) - z;
    Vec2::xy(screen_x, screen_y)
}

pub trait Draw {
    fn draw(&self, pencil: &mut Pencil);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Void,  // Permettra de faire des niveaux pas forcément rectangulaires
    Empty, // Juste une case vide, là où Void c'est vraiment du rien
    Block, // Un mur ou un sol
    Food,
}

impl Cell {
    fn to_char(self) -> char {
        match self {
            Cell::Void => 'V',
            Cell::Empty => ' ',
            Cell::Block => 'W',
            Cell::Food => 'F',
        }
    }
}

#[derive(Debug)]
pub struct Grid {
    // le vecteur se parcourt de tous les x, puis incrément y, puis incrément z après avoir fait la première couche
    cells: Vec<Cell>, // Vecteur comme ça on pourrait faire des niveaux dont la taille change en cours de route par ex
    dimensions: Vec3,
}

impl Grid {
    pub fn new(dimensions: Vec3, cells: Vec<Cell>) -> Self {
        Self { cells, dimensions }
    }

    pub fn empty((mx, my, mz): Vec3) -> Self {
        Self {
            cells: vec![Cell::Empty; (mx * my * mz) as usize],
            dimensions: (mx, my, mz),
        }
    }

    pub fn get(&self, (x, y, z): Vec3) -> Option<Cell> {
        if contains((x, y, z), self.dimensions)
            && let cell = self.cells[self.coord_to_index((x, y, z))]
            && cell != Cell::Void
        {
            Some(cell)
        } else {
            None
        }
    }

    pub fn set(&mut self, coord: Vec3, cell: Cell) -> Result<()> {
        if contains(coord, self.dimensions) {
            let idx = self.coord_to_index(coord);
            self.cells[idx] = cell;
            Ok(())
        } else {
            Err(anyhow!("Coordonnées hors de la grille"))
        }
    }

    // UTILS

    // does not check if the coord is in the grid
    fn coord_to_index(&self, (x, y, z): Vec3) -> usize {
        let (mx, my, _) = self.dimensions;
        (z * my * mx + y * mx + x) as usize
    }
    fn index_to_coord(&self, idx: usize) -> Vec3 {
        let idx = idx as isize;
        let (mx, my, _) = self.dimensions;
        let x = idx % mx;
        let y = (idx / mx) % my;
        let z = idx / (mx * my);
        (x, y, z)
    }
}

impl Draw for Grid {
    fn draw(&self, pencil: &mut Pencil) {
        for (idx, &cell) in self.cells.iter().enumerate() {
            let coord = self.index_to_coord(idx);
            let screen_vec = coord_to_screen(coord);
            let c = cell.to_char();
            pencil.draw_char(c, screen_vec);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    West,
    East,
    Up,
    Down,
    None,
}

impl From<Key> for Direction {
    fn from(key: Key) -> Self {
        match key {
            Key::Up => Direction::North,
            Key::Down => Direction::South,
            Key::Left => Direction::West,
            Key::Right => Direction::East,
            Key::Space => Direction::Up, // TODO : pour tester le saut voir
            _ => Direction::None,
        }
    }
}

impl Add<Direction> for Vec3 {
    type Output = Vec3;

    fn add(self, dir: Direction) -> Vec3 {
        let (x, y, z) = self;
        let (dx, dy, dz) = match dir {
            Direction::North => (0, -1, 0),
            Direction::South => (0, 1, 0),
            Direction::West => (-1, 0, 0),
            Direction::East => (1, 0, 0),
            Direction::Up => (0, 0, 1),
            Direction::Down => (0, 0, -1),
            Direction::None => (0, 0, 0),
        };
        (x + dx, y + dy, z + dz)
    }
}

#[derive(Debug)]
// Le snake peut se téléporter mais ça peut être cool d'avoir des upgrades au snake ou genre des téléporteurs sur la map
// Il peut aussi se passer sur lui-même mais genre imagine foutre des ponts sur la map
struct Snake {
    direction: Direction,
    body: VecDeque<Vec3>,
}

impl Snake {
    fn new(pos: Vec3) -> Self {
        let mut body = VecDeque::new();
        body.push_back(pos);
        Self {
            direction: Direction::None,
            body,
        }
    }

    fn head(&self) -> &Vec3 {
        self.body
            .front()
            .expect("C'est pas normal un serpent sans queue ni tête (pun intended)")
    }

    fn move_to(&mut self, target: Vec3, growing: bool) {
        self.body.push_front(target);
        if !growing {
            self.body.pop_back();
        }
    }

    fn is_superlapping(&self) -> bool {
        let mut seen = HashSet::new();
        self.body.iter().any(|&coord| !seen.insert(coord))
    }
}

impl Draw for Snake {
    fn draw(&self, pencil: &mut Pencil) {
        for &coord in &self.body {
            let screen_vec = coord_to_screen(coord);
            pencil.draw_char('S', screen_vec);
        }
    }
}

#[derive(Debug)]
pub struct GameState {
    grid: Grid,
    snake: Snake,
}

impl GameState {
    pub fn new(starting_pos: Vec3, level: Grid) -> Self {
        Self {
            grid: level,
            snake: Snake::new(starting_pos),
        }
    }

    pub fn update(&mut self, dir_held_player: Direction) -> Result<()> {
        let dir = if dir_held_player == Direction::None {
            self.snake.direction
        } else {
            dir_held_player
        };
        self.snake.direction = dir;

        let next_head = *self.snake.head() + dir;
        if let Some(cell) = self.grid.get(next_head)
            && cell != Cell::Block
        {
            // falls if not on a block
            let cell = match self.grid.get(next_head + Direction::Down) {
                Some(Cell::Block) => {
                    cell // if there is a block under the next head, we can move
                }
                Some(c) => {
                    c // if not we fall if there is somewhere to fall
                }
                None => {
                    // otherwise we die falling out of the map
                    return Err(GameError::SnakeFell {
                        head: *self.snake.head(),
                        attempted_move: next_head,
                    }
                    .into());
                }
            };

            // Faudrait changer ça si on ajoute des upgrades pour traverser les murs par exemple
            match cell {
                Cell::Empty => {
                    self.snake.move_to(next_head, false);
                }
                Cell::Food => {
                    self.snake.move_to(next_head, true);
                    self.grid.set(next_head, Cell::Empty)?;
                }
                _ => unreachable!(),
            }
        } else {
            return Err(GameError::SnakeCollision {
                head: *self.snake.head(),
                attempted_move: next_head,
            }
            .into());
        }
        if self.snake.is_superlapping() {
            return Err(GameError::SnakeCannibalism {
                head: *self.snake.head(),
                attempted_move: next_head,
            }
            .into());
        }
        Ok(())
    }
}

impl Draw for GameState {
    fn draw(&self, pencil: &mut Pencil) {
        self.grid.draw(pencil);
        self.snake.draw(pencil);
    }
}

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)] // passque y'aura maybe d'autres erreurs que juste le serpent
enum GameError {
    #[error("Snake collision when attempting to move head from {head:?} to {attempted_move:?}")]
    SnakeCollision { head: Vec3, attempted_move: Vec3 },
    #[error("Snake at {head:?} tried to eat itself at {attempted_move:?}")]
    SnakeCannibalism { head: Vec3, attempted_move: Vec3 },
    #[error("Snake fell at {attempted_move:?} from {head:?}")]
    SnakeFell { head: Vec3, attempted_move: Vec3 },
}
