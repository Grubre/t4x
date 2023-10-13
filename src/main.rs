use std::{
    cmp::max,
    io::{self, Write},
    process::exit,
    time::Duration,
};

use crossterm::{
    cursor::{self, MoveTo},
    event::{poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute, queue,
    style::{self, Color, SetForegroundColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
    QueueableCommand,
};
use rand::Rng;

fn setup_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    execute!(stdout, EnableMouseCapture)?;
    Ok(())
}

fn cleanup_terminal() -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, DisableMouseCapture)?;

    disable_raw_mode()?;
    execute!(stdout, LeaveAlternateScreen)?;
    Ok(())
}

fn poll_events(state: &mut State) -> io::Result<()> {
    if !poll(Duration::from_millis(100))? {
        return Ok(());
    }

    let event = read()?;

    if let Event::Key(key) = event {
        match key.code {
            KeyCode::Char('q') => {
                cleanup_terminal()?;
                exit(0);
            }
            KeyCode::Left => {
                state.pointer_pos.0 = max(state.pointer_pos.0 - 1, 0);
            }
            KeyCode::Right => {
                state.pointer_pos.0 += 1;
            }
            KeyCode::Up => {
                state.pointer_pos.1 = max(state.pointer_pos.1 - 1, 0);
            }
            KeyCode::Down => {
                state.pointer_pos.1 += 1;
            }
            KeyCode::Enter => {
                for tile in &mut state.tiles {
                    if tile.color != Color::Black {
                        tile.color = Color::DarkYellow;
                    }
                }
            }
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

struct Tile {
    color: Color,
}

fn generate_map(width: u16, height: u16) -> Vec<Tile> {
    let mut vec = Vec::new();
    for _y in 0..height {
        for _x in 0..width {
            vec.push(Tile {
                color: random_color(),
            })
        }
    }
    vec
}

fn get_visible_screen_rect_left_top(state: &State, width: u16, height: u16) -> (u64, u64) {
    let pointer_pos = &state.pointer_pos;

    let half_screen_w: u64 = (width / 2).into();
    let half_screen_h: u64 = (height / 2).into();

    let left_top_x = pointer_pos.0 - half_screen_w;
    let left_top_y = pointer_pos.1 - half_screen_h;

    (left_top_x, left_top_y)
}

fn draw(state: &State, width: u16, height: u16) -> io::Result<()> {
    let solid_rectangle_char = '\u{2588}';
    let mut stdout = io::stdout();
    let rect = get_visible_screen_rect_left_top(state, width, height);
    for y in 0..height {
        for x in 0..width {
            let tx: u64 = u64::from(x) + rect.0;
            let ty: u64 = u64::from(y) + rect.1;
            let tile = state.tiles.get((ty * u64::from(width) + tx) as usize);
            let color = if let Some(tile) = tile {
                tile.color
            } else {
                Color::Black
            };
            queue!(
                stdout,
                cursor::MoveTo(x, y),
                SetForegroundColor(color),
                style::Print(solid_rectangle_char)
            )?
        }
    }
    stdout.flush()?;

    Ok(())
}

struct State {
    pointer_pos: (u64, u64),
    tiles: Vec<Tile>,
}

fn main() -> io::Result<()> {
    setup_terminal()?;

    let (width, height) = terminal::size().unwrap();
    let map = generate_map(width, height);

    let mut state = State {
        pointer_pos: ((width / 2).into(), (height / 2).into()),
        tiles: map,
    };

    loop {
        if let Err(e) = poll_events(&mut state) {
            println!("Error: {:?}\r", e);
        }
        draw(&state, width, height)?;
    }
}
