use std::{collections::VecDeque, time::Instant};

use macroquad::prelude::*;

const EXPERT_WIDTH: usize = 30;
const EXPERT_HEIGHT: usize = 16;
const EXPERT_MINES: usize = 99;
// const EXPERT_MINES: usize = 20;
const WINDOW_SIZE_MULTIPLIER: f32 = 1.5;
const WINDOW_WIDTH: i32 = (505.0 * WINDOW_SIZE_MULTIPLIER) as i32;
const WINDOW_HEIGHT: i32 = (324.0 * WINDOW_SIZE_MULTIPLIER) as i32;
const TILE_START_X: f32 = 12.0 * WINDOW_SIZE_MULTIPLIER;
const TILE_START_Y: f32 = 55.0 * WINDOW_SIZE_MULTIPLIER;
const TILE_SIZE: f32 = 16.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_START_X: f32 = 239.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_START_Y: f32 = 15.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_SIZE: f32 = 26.0 * WINDOW_SIZE_MULTIPLIER;
const COUNTER_DIGIT_WIDTH: f32 = 13.0 * WINDOW_SIZE_MULTIPLIER;
const COUNTER_DIGIT_HEIGHT: f32 = 23.0 * WINDOW_SIZE_MULTIPLIER;
const MINES_COUNTER_START_X: f32 = 16.0 * WINDOW_SIZE_MULTIPLIER;
const MINES_COUNTER_START_Y: f32 = 17.0 * WINDOW_SIZE_MULTIPLIER;
const TIME_COUNTER_START_X: f32 = 446.0 * WINDOW_SIZE_MULTIPLIER;

fn window_conf() -> Conf {
    Conf {
        window_title: "Minesweeper".to_owned(),
        fullscreen: false,
        window_width: WINDOW_WIDTH,
        window_height: WINDOW_HEIGHT,
        window_resizable: false,
        ..Default::default()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
enum State {
    Playing,
    Dead,
    Won,
    NewGame,
}

#[derive(PartialEq, Default, Clone, Copy)]
enum TileState {
    #[default]
    Hidden,
    Revealed,
    Flagged,
}

#[derive(Default, Clone)]
struct Tile {
    state: TileState,
    neighbour_mines_count: u8,
    mine: bool,
}

struct Textures {
    background: Texture2D,
    tile: Texture2D,
    flag: Texture2D,
    mine: Texture2D,
    cross: Texture2D,
    smiley: Texture2D,
    smiley_open: Texture2D,
    smiley_dead: Texture2D,
    smiley_clicked: Texture2D,
    smiley_glasses: Texture2D,
    neighbour_mines: [Texture2D; 8],
    counter_digits: [Texture2D; 10],
}

struct Board {
    tiles: Vec<Vec<Tile>>,
    number_flagged: usize,
    state: State,
    start: Instant,
    elapsed: usize,
    unflagged_mines: Vec<(usize, usize)>,
}

impl Board {
    fn new(width: usize, height: usize) -> Self {
        Self {
            tiles: vec![vec![Tile::default(); height]; width],
            number_flagged: 0,
            state: State::NewGame,
            start: Instant::now(),
            elapsed: 0,
            unflagged_mines: vec![],
        }
    }

    fn update(&mut self, mouse_x: f32, mouse_y: f32) {
        if self.state == State::NewGame {
            let (col, row) = (
                (((mouse_x - TILE_START_X) / TILE_SIZE) as usize).min(EXPERT_WIDTH - 1),
                (((mouse_y - TILE_START_Y) / TILE_SIZE) as usize).min(EXPERT_HEIGHT - 1),
            );

            if is_mouse_button_pressed(MouseButton::Left)
                && hovering_tile(mouse_x, mouse_y, col, row)
            {
                self.start(col, row);
            }
        }

        for col in 0..self.tiles.len() {
            for row in 0..self.tiles[0].len() {
                if self.neighbour_mines(col, row) == 0
                    && !self.mine(col, row)
                    && self.revealed(col, row)
                {
                    self.reveal_empty_space_at(col, row)
                }
            }
        }

        if self.state == State::Playing {
            // self.player_game(mouse_x, mouse_y);
            self.computer_game();
        }
    }

    fn player_game(&mut self, mouse_x: f32, mouse_y: f32) {
        let (col, row) = (
            (((mouse_x - TILE_START_X) / TILE_SIZE) as usize).min(EXPERT_WIDTH - 1),
            (((mouse_y - TILE_START_Y) / TILE_SIZE) as usize).min(EXPERT_HEIGHT - 1),
        );

        if self.state == State::Playing
            && (is_mouse_button_released(MouseButton::Left)
                && is_mouse_button_released(MouseButton::Right)
                || is_mouse_button_down(MouseButton::Left)
                    && is_mouse_button_released(MouseButton::Right)
                || is_mouse_button_down(MouseButton::Right)
                    && is_mouse_button_released(MouseButton::Left)
                || is_mouse_button_released(MouseButton::Middle))
            && self.revealed(col, row)
            && surrounding_tiles(col, row)
                .into_iter()
                .filter(|(col, row)| self.mine(*col, *row))
                .count()
                == surrounding_tiles(col, row)
                    .into_iter()
                    .filter(|(col, row)| self.flagged(*col, *row))
                    .count()
        {
            let surrounding_tiles = surrounding_tiles(col, row);

            for (surrounding_tile_col, surrounding_tile_row) in surrounding_tiles {
                if self.mine(surrounding_tile_col, surrounding_tile_row) {
                    if !self.flagged(surrounding_tile_col, surrounding_tile_row) {
                        self.state = State::Dead;
                        self.reveal_all_mines();
                        self.unflagged_mines
                            .push((surrounding_tile_col, surrounding_tile_row))
                    } else {
                        continue;
                    }
                }

                if self.state == State::Playing {
                    self.tiles[surrounding_tile_col][surrounding_tile_row].state =
                        TileState::Revealed;
                }
            }
        } else if self.state == State::Playing
            && !self.revealed(col, row)
            && is_mouse_button_pressed(MouseButton::Right)
            && hovering_tile(mouse_x, mouse_y, col, row)
        {
            if self.flagged(col, row) {
                self.number_flagged -= 1;
                self.tiles[col][row].state = TileState::Hidden
            } else {
                self.number_flagged += 1;
                self.tiles[col][row].state = TileState::Flagged
            }
        } else if (self.state == State::Playing || self.state == State::NewGame)
            && is_mouse_button_pressed(MouseButton::Left)
            && hovering_tile(mouse_x, mouse_y, col, row)
            && !self.flagged(col, row)
        {
            self.tiles[col][row].state = TileState::Revealed;

            if self.mine(col, row) {
                self.state = State::Dead;
                self.reveal_all_mines();
                self.unflagged_mines.push((col, row));
            }
        }
    }

    fn computer_game(&mut self) {
        for col in 0..self.tiles.len() {
            for row in 0..self.tiles[0].len() {
                // only allowed to use revealed tiles
                if !self.revealed(col, row) {
                    continue;
                }

                let surrounding_tiles = surrounding_tiles(col, row);
                let neighbour_mines = self.neighbour_mines(col, row);
                let neighbour_flagged = surrounding_tiles
                    .iter()
                    .filter(|(col, row)| self.flagged(*col, *row))
                    .count() as u8;
                // let effective_neighbour_mines = neighbour_mines - neighbour_flagged;
                let neighbour_unrevealed = surrounding_tiles
                    .iter()
                    .filter(|(col, row)| !self.revealed(*col, *row))
                    .count() as u8;

                // trivial corner 1s etc
                if self.revealed(col, row) && neighbour_mines == neighbour_unrevealed {
                    for (col, row) in surrounding_tiles.iter() {
                        if !self.revealed(*col, *row) {
                            self.tiles[*col][*row].state = TileState::Flagged;
                        }
                    }
                }

                if self.satisfied(col, row) {
                    for (col, row) in surrounding_tiles.iter() {
                        if !self.flagged(*col, *row) {
                            self.tiles[*col][*row].state = TileState::Revealed;
                        }
                    }
                }

                self.solve_121s(col, row);
            }
        }
    }

    fn solve_121s(&mut self, col: usize, row: usize) {
        if self.effective_neighbour_mines(col, row) != 2 {
            return;
        }

        // has tiles to left and right
        if col + 1 < EXPERT_WIDTH && col != 0 {
            // 121 found
            if self.effective_neighbour_mines(col + 1, row) == 1
                && self.revealed(col + 1, row)
                && self.effective_neighbour_mines(col - 1, row) == 1
                && self.revealed(col - 1, row)
            {
                // has tiles below
                if row + 1 < EXPERT_HEIGHT {
                    // all 3 tiles below arent revealed
                    if !self.revealed(col, row + 1)
                        && !self.revealed(col + 1, row + 1)
                        && !self.revealed(col - 1, row + 1)
                    {
                        self.tiles[col - 1][row + 1].state = TileState::Flagged;
                        self.tiles[col + 1][row + 1].state = TileState::Flagged;
                        self.tiles[col][row + 1].state = TileState::Revealed;
                        println!("DOWN 121 solved {col} {row}");
                    }
                }
                // has tiles above
                if row != 0 {
                    // all 3 tiles above arent revealed
                    if !self.revealed(col, row - 1)
                        && !self.revealed(col + 1, row - 1)
                        && !self.revealed(col - 1, row - 1)
                    {
                        self.tiles[col - 1][row - 1].state = TileState::Flagged;
                        self.tiles[col + 1][row - 1].state = TileState::Flagged;
                        self.tiles[col][row - 1].state = TileState::Revealed;
                        println!("UP 121 solved {col} {row}");
                    }
                }
            }
        }
        // has tiles above and below
        if row + 1 < EXPERT_HEIGHT && row != 0 {
            // 121 found
            if self.effective_neighbour_mines(col, row + 1) == 1
                && self.revealed(col, row + 1)
                && self.effective_neighbour_mines(col, row - 1) == 1
                && self.revealed(col, row - 1)
            {
                // has tiles to left
                if col != 0 {
                    // all 3 tiles to left arent revealed
                    if !self.revealed(col - 1, row)
                        && !self.revealed(col - 1, row + 1)
                        && !self.revealed(col - 1, row - 1)
                    {
                        self.tiles[col - 1][row + 1].state = TileState::Flagged;
                        self.tiles[col - 1][row - 1].state = TileState::Flagged;
                        self.tiles[col - 1][row].state = TileState::Revealed;
                        println!("LEFT 121 solved {col} {row}");
                    }
                    // has tiles to right
                }
                if col + 1 < EXPERT_WIDTH {
                    // all 3 tiles to right arent revealed
                    if !self.revealed(col + 1, row)
                        && !self.revealed(col + 1, row + 1)
                        && !self.revealed(col + 1, row - 1)
                    {
                        self.tiles[col + 1][row + 1].state = TileState::Flagged;
                        self.tiles[col + 1][row - 1].state = TileState::Flagged;
                        self.tiles[col + 1][row].state = TileState::Revealed;
                        println!("RIGHT 121 solved {col} {row}");
                    }
                }
            }
        }
    }

    fn effective_neighbour_mines(&self, col: usize, row: usize) -> u8 {
        let neighbour_mines = self.neighbour_mines(col, row);
        let neighbour_flagged = surrounding_tiles(col, row)
            .iter()
            .filter(|(col, row)| self.flagged(*col, *row))
            .count() as u8;

        neighbour_mines - neighbour_flagged
    }

    fn satisfied(&self, col: usize, row: usize) -> bool {
        self.neighbour_mines(col, row)
            == surrounding_tiles(col, row)
                .into_iter()
                .filter(|(col, row)| self.flagged(*col, *row))
                .count() as u8
    }

    fn start(&mut self, start_col: usize, start_row: usize) {
        self.tiles = generate_fair_game(start_col, start_row);
        self.start = Instant::now();
        self.state = State::Playing;
        self.tiles[start_col][start_row].state = TileState::Revealed
    }

    fn revealed(&self, col: usize, row: usize) -> bool {
        self.tiles[col][row].state == TileState::Revealed
    }

    fn flagged(&self, col: usize, row: usize) -> bool {
        self.tiles[col][row].state == TileState::Flagged
    }

    fn mine(&self, col: usize, row: usize) -> bool {
        self.tiles[col][row].mine
    }

    fn neighbour_mines(&self, col: usize, row: usize) -> u8 {
        self.tiles[col][row].neighbour_mines_count
    }

    fn is_game_won(&self) -> bool {
        for row in self.tiles.iter() {
            for tile in row.iter() {
                if !tile.mine && tile.state != TileState::Revealed {
                    return false;
                }
            }
        }
        true
    }

    fn reveal_empty_space_at(&mut self, col: usize, row: usize) {
        let mut queue = VecDeque::<(usize, usize)>::new();
        queue.push_back((col, row));

        while !queue.is_empty() {
            let (current_col, current_row) = queue.pop_back().unwrap();

            for (neighbour_col, neighbour_row) in surrounding_tiles(current_col, current_row) {
                let mut neighbour_tile = &mut self.tiles[neighbour_col][neighbour_row];
                if neighbour_tile.neighbour_mines_count == 0
                    && neighbour_tile.state != TileState::Revealed
                    && !neighbour_tile.mine
                {
                    queue.push_back((neighbour_col, neighbour_row))
                }

                neighbour_tile.state = TileState::Revealed;
            }
        }
    }

    fn reveal_all_mines(&mut self) {
        for col in self.tiles.iter_mut() {
            for tile in col.iter_mut() {
                if tile.mine && tile.state != TileState::Flagged {
                    tile.state = TileState::Revealed
                }
            }
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let textures = Textures {
        background: load_texture!("../assets/background.png"),
        tile: load_texture!("../assets/tile.png"),
        flag: load_texture!("../assets/flag.png"),
        mine: load_texture!("../assets/mine.png"),
        cross: load_texture!("../assets/cross.png"),
        smiley: load_texture!("../assets/smiley.png"),
        smiley_open: load_texture!("../assets/smiley_open.png"),
        smiley_dead: load_texture!("../assets/smiley_dead.png"),
        smiley_clicked: load_texture!("../assets/smiley_clicked.png"),
        smiley_glasses: load_texture!("../assets/smiley_glasses.png"),
        neighbour_mines: [
            load_texture!("../assets/1.png"),
            load_texture!("../assets/2.png"),
            load_texture!("../assets/3.png"),
            load_texture!("../assets/4.png"),
            load_texture!("../assets/5.png"),
            load_texture!("../assets/6.png"),
            load_texture!("../assets/7.png"),
            load_texture!("../assets/8.png"),
        ],
        counter_digits: [
            load_texture!("../assets/0_counter.png"),
            load_texture!("../assets/1_counter.png"),
            load_texture!("../assets/2_counter.png"),
            load_texture!("../assets/3_counter.png"),
            load_texture!("../assets/4_counter.png"),
            load_texture!("../assets/5_counter.png"),
            load_texture!("../assets/6_counter.png"),
            load_texture!("../assets/7_counter.png"),
            load_texture!("../assets/8_counter.png"),
            load_texture!("../assets/9_counter.png"),
        ],
    };

    let mut board = Board::new(EXPERT_WIDTH, EXPERT_HEIGHT);

    loop {
        clear_background(Color::from_rgba(192, 192, 192, 255));
        let (mouse_x, mouse_y) = mouse_position();

        draw_texture_with_size(
            &textures.background,
            0.0,
            0.0,
            screen_width(),
            screen_height(),
        );

        draw_counter(
            board.elapsed.min(999),
            TIME_COUNTER_START_X,
            MINES_COUNTER_START_Y,
            &textures.counter_digits,
        );

        draw_counter(
            (EXPERT_MINES - board.number_flagged).max(0),
            MINES_COUNTER_START_X,
            MINES_COUNTER_START_Y,
            &textures.counter_digits,
        );

        if board.state == State::Playing && board.is_game_won() {
            board.state = State::Won;
        }

        if hovering_square(
            mouse_x,
            mouse_y,
            SMILEY_START_X,
            SMILEY_START_Y,
            SMILEY_SIZE,
        ) {
            if is_mouse_button_pressed(MouseButton::Left) {
                board = Board::new(EXPERT_WIDTH, EXPERT_HEIGHT);
            }
        }

        let smiley_texture = if is_mouse_button_down(MouseButton::Left) {
            if hovering_square(
                mouse_x,
                mouse_y,
                SMILEY_START_X,
                SMILEY_START_Y,
                SMILEY_SIZE,
            ) {
                &textures.smiley_clicked
            } else {
                &textures.smiley_open
            }
        } else if board.state == State::Dead {
            &textures.smiley_dead
        } else if board.state == State::Won {
            &textures.smiley_glasses
        } else {
            &textures.smiley
        };

        draw_texture_with_size(
            smiley_texture,
            SMILEY_START_X,
            SMILEY_START_Y,
            SMILEY_SIZE,
            SMILEY_SIZE,
        );

        board.update(mouse_x, mouse_y);

        draw_tiles(&mut board, &textures);

        if board.state == State::Playing {
            board.elapsed = board.start.elapsed().as_secs() as usize
        }

        next_frame().await
    }
}

fn draw_tiles(board: &mut Board, textures: &Textures) {
    for row in 0..EXPERT_HEIGHT {
        for col in 0..EXPERT_WIDTH {
            if board.state == State::Dead && board.mine(col, row) {
                if board.unflagged_mines.contains(&(col, row)) {
                    draw_rectangle(
                        TILE_START_X + 1.0 + col as f32 * TILE_SIZE,
                        TILE_START_Y + 1.0 + row as f32 * TILE_SIZE,
                        TILE_SIZE - 1.0,
                        TILE_SIZE - 1.0,
                        RED,
                    )
                }

                draw_at_tile(&textures.mine, col, row)
            }

            if board.revealed(col, row) {
                let neighbour_mines_count = board.tiles[col][row].neighbour_mines_count;

                if neighbour_mines_count != 0 && !board.mine(col, row) && !board.flagged(col, row) {
                    draw_at_tile(
                        &textures.neighbour_mines[neighbour_mines_count as usize - 1],
                        col,
                        row,
                    )
                }
            }

            if board.state == State::Dead && board.flagged(col, row) && !board.mine(col, row) {
                board.tiles[col][row].state = TileState::Revealed;
                draw_at_tile(&textures.mine, col, row);
                draw_at_tile(&textures.cross, col, row);
            } else {
                if !board.revealed(col, row) {
                    draw_at_tile(&textures.tile, col, row);

                    if board.flagged(col, row) {
                        draw_at_tile(&textures.flag, col, row)
                    }
                }
            }
        }
    }
}

#[macro_export]
macro_rules! load_texture {
    ( $path:tt ) => {
        Texture2D::from_file_with_format(include_bytes!($path), None)
    };
}

fn draw_at_tile(texture: &Texture2D, col: usize, row: usize) {
    draw_texture_with_size(
        texture,
        TILE_START_X + col as f32 * TILE_SIZE,
        TILE_START_Y + row as f32 * TILE_SIZE,
        TILE_SIZE,
        TILE_SIZE,
    )
}

fn draw_counter(number: usize, x: f32, y: f32, textures: &[Texture2D]) {
    for i in 0..3 {
        draw_texture_with_size(
            &textures[(number / 10_usize.pow(2 - i)) % 10],
            x + COUNTER_DIGIT_WIDTH * i as f32,
            y,
            COUNTER_DIGIT_WIDTH,
            COUNTER_DIGIT_HEIGHT,
        );
    }
}

fn draw_texture_with_size(texture: &Texture2D, x: f32, y: f32, width: f32, height: f32) {
    draw_texture_ex(
        texture,
        x,
        y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    )
}

fn generate_fair_game(start_col: usize, start_row: usize) -> Vec<Vec<Tile>> {
    loop {
        let tiles = generate_game();

        if tiles[start_col][start_row].neighbour_mines_count == 0
            && !tiles[start_col][start_row].mine
        {
            return tiles;
        }
    }
}

fn generate_game() -> Vec<Vec<Tile>> {
    let mut tiles = vec![vec![Tile::default(); EXPERT_HEIGHT]; EXPERT_WIDTH];

    let mut placed_mines = 0;

    while placed_mines < EXPERT_MINES {
        let col = fastrand::usize(0..EXPERT_WIDTH);
        let row = fastrand::usize(0..EXPERT_HEIGHT);

        if !tiles[col][row].mine {
            tiles[col][row].mine = true;
            placed_mines += 1;
        }
    }

    for col in 0..tiles.len() {
        for row in 0..tiles[0].len() {
            tiles[col][row].neighbour_mines_count = surrounding_tiles(col, row)
                .into_iter()
                .filter(|(col, row)| tiles[*col][*row].mine)
                .count() as u8
        }
    }

    tiles
}

fn hovering_square(mouse_x: f32, mouse_y: f32, start_x: f32, start_y: f32, size: f32) -> bool {
    mouse_x >= start_x
        && mouse_x <= start_x + size
        && mouse_y >= start_y
        && mouse_y <= start_y + size
}

fn hovering_tile(mouse_x: f32, mouse_y: f32, col: usize, row: usize) -> bool {
    let tile_x = TILE_START_X + col as f32 * TILE_SIZE;
    let tile_y = TILE_START_Y + row as f32 * TILE_SIZE;

    hovering_square(mouse_x, mouse_y, tile_x, tile_y, TILE_SIZE)
}

fn surrounding_bounds(col: usize, row: usize) -> ((usize, usize), (usize, usize)) {
    let min_row = if row == 0 { 0 } else { row - 1 };
    let max_row = if row == EXPERT_HEIGHT - 1 {
        EXPERT_HEIGHT - 1
    } else {
        row + 1
    };

    let min_col = if col == 0 { 0 } else { col - 1 };
    let max_col = if col == EXPERT_WIDTH - 1 {
        EXPERT_WIDTH - 1
    } else {
        col + 1
    };

    ((min_col, min_row), (max_col, max_row))
}

fn surrounding_tiles(col: usize, row: usize) -> Vec<(usize, usize)> {
    let ((min_col, min_row), (max_col, max_row)) = surrounding_bounds(col, row);

    let mut tiles = Vec::<(usize, usize)>::new();

    for col_2 in min_col..=max_col {
        for row_2 in min_row..=max_row {
            if col_2 == col && row_2 == row {
                continue;
            }

            tiles.push((col_2, row_2))
        }
    }

    tiles
}
