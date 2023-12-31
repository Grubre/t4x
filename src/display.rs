use std::io::{self, Write};

use crossterm::{
    cursor, queue,
    style::{self, Color, SetBackgroundColor, SetForegroundColor},
};

use crate::{
    map::{Tile, TileType},
    State,
};

static PLAINS_COLOR: Color = Color::Green;
static HILLS_COLOR: Color = Color::DarkGreen;
static DESERT_COLOR: Color = Color::Yellow;
static SOLID_RECTANGLE_CHAR: char = '\u{2588}';

fn get_visible_world_rect_left_top(state: &State, width: u16, height: u16) -> (u64, u64) {
    let pointer_pos = &state.pointer_pos;

    let half_screen_w: u64 = (width / 2).into();
    let half_screen_h: u64 = (height / 2).into();

    let left_top_x = pointer_pos.0.saturating_sub(half_screen_w);
    let left_top_y = pointer_pos.1.saturating_sub(half_screen_h);

    (left_top_x, left_top_y)
}

pub fn get_character(tile: &Tile) -> char {
    let unit = &tile.unit;
    let Some(unit) = unit else {
        return ' ';
    };
    match unit.unit_type {
        crate::map::UnitType::Civilian => 'c',
        crate::map::UnitType::Builder => 'b',
    }
}

pub fn get_color(tile: &Tile) -> Color {
    match tile.tile_type {
        TileType::Plains => PLAINS_COLOR,
        TileType::Desert => DESERT_COLOR,
        TileType::Hills => HILLS_COLOR,
    }
}

pub fn draw_map(
    state: &State,
    screen_left_top_offset: (u16, u16),
    width: u16,
    height: u16,
) -> io::Result<()> {
    let mut stdout = io::stdout();
    let rect = get_visible_world_rect_left_top(state, width, height);
    for y in 0..height {
        for x in 0..width {
            let tx: u64 = u64::from(x) + rect.0;
            let ty: u64 = u64::from(y) + rect.1;
            if (tx, ty) == state.pointer_pos {
                queue!(
                    stdout,
                    cursor::MoveTo(x + screen_left_top_offset.0, y + screen_left_top_offset.1),
                    SetForegroundColor(Color::White),
                    style::Print(SOLID_RECTANGLE_CHAR)
                )?;
                continue;
            }
            let tile = state
                .tiles
                .get(tx as usize)
                .and_then(|row| row.get(ty as usize));
            let (color, character) = if let Some(tile) = tile {
                let color = get_color(tile);
                let character = get_character(tile);
                (color, character)
            } else {
                (Color::Black, ' ')
            };
            queue!(
                stdout,
                cursor::MoveTo(x + screen_left_top_offset.0, y + screen_left_top_offset.1),
                SetForegroundColor(Color::Black),
                SetBackgroundColor(color),
                style::Print(character)
            )?
        }
    }
    stdout.flush()?;

    Ok(())
}

pub fn draw_ui(
    state: &State,
    screen_left_top_offset: (u16, u16),
    width: u16,
    height: u16,
) -> io::Result<()> {
    let mut stdout = io::stdout();

    let blank_row = " ".repeat(width as usize);

    // clear the row
    queue!(
        stdout,
        SetBackgroundColor(Color::Black),
        cursor::MoveTo(screen_left_top_offset.0, screen_left_top_offset.1),
        style::Print(&blank_row)
    )?;

    queue!(
        stdout,
        cursor::MoveTo(screen_left_top_offset.0, screen_left_top_offset.1),
        SetForegroundColor(Color::White),
        style::Print(format!(
            "x: {}, y: {}",
            state.pointer_pos.0, state.pointer_pos.1
        ))
    )?;

    Ok(())
}
