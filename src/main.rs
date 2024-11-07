use std::{
    env::args,
    fs,
    io::{self, stdout, Write},
    process::exit,
};

use crossterm::{
    cursor::{Hide, MoveTo, SetCursorStyle, Show},
    event::{read, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute, queue,
    style::{Print, ResetColor},
    terminal::{
        self, disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
};

fn main() {
    let mut stdout = stdout();

    let args: Vec<String> = args().collect();
    if args.len() > 2 {
        eprintln!("USAGE: {} [filename]", args[0]);
        eprintln!("- If file is not provided, an empty buffer is opened.");
        exit(1);
    }

    let mut buffer = if args.len() == 1 {
        vec![]
    } else {
        match fs::read(&args[1]) {
            Ok(bytes) => bytes.into_iter().map(|b| b as char).collect(),
            Err(_) => {
                // If file does not exist, make an empty buffer
                vec![]
            }
        }
    };
    let filename = if args.len() == 1 {
        String::new()
    } else {
        args[1].clone()
    };

    let mut lines = vec![];
    let mut cursor_pos = 0;

    recalculate_lines(&buffer, &mut lines);

    // let size = terminal::size().unwrap();

    queue!(stdout, EnterAlternateScreen, SetCursorStyle::BlinkingBar,).unwrap();

    refresh_screen(&mut stdout).unwrap();

    stdout.flush().unwrap();

    enable_raw_mode().unwrap();

    loop {
        stdout.flush().unwrap();

        refresh_screen(&mut stdout).unwrap();

        if let Ok(event) = read() {
            match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => break,
                Event::Key(KeyEvent {
                    code: KeyCode::Char('s'),
                    modifiers: KeyModifiers::CONTROL,
                    ..
                }) => {
                    if filename.len() > 0 {
                        fs::write(
                            &filename,
                            buffer.iter().map(|c| *c as u8).collect::<Vec<u8>>(),
                        )
                        .unwrap();
                    } else {
                        queue!(stdout, Print("No file name provided in command line arguments... lol we don't have `save as` yet.")).unwrap();
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.insert(cursor_pos, c);
                    cursor_pos += 1;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    modifiers: KeyModifiers::SHIFT,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.insert(cursor_pos, c.to_ascii_uppercase());
                    cursor_pos += 1;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    buffer.insert(cursor_pos, '\n');
                    cursor_pos += 1;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Backspace,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if cursor_pos > 0 {
                        buffer.remove(cursor_pos - 1); // TODO: This panics when we try to backspace an empty buffer
                        cursor_pos -= 1;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Delete,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if buffer.len() > cursor_pos {
                        buffer.remove(cursor_pos); // TODO: This panics when we try to delete at end of buffer
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if cursor_pos > 0 {
                        cursor_pos -= 1;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    if cursor_pos < buffer.len() {
                        cursor_pos += 1;
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Up,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let (mut cursor_x, mut cursor_y) = get_cursor_xy(cursor_pos, &mut lines);

                    if cursor_y > 0 {
                        cursor_y -= 1;
                        if let Some(line) = lines.get(cursor_y) {
                            if (line.1 - line.0) < cursor_x {
                                cursor_x = line.1 - line.0;
                            }

                            cursor_pos = line.0 + cursor_x;
                        }
                    }
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    modifiers: KeyModifiers::NONE,
                    kind: KeyEventKind::Press,
                    ..
                }) => {
                    let (mut cursor_x, mut cursor_y) = get_cursor_xy(cursor_pos, &mut lines);

                    // When refactoring, please rewrite this
                    if !is_cursor_at_last_line(cursor_pos, &mut lines) {
                        cursor_y += 1;
                        if let Some(line) = lines.get(cursor_y) {
                            if (line.1 - line.0) < cursor_x {
                                cursor_x = line.1 - line.0;
                            }
                            cursor_pos = line.0 + cursor_x;
                        }
                    }
                }

                _ => (),
            }
        } else {
            break;
        }

        recalculate_lines(&buffer, &mut lines);

        let (cursor_x, cursor_y) = get_cursor_xy(cursor_pos, &mut lines);

        queue!(
            stdout,
            Hide,
            Print(buffer.iter().collect::<String>()),
            Show,
            MoveTo(cursor_x as u16, cursor_y as u16),
        )
        .unwrap();
    }

    disable_raw_mode().unwrap();

    execute!(stdout, ResetColor, LeaveAlternateScreen,).unwrap();
}

fn get_cursor_xy(cursor_pos: usize, lines: &mut Vec<(usize, usize)>) -> (usize, usize) {
    let mut cursor_x = 0;
    let mut cursor_y = 0;

    if let Some((_begin, end)) = lines.last() {
        if cursor_pos > *end {
            cursor_y = lines.len();
            cursor_x = cursor_pos - end - 1;
        } else {
            for (i, (begin, end)) in lines.iter().enumerate() {
                if *begin <= cursor_pos && *end >= cursor_pos {
                    cursor_y = i;
                    cursor_x = cursor_pos - begin;
                }
            }
        }
    } else {
        cursor_x = cursor_pos;
    }

    (cursor_x, cursor_y)
}

fn is_cursor_at_last_line(cursor_pos: usize, lines: &mut Vec<(usize, usize)>) -> bool {
    if let Some((_begin, end)) = lines.last() {
        cursor_pos > *end
    } else {
        true
    }
}

fn recalculate_lines(buffer: &Vec<char>, lines: &mut Vec<(usize, usize)>) {
    let mut previous_begining = 0;
    lines.clear();

    for (i, ch) in buffer.iter().enumerate() {
        if *ch == '\n' {
            lines.push((previous_begining, i));
            previous_begining = i + 1;
        }
    }

    lines.push((previous_begining, buffer.len()));
}

fn refresh_screen<W>(w: &mut W) -> io::Result<()>
where
    W: Write,
{
    queue!(w, Clear(ClearType::All), MoveTo(0, 0), ResetColor,)
}
