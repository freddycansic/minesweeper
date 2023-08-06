use std::{collections::VecDeque, time::Instant};

use macroquad::prelude::*;

const EXPERT_WIDTH: usize = 30;
const EXPERT_HEIGHT: usize = 16;
// const EXPERT_MINES: usize = 99;
const EXPERT_MINES: usize = 10;
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

#[derive(PartialEq, Debug)]
enum State {
    Playing,
    Dead,
    Won,
    NewGame,
}

#[derive(PartialEq, Default, Clone)]
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

#[macroquad::main(window_conf)]
async fn main() {
    let background =
        Texture2D::from_file_with_format(include_bytes!("../assets/background.png"), None);
    let tile = Texture2D::from_file_with_format(include_bytes!("../assets/tile.png"), None);
    let flag = Texture2D::from_file_with_format(include_bytes!("../assets/flag.png"), None);
    let mine = Texture2D::from_file_with_format(include_bytes!("../assets/mine.png"), None);
    let cross = Texture2D::from_file_with_format(include_bytes!("../assets/cross.png"), None);
    let smiley = Texture2D::from_file_with_format(include_bytes!("../assets/smiley.png"), None);
    let smiley_open =
        Texture2D::from_file_with_format(include_bytes!("../assets/smiley_open.png"), None);
    let smiley_dead =
        Texture2D::from_file_with_format(include_bytes!("../assets/smiley_dead.png"), None);
    let smiley_clicked =
        Texture2D::from_file_with_format(include_bytes!("../assets/smiley_clicked.png"), None);
    let smiley_glasses =
        Texture2D::from_file_with_format(include_bytes!("../assets/smiley_glasses.png"), None);

    let neighbour_mines_textures = [
        Texture2D::from_file_with_format(include_bytes!("../assets/1.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/2.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/3.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/4.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/5.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/6.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/7.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/8.png"), None),
    ];

    let counters_textures = [
        Texture2D::from_file_with_format(include_bytes!("../assets/0_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/1_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/2_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/3_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/4_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/5_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/6_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/7_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/8_counter.png"), None),
        Texture2D::from_file_with_format(include_bytes!("../assets/9_counter.png"), None),
    ];

    let mut tiles = vec![vec![Tile::default(); EXPERT_HEIGHT]; EXPERT_WIDTH];

    let mut unflagged_mines = Vec::<(usize, usize)>::new();
    let mut number_flagged = 0;
    let mut start = Instant::now();
    let mut elapsed = 0;

    let mut state = State::NewGame;

    loop {
        clear_background(Color::from_rgba(192, 192, 192, 255));
        let (mouse_x, mouse_y) = mouse_position();

        draw_texture_with_size(&background, 0.0, 0.0, screen_width(), screen_height());

        draw_counter(
            elapsed.min(999),
            TIME_COUNTER_START_X,
            MINES_COUNTER_START_Y,
            &counters_textures,
        );

        draw_counter(
            EXPERT_MINES - number_flagged,
            MINES_COUNTER_START_X,
            MINES_COUNTER_START_Y,
            &counters_textures,
        );

        if state == State::Playing && is_game_won(&tiles) {
            state = State::Won;
        }

        if is_mouse_button_pressed(MouseButton::Left)
            && hovering_square(
                mouse_x,
                mouse_y,
                SMILEY_START_X,
                SMILEY_START_Y,
                SMILEY_SIZE,
            )
        {
            tiles = vec![vec![Tile::default(); EXPERT_HEIGHT]; EXPERT_WIDTH];
            unflagged_mines.clear();
            number_flagged = 0;
            elapsed = 0;

            state = State::NewGame;
        }

        let smiley_texture = if is_mouse_button_down(MouseButton::Left) {
            if hovering_square(
                mouse_x,
                mouse_y,
                SMILEY_START_X,
                SMILEY_START_Y,
                SMILEY_SIZE,
            ) {
                &smiley_clicked
            } else {
                &smiley_open
            }
        } else if state == State::Dead {
            &smiley_dead
        } else if state == State::Won {
            &smiley_glasses
        } else {
            &smiley
        };

        draw_texture_with_size(
            smiley_texture,
            SMILEY_START_X,
            SMILEY_START_Y,
            SMILEY_SIZE,
            SMILEY_SIZE,
        );

        let (col, row) = (
            (((mouse_x - TILE_START_X) / TILE_SIZE) as usize).min(EXPERT_WIDTH - 1),
            (((mouse_y - TILE_START_Y) / TILE_SIZE) as usize).min(EXPERT_HEIGHT - 1),
        );

        if state == State::Playing
            && (is_mouse_button_released(MouseButton::Left)
                && is_mouse_button_released(MouseButton::Right)
                || is_mouse_button_down(MouseButton::Left)
                    && is_mouse_button_released(MouseButton::Right)
                || is_mouse_button_down(MouseButton::Right)
                    && is_mouse_button_released(MouseButton::Left)
                || is_mouse_button_released(MouseButton::Middle))
            && tiles[col][row].state == TileState::Revealed
            && surrounding_tiles(col, row)
                .into_iter()
                .filter(|(col, row)| tiles[*col][*row].mine)
                .count()
                == surrounding_tiles(col, row)
                    .into_iter()
                    .filter(|(col, row)| tiles[*col][*row].state == TileState::Flagged)
                    .count()
        {
            let surrounding_tiles = surrounding_tiles(col, row);

            for (surrounding_tile_col, surrounding_tile_row) in surrounding_tiles {
                if tiles[surrounding_tile_col][surrounding_tile_row].mine {
                    if tiles[surrounding_tile_col][surrounding_tile_row].state != TileState::Flagged
                    {
                        state = State::Dead;
                        reveal_all_mines(&mut tiles);
                        unflagged_mines.push((surrounding_tile_col, surrounding_tile_row))
                    } else {
                        continue;
                    }
                }

                if state == State::Playing {
                    tiles[surrounding_tile_col][surrounding_tile_row].state = TileState::Revealed;
                }

                if tiles[surrounding_tile_col][surrounding_tile_row].neighbour_mines_count == 0 {
                    reveal_empty_space(surrounding_tile_col, surrounding_tile_row, &mut tiles)
                }
            }
        } else if state == State::Playing
            && tiles[col][row].state != TileState::Revealed
            && is_mouse_button_pressed(MouseButton::Right)
            && hovering_tile(mouse_x, mouse_y, col, row)
        {
            if tiles[col][row].state == TileState::Flagged {
                number_flagged -= 1;
                tiles[col][row].state = TileState::Hidden
            } else {
                number_flagged += 1;
                tiles[col][row].state = TileState::Flagged
            }
        } else if (state == State::Playing || state == State::NewGame)
            && is_mouse_button_pressed(MouseButton::Left)
            && hovering_tile(mouse_x, mouse_y, col, row)
            && tiles[col][row].state != TileState::Flagged
        {
            if state == State::NewGame {
                tiles = generate_fair_game(col, row);
                start = Instant::now();
                state = State::Playing;
            }

            tiles[col][row].state = TileState::Revealed;

            if tiles[col][row].mine {
                state = State::Dead;

                reveal_all_mines(&mut tiles);

                unflagged_mines.push((col, row));
            } else if tiles[col][row].neighbour_mines_count == 0 {
                reveal_empty_space(col, row, &mut tiles)
            }
        }

        for row in 0..EXPERT_HEIGHT {
            for col in 0..EXPERT_WIDTH {
                if state == State::Dead && tiles[col][row].mine {
                    if unflagged_mines.contains(&(col, row)) {
                        draw_rectangle(
                            TILE_START_X + 1.0 + col as f32 * TILE_SIZE,
                            TILE_START_Y + 1.0 + row as f32 * TILE_SIZE,
                            TILE_SIZE - 1.0,
                            TILE_SIZE - 1.0,
                            RED,
                        )
                    }

                    draw_at_tile(&mine, col, row)
                }

                if tiles[col][row].state == TileState::Revealed {
                    let neighbour_mines_count = tiles[col][row].neighbour_mines_count;

                    if neighbour_mines_count != 0
                        && !tiles[col][row].mine
                        && tiles[col][row].state != TileState::Flagged
                    {
                        draw_at_tile(
                            &neighbour_mines_textures[neighbour_mines_count as usize - 1],
                            col,
                            row,
                        )
                    }
                }

                if state == State::Dead
                    && tiles[col][row].state == TileState::Flagged
                    && !tiles[col][row].mine
                {
                    tiles[col][row].state = TileState::Revealed;
                    draw_at_tile(&mine, col, row);
                    draw_at_tile(&cross, col, row);
                } else {
                    if tiles[col][row].state != TileState::Revealed {
                        draw_at_tile(&tile, col, row);
                    }

                    if tiles[col][row].state == TileState::Flagged {
                        draw_at_tile(&flag, col, row)
                    }
                }
            }
        }

        if state == State::Playing {
            elapsed = start.elapsed().as_secs() as usize
        }

        next_frame().await
    }
}

fn is_game_won(tiles: &[Vec<Tile>]) -> bool {
    for row in tiles {
        for tile in row {
            if !tile.mine && tile.state != TileState::Revealed {
                return false;
            }
        }
    }
    true
}

fn reveal_empty_space(col: usize, row: usize, tiles: &mut [Vec<Tile>]) {
    let mut queue = VecDeque::<(usize, usize)>::new();
    queue.push_back((col, row));

    while !queue.is_empty() {
        let (current_col, current_row) = queue.pop_back().unwrap();

        for (neighbour_col, neighbour_row) in surrounding_tiles(current_col, current_row) {
            let mut neighbour_tile = &mut tiles[neighbour_col][neighbour_row];
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

fn reveal_all_mines(tiles: &mut [Vec<Tile>]) {
    for col in tiles.iter_mut() {
        for tile in col.iter_mut() {
            if tile.mine && tile.state != TileState::Flagged {
                tile.state = TileState::Revealed
            }
        }
    }
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
