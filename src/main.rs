use std::{
    cmp::max,
    io::{self, Write},
    iter::from_fn,
    process::exit,
    time::Duration,
};

use crossterm::{
    cursor::{self},
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute, queue,
    style::{self, Color, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use rand::Rng;

static PLAINS_COLOR: Color = Color::Green;
static DESERT_COLOR: Color = Color::Yellow;
static SOLID_RECTANGLE_CHAR: char = '\u{2588}';

fn setup_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(
        io::stdout(),
        EnterAlternateScreen,
        cursor::Hide,
        cursor::DisableBlinking
    )?;
    enable_raw_mode()?;

    execute!(stdout, EnableMouseCapture)?;
    Ok(())
}

fn cleanup_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()?;
    execute!(
        stdout,
        LeaveAlternateScreen,
        cursor::Show,
        cursor::EnableBlinking
    )?;
    Ok(())
}

fn poll_events(state: &mut State) -> io::Result<()> {
    if !poll(Duration::from_millis(100))? {
        return Ok(());
    }

    let event = read()?;

    let movement_mult = if let Event::Key(KeyEvent {
        code: _,
        modifiers: KeyModifiers::ALT,
        ..
    }) = event
    {
        5
    } else {
        1
    };

    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => {
                cleanup_terminal()?;
                exit(0);
            }
            KeyCode::Left => {
                state.pointer_pos.0 = state.pointer_pos.0.saturating_sub(movement_mult);
            }
            KeyCode::Right => {
                state.pointer_pos.0 = state.pointer_pos.0.saturating_add(movement_mult);
            }
            KeyCode::Up => {
                state.pointer_pos.1 = state.pointer_pos.1.saturating_sub(movement_mult);
            }
            KeyCode::Down => {
                state.pointer_pos.1 = state.pointer_pos.1.saturating_add(movement_mult);
            }
            KeyCode::Enter => {}
            _ => {}
        }
    }

    Ok(())
}

fn random_color() -> Color {
    let mut rng = rand::thread_rng();
    let r: u8 = rng.gen_range(0..=255);
    let g: u8 = rng.gen_range(0..=255);
    let b: u8 = rng.gen_range(0..=255);

    Color::Rgb { r, g, b }
}

#[derive(Debug, Clone)]
enum TileType {
    Plains,
    Desert,
}

#[derive(Clone)]
struct Unit {}

#[derive(Clone)]
struct Building {}

#[derive(Clone)]
struct Tile {
    tile_type: TileType,
    unit: Option<Unit>,
    building: Option<Building>,
}

fn generate_map(width: u16, height: u16) -> Vec<Vec<Tile>> {
    let mut vec: Vec<_> = vec![vec![]; width as usize];
    for x in 0..width {
        for y in 0..height {
            vec[x as usize].push(Tile{ tile_type: TileType::Plains, unit: None, building: None });
        }
    }
    vec
}

fn get_visible_world_rect_left_top(state: &State, width: u16, height: u16) -> (u64, u64) {
    let pointer_pos = &state.pointer_pos;

    let half_screen_w: u64 = (width / 2).into();
    let half_screen_h: u64 = (height / 2).into();

    let left_top_x = pointer_pos.0.saturating_sub(half_screen_w);
    let left_top_y = pointer_pos.1.saturating_sub(half_screen_h);

    (left_top_x, left_top_y)
}

fn draw_map(
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
            let color = if let Some(tile) = tile {
                match tile.tile_type {
                    TileType::Plains => PLAINS_COLOR,
                    TileType::Desert => DESERT_COLOR,
                }
            } else {
                Color::Black
            };
            queue!(
                stdout,
                cursor::MoveTo(x + screen_left_top_offset.0, y + screen_left_top_offset.1),
                SetForegroundColor(color),
                style::Print(SOLID_RECTANGLE_CHAR)
            )?
        }
    }
    stdout.flush()?;

    Ok(())
}

fn draw_ui(
    state: &State,
    screen_left_top_offset: (u16, u16),
    width: u16,
    height: u16,
) -> io::Result<()> {
    let mut stdout = io::stdout();

    // FIXME: the display is not being cleaned so there is leftover stuff from previous frames.
    // for example when we go from higher x,y values to lower
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

struct State {
    pointer_pos: (u64, u64),
    tiles: Vec<Vec<Tile>>,
}

fn main() -> io::Result<()> {
    setup_terminal()?;

    let (width, height) = terminal::size().unwrap();

    let map = generate_map(width * 2, height * 2);

    let mut state = State {
        pointer_pos: ((width / 2).into(), (height / 2).into()),
        tiles: map,
    };

    let ui_width = width / 5;

    loop {
        if let Err(e) = poll_events(&mut state) {
            println!("Error: {:?}\r", e);
        }
        draw_map(&state, (0, 0), width - ui_width, height)?;
        draw_ui(&state, (width - ui_width, 0), ui_width, height)?;
    }
}
