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

#[derive(Debug)]
enum Tile {
    Plains { color: Color },
}

fn generate_map(width: u16, height: u16) -> Vec<Vec<Tile>> {
    let mut vec: Vec<_> = from_fn(|| Some(Vec::new())).take(width as usize).collect();
    for x in 0..width {
        for y in 0..height {
            vec[x as usize].push(Tile::Plains {
                color: Color::Rgb {
                    r: (y as u32 * 256 / height as u32) as u8,
                    g: (x as u32 * 256 / width as u32) as u8,
                    b: rand::thread_rng().gen_range(0..=255),
                },
            });
        }
    }
    vec
}

fn get_visible_screen_rect_left_top(state: &State, width: u16, height: u16) -> (u64, u64) {
    let pointer_pos = &state.pointer_pos;

    let half_screen_w: u64 = (width / 2).into();
    let half_screen_h: u64 = (height / 2).into();

    let left_top_x = pointer_pos.0.saturating_sub(half_screen_w);
    let left_top_y = pointer_pos.1.saturating_sub(half_screen_h);

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
            if (tx, ty) == state.pointer_pos {
                queue!(
                    stdout,
                    cursor::MoveTo(x, y),
                    SetForegroundColor(Color::White),
                    style::Print(solid_rectangle_char)
                )?;
                continue;
            }
            let tile = state
                .tiles
                .get(tx as usize)
                .and_then(|row| row.get(ty as usize));
            let color = if let Some(tile) = tile {
                match tile {
                    Tile::Plains { color } => *color,
                }
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

    loop {
        if let Err(e) = poll_events(&mut state) {
            println!("Error: {:?}\r", e);
        }
        draw(&state, width, height)?;
    }
}
