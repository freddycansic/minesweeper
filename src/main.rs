use std::{collections::VecDeque, fmt::Display};

use macroquad::prelude::*;

const EXPERT_WIDTH: usize = 30;
const EXPERT_HEIGHT: usize = 16;
const EXPERT_MINES: usize = 99;
const WINDOW_SIZE_MULTIPLIER: f32 = 1.5;
const WINDOW_WIDTH: i32 = (505.0 * WINDOW_SIZE_MULTIPLIER) as i32;
const WINDOW_HEIGHT: i32 = (324.0 * WINDOW_SIZE_MULTIPLIER) as i32;
const TILE_START_X: f32 = 12.0 * WINDOW_SIZE_MULTIPLIER;
const TILE_START_Y: f32 = 55.0 * WINDOW_SIZE_MULTIPLIER;
const TILE_SIZE: f32 = 16.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_START_X: f32 = 239.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_START_Y: f32 = 15.0 * WINDOW_SIZE_MULTIPLIER;
const SMILEY_SIZE: f32 = 26.0 * WINDOW_SIZE_MULTIPLIER;

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

#[macroquad::main(window_conf)]
async fn main() {
    let mut revealed = [[false; EXPERT_HEIGHT]; EXPERT_WIDTH];
    let mut flagged = [[false; EXPERT_HEIGHT]; EXPERT_WIDTH];
    let mut mines = generate_mines();
    let mut neighbour_mines_counts = generate_neighboured_mines(&mines);

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

    let mut alive = true;
    let mut unflagged_mines = Vec::<(usize, usize)>::new();
    let mut restart_game = false;

    loop {
        clear_background(Color::from_rgba(192, 192, 192, 255));
        let (mouse_x, mouse_y) = mouse_position();

        draw_texture_ex(
            &background,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width(), screen_height())),
                ..Default::default()
            },
        );

        if restart_game && is_mouse_button_released(MouseButton::Left) {
            alive = true;
            revealed = [[false; EXPERT_HEIGHT]; EXPERT_WIDTH];
            flagged = [[false; EXPERT_HEIGHT]; EXPERT_WIDTH];
            mines = generate_mines();
            neighbour_mines_counts = generate_neighboured_mines(&mines);
            unflagged_mines.clear();
            restart_game = false;
        }

        if is_mouse_button_down(MouseButton::Left) {
            if hovering_square(
                mouse_x,
                mouse_y,
                SMILEY_START_X,
                SMILEY_START_Y,
                SMILEY_SIZE,
            ) {
                draw_texture_ex(
                    &smiley_clicked,
                    SMILEY_START_X,
                    SMILEY_START_Y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(SMILEY_SIZE, SMILEY_SIZE)),
                        ..Default::default()
                    },
                );

                restart_game = true;
            } else {
                draw_texture_ex(
                    &smiley_open,
                    SMILEY_START_X,
                    SMILEY_START_Y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(vec2(SMILEY_SIZE, SMILEY_SIZE)),
                        ..Default::default()
                    },
                );
            }
        } else if !alive {
            draw_texture_ex(
                &smiley_dead,
                SMILEY_START_X,
                SMILEY_START_Y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(SMILEY_SIZE, SMILEY_SIZE)),
                    ..Default::default()
                },
            )
        } else {
            draw_texture_ex(
                &smiley,
                SMILEY_START_X,
                SMILEY_START_Y,
                WHITE,
                DrawTextureParams {
                    dest_size: Some(vec2(SMILEY_SIZE, SMILEY_SIZE)),
                    ..Default::default()
                },
            );
        };

        let (col, row) = (
            ((mouse_x - TILE_START_X) / TILE_SIZE) as usize,
            ((mouse_y - TILE_START_Y) / TILE_SIZE) as usize,
        );

        if (is_mouse_button_pressed(MouseButton::Left)
            && is_mouse_button_pressed(MouseButton::Right)
            || is_mouse_button_down(MouseButton::Left)
                && is_mouse_button_released(MouseButton::Right)
            || is_mouse_button_down(MouseButton::Right)
                && is_mouse_button_released(MouseButton::Left))
            && revealed[col][row]
            && number_flags_around(&flagged, col, row) == number_flags_around(&mines, col, row)
        {
            let surrounding_tiles = surrounding_tiles(col, row);

            for (surrounding_tile_col, surrounding_tile_row) in surrounding_tiles {
                if mines[surrounding_tile_col][surrounding_tile_row] {
                    if !flagged[surrounding_tile_col][surrounding_tile_row] {
                        alive = false;
                        reveal_all_mines(&mines, &flagged, &mut revealed);
                        unflagged_mines.push((surrounding_tile_col, surrounding_tile_row))
                    } else {
                        continue
                    }
                }

                revealed[surrounding_tile_col][surrounding_tile_row] = true;

                if neighbour_mines_counts[surrounding_tile_col][surrounding_tile_row] == 0 {
                    reveal_empty_space(
                        surrounding_tile_col,
                        surrounding_tile_row,
                        &mut revealed,
                        &neighbour_mines_counts,
                        &mines,
                    )
                }
            }
        }

        for row in 0..EXPERT_HEIGHT {
            for col in 0..EXPERT_WIDTH {
                if !alive && mines[col][row] {
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

                if revealed[col][row] {
                    let neighbour_mines_count = neighbour_mines_counts[col][row];

                    if neighbour_mines_count != 0 && !mines[col][row] && !flagged[col][row] {
                        draw_at_tile(
                            &neighbour_mines_textures[neighbour_mines_count as usize - 1],
                            col,
                            row,
                        )
                    }
                }

                if alive
                    && is_mouse_button_pressed(MouseButton::Left)
                    && hovering_tile(mouse_x, mouse_y, col, row)
                {
                    revealed[col][row] = true;

                    if mines[col][row] {
                        alive = false;

                        reveal_all_mines(&mines, &flagged, &mut revealed);

                        unflagged_mines.push((col, row));
                    } else if neighbour_mines_counts[col][row] == 0 {
                        reveal_empty_space(col, row, &mut revealed, &neighbour_mines_counts, &mines)
                    }
                }

                if alive
                    && !revealed[col][row]
                    && is_mouse_button_pressed(MouseButton::Right)
                    && hovering_tile(mouse_x, mouse_y, col, row)
                {
                    flagged[col][row] = !flagged[col][row];
                }

                if !alive && flagged[col][row] && !mines[col][row] {
                    revealed[col][row] = true;
                    draw_at_tile(&mine, col, row);
                    draw_at_tile(&cross, col, row);
                } else {
                    if !revealed[col][row] {
                        draw_at_tile(&tile, col, row);
                    }

                    if flagged[col][row] {
                        draw_at_tile(&flag, col, row)
                    }
                }
            }
        }

        next_frame().await
    }
}

fn reveal_empty_space(
    col: usize,
    row: usize,
    revealed: &mut [[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
    neighbour_mines_counts: &Vec<Vec<i32>>,
    mines: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
) {
    let mut queue = VecDeque::<(usize, usize)>::new();
    queue.push_back((col, row));

    while !queue.is_empty() {
        let (current_col, current_row) = queue.pop_back().unwrap();

        for (neighbour_col, neighbour_row) in surrounding_tiles(current_col, current_row) {
            if neighbour_mines_counts[neighbour_col][neighbour_row] == 0
                && !revealed[neighbour_col][neighbour_row]
                && !mines[neighbour_col][neighbour_row]
            {
                queue.push_back((neighbour_col, neighbour_row))
            }

            if !mines[neighbour_col][neighbour_row] {
                revealed[neighbour_col][neighbour_row] = true;
            }
        }
    }
}

fn reveal_all_mines(
    mines: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
    flagged: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
    revealed: &mut [[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
) {
    for row in 0..EXPERT_HEIGHT {
        for col in 0..EXPERT_WIDTH {
            if mines[col][row] && !flagged[col][row] {
                revealed[col][row] = true;
            }
        }
    }
}

fn draw_at_tile(texture: &Texture2D, col: usize, row: usize) {
    draw_texture_ex(
        texture,
        TILE_START_X + col as f32 * TILE_SIZE,
        TILE_START_Y + row as f32 * TILE_SIZE,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(TILE_SIZE, TILE_SIZE)),
            ..Default::default()
        },
    );
}

fn print_mines(mines: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH]) {
    for row in 0..EXPERT_HEIGHT {
        for col in 0..EXPERT_WIDTH {
            print!("{}", mines[col][row] as i32)
        }
        println!()
    }
}

fn generate_mines() -> [[bool; EXPERT_HEIGHT]; EXPERT_WIDTH] {
    let mut mines = [[false; EXPERT_HEIGHT]; EXPERT_WIDTH];

    for i in 0..EXPERT_MINES {
        mines[i % EXPERT_WIDTH][i / EXPERT_WIDTH] = true;
    }

    for col in mines.iter_mut() {
        fastrand::shuffle(col);
    }

    let (starting_click_x, starting_click_y) = (0, 0);

    if mines[starting_click_x][starting_click_y] {
        loop {
            let (random_tile_x, random_tile_y) = (
                fastrand::usize(0..EXPERT_WIDTH),
                fastrand::usize(0..EXPERT_HEIGHT),
            );

            if !mines[random_tile_x][random_tile_y] {
                mines[random_tile_x][random_tile_y] = true;
                break;
            }
        }

        mines[starting_click_x][starting_click_y] = false;
    }

    mines
}

fn generate_neighboured_mines(mines: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH]) -> Vec<Vec<i32>> {
    mines
        .iter()
        .enumerate()
        .map(|(col_index, col)| {
            col.iter()
                .enumerate()
                .map(move |(row_index, _)| number_flags_around(&mines, col_index, row_index))
                .collect::<Vec<i32>>()
        })
        .collect::<Vec<Vec<i32>>>()
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

fn number_flags_around(
    flagged: &[[bool; EXPERT_HEIGHT]; EXPERT_WIDTH],
    col: usize,
    row: usize,
) -> i32 {
    surrounding_tiles(col, row)
        .into_iter()
        .map(|(col, row)| flagged[col][row] as i32)
        .sum()
}
