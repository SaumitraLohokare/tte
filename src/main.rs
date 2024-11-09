mod buffer;
mod display;

use std::{
    env::args,
    io::{self, stdout},
    panic,
    process::exit,
};

use buffer::Buffer;
use crossterm::{
    cursor::SetCursorStyle,
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
};
use display::Display;

fn main() {
    panic::set_hook(Box::new(|panic_info| {
        // Get the panic location if available
        if let Some(location) = panic_info.location() {
            println!("Panic occurred at {}:{}", location.file(), location.line());
        } else {
            println!("Panic occurred, but location is unknown.");
        }
    }));

    if let Err(e) = run() {
        eprintln!("ERROR : {e}");
        exit(1);
    }
}

fn run() -> io::Result<()> {
    // TODO: Make this better
    let args: Vec<String> = args().collect();
    if args.len() > 2 {
        eprintln!("USAGE: {} [filename]", args[0]);
        eprintln!("- If file is not provided, an empty buffer is opened.");
        exit(1);
    }

    let mut buffer = if args.len() == 1 {
        Buffer::new()
    } else {
        Buffer::from_file(&args[1])
    };

    let mut display = Display::new(stdout())?;
    display.set_cursor_style(SetCursorStyle::BlinkingBar)?;

    loop {
        display.clear_all()?;
        display.begin_draw()?;

        if let Ok(event) = read() {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break,
                Event::Resize(w, h) => display.resize(w, h)?,

                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.move_cursor_left(1);
                    buffer.scroll()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.move_cursor_right(1);
                    buffer.scroll()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.move_cursor_up(1);
                    buffer.scroll()?;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.move_cursor_down(1);
                    buffer.scroll()?;
                }

                _ => (),
            }
        }
        
        // DEBUGGING STUFF
        // display.print(format!("{event:?}"))?;

        // display.move_cursor_to(30, 0)?;
        // display.print(format!("{} ({:?}) -> {:?}", buffer.cursor_pos, buffer.data[buffer.cursor_pos], buffer.cursor_xy()))?;
        // display.move_cursor_to(30, 1)?;
        // display.print(format!("({}, {})", buffer.lines[0].start, buffer.lines[0].end))?;
        
        // display.move_cursor_to(30, 0)?;
        // display.print(format!(" Cursor {:?} | Terminal {:?} | Y Off {}", buffer.cursor_xy(), terminal::size()?, buffer.offset_y))?;
        
        display.draw_buffer(&buffer)?;

        display.end_draw()?;
    }

    Ok(())
}
