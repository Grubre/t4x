use std::{
    io::{self},
    process::exit,
    time::Duration,
};

use crossterm::style::Color;
use crossterm::{
    cursor::{self},
    event::{
        poll, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
    },
    execute,
    terminal::{
        self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
    },
};
use rand::Rng;
use t4x::{display::*, map::*, State};

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
            KeyCode::Char(' ') => {
                let tile = state
                    .tiles
                    .get_mut(state.pointer_pos.0 as usize)
                    .and_then(|row| row.get_mut(state.pointer_pos.1 as usize));
                if let Some(tile) = tile {
                    tile.unit = Some(Unit {});
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
