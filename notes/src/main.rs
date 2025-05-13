use crossterm::{
    cursor::{
        self, DisableBlinking, EnableBlinking, MoveDown, MoveRight, 
        MoveTo, MoveUp, SavePosition,MoveLeft, Hide, Show
    },
    event::{poll, read, Event, KeyCode, KeyEventKind},
    execute,
    queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, ClearType::All, ClearType::Purge},
    Command, ExecutableCommand, QueueableCommand,
    event::{KeyEvent, KeyModifiers},
};

use std::{
    collections::HashMap,
    fmt,
    fs::{self, File, read_to_string, OpenOptions, rename},
    io::{self, Read, Write, stdout, Result, Stdout, BufReader, BufRead, stdin},
    path::Path,
    thread,
    time::{Duration, Instant},
    result::Result as StdResult,
    
};



#[derive(Eq, Hash, PartialEq)]
struct Resident{
    name: String,
    prompt: u8,
}

// impl Resident {
//     fn new(name: &str, prompt: u8) -> Resident {
//         Resident { name: name.to_string(), prompt: prompt}
//     }
// }

impl fmt::Display for Resident {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Define how you want the structure to be printed
        write!(f, "Resident Name: {}, Prompt: {}", self.name, self.prompt)
    }
}

enum AppState {
    Display,
    Editing,
}

enum Direction {
    Left,
    Right,
    Up,
    Down,
}

fn main() {
    let mut stdout = stdout();
    let _ = enable_raw_mode().unwrap();

    let mut index = 1usize;
    let _ = stdout.execute(Clear(All));
    let _ = stdout.execute(MoveTo(0,0));
    let mut room_names: Vec<String> = Vec::with_capacity(11);
    let mut note = String::new();
    let mut quit = false;
    let mut content:Vec<String> = Vec::new();
    content.push(String::new());
    let mut state = AppState::Display;

    let mut cursor_line = 1usize;
    let mut cursor_col = 0usize;
 
    

    for i in 1..=11 {
        let filename = format!("{}.txt", i);
        if !Path::new(&filename).exists(){
            File::create(&filename).unwrap();
        } 
        let file = File::open(&filename).unwrap();
        let mut reader = BufReader::new(file);
        let mut first_line = String::new();
        let bytes_read = reader.read_line(&mut first_line).unwrap();
        if bytes_read == 0 {
            first_line = "unknown".to_string();
        } else {
            first_line = first_line.trim_end().to_string();
        }
        room_names.push(first_line);
    }

       render_rooms(&mut stdout, &room_names).unwrap();

    while !quit {
        if let AppState::Display = state {
            execute!(stdout, Hide);
            //stdout.flush().unwrap();
        } else {
            execute!(stdout, Show);
            //stdout.flush().unwrap();
        }
        if poll(Duration::from_millis(500)).unwrap() {
            let event = read().unwrap();
            match state {
                AppState::Display => {
                    //render_room_names(&mut stdout, &room_names).unwrap();
                    match event {
                        Event::Key(KeyEvent { code: KeyCode::Char('e'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press, ..}) => {
                            export(&mut stdout);

                        // Re-render the rooms
                            render_rooms(&mut stdout, &room_names).unwrap();
                            
                        }

                        Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press,.. }) => {
                            disable_raw_mode().unwrap();
                            quit = true;
                        }
                        Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press,.. }) => {
                            // Prompt to enter index
                            let (_w, h) = terminal::size().unwrap();
                            queue!(
                                stdout,
                                MoveTo(0, h - 1),
                                Clear(ClearType::CurrentLine),
                                Print("Enter index (1-11): ")
                            ).unwrap();
                            stdout.flush().unwrap();
    
                            if let Some(new_index) = read_number_input(1, 11) {
                            index = new_index as usize;
                            let filename = format!("{}.txt", index);
                            let file = File::open(&filename).unwrap();
                            let reader = BufReader::new(file);
                            content = reader.lines().collect::<StdResult<_, _>>().unwrap();
                            
                            if content.is_empty() {
                                content.push("unknown".to_string());
                            }

                            cursor_line = content.len() - 1;
                            cursor_col = content[cursor_line].len();

                            // cursor_line = 1;
                            // cursor_col = 0;

                            // Clear the screen and display the content
                            render_content(&mut stdout, &content, cursor_line, cursor_col).unwrap();

                            // Switch to editing mode
                            queue!(stdout, Show);
                            state = AppState::Editing;
                         
                        } else {
                            queue!(
                                stdout,
                                MoveTo(0, h -1),
                                Clear(ClearType::CurrentLine),
                                Print("Operaton canceled"),
                            ).unwrap();
                            stdout.flush().unwrap();
                        }
                    }
                        Event::Key(KeyEvent {
                            code: KeyCode::Char('r'),
                            modifiers: KeyModifiers::CONTROL,
                            ..
                        }) => {

                            let (_w, h) = crossterm::terminal::size().unwrap();
                            queue!(
                                stdout,
                                MoveTo(0, h - 1),
                                Clear(ClearType::CurrentLine),
                                Print("Enter room_names index to rename (1-11): ")
                            ).unwrap();
                            stdout.flush().unwrap();

                            if let Some(room_index) = read_number_input(1, 11) {
                                let room_idx = (room_index - 1) as usize;
                                queue!(
                                    stdout,
                                    MoveTo(0, h - 1),
                                    Clear(ClearType::CurrentLine),
                                    Print("Enter new name: ")
                                ).unwrap();
                                stdout.flush().unwrap();

                            if let Some(new_name) = read_string_input() {
                                room_names[room_idx] = new_name.clone();
                                let filename = format!("{}.txt", room_index);
                                let mut file_content = vec![new_name];

                                let file = OpenOptions::new().read(true).open(&filename).unwrap();
                                let reader = BufReader::new(file);
                                let lines = reader.lines().skip(1).collect::<StdResult<Vec<_>,_>>().unwrap();
                                file_content.extend(lines);

                                let mut file = OpenOptions::new()
                                    .write(true)
                                    .truncate(true)
                                    .open(&filename).unwrap();
                                let joined_content = file_content.join("\n");
                                file.write_all(joined_content.as_bytes()).unwrap();
                                
                                render_rooms(&mut stdout, &room_names).unwrap();
                            
                            } else {
                                queue!(
                                    stdout,
                                    MoveTo(0, h -1),
                                    Clear(ClearType::CurrentLine),
                                    Print("Rename canceled"),
                                ).unwrap();
                                stdout.flush().unwrap();
                            }
                        } else {
                            queue!(
                                stdout,
                                MoveTo(0, h - 1),
                                Clear(ClearType::CurrentLine),
                                Print("Rename canceled."),
                            ).unwrap();
                            stdout.flush().unwrap();
                        }
                        //stdout.flush().unwrap();
                    }
                        _ => {}
                    }
                }
                AppState::Editing => {
                    
                    match event {
                        Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press,.. }) => {
                            match code {
                            KeyCode::Char(c) => {
                                if cursor_line >= content.len() {
                                    content.push(String::new());
                                }
                                
                                // Insert the character and render just the change
                                content[cursor_line].insert(cursor_col, c);
                                
                                // Render only the new character at the current cursor position
                                queue!(
                                    stdout,
                                    MoveTo(cursor_col as u16, cursor_line as u16),
                                    Clear(ClearType::UntilNewLine), // Clear the line from the current cursor position onward
                                    Print(&content[cursor_line][cursor_col..]) // Print the updated line from the cursor position
                                ).unwrap();
                                
                                // Move the cursor to the next position
                                cursor_col += 1;
                                queue!(stdout, MoveTo(cursor_col as u16, cursor_line as u16)).unwrap();
                                
                                //stdout.flush().unwrap();

                                // if cursor_line >= content.len() {
                                //     content.push(String::new());
                                // }
                                // content[cursor_line].insert(cursor_col, c);
                                // cursor_col +=1;

                                // render_content(&mut stdout, &content, cursor_line, cursor_col).unwrap();
                                
                            }
                            KeyCode::Backspace => {
                                if cursor_col > 0 {
                                    cursor_col -= 1;
                                    content[cursor_line].remove(cursor_col);
                                } else if cursor_line > 1 {
                                    // Merge with the previous line
                                    cursor_col = content[cursor_line - 1].len();
                                    let current_line = content.remove(cursor_line);
                                    cursor_line -= 1;
                                    content[cursor_line].push_str(&current_line);
                                }
                                // Re-render the content
                                render_content(&mut stdout, &content, cursor_line, cursor_col).unwrap();
                            }
                            KeyCode::Enter => {
                                if cursor_line >= content.len() {
                                    content.push(String::new());
                                }
                                let new_line = content[cursor_line].split_off(cursor_col);
                                content.insert(cursor_line + 1, new_line);
                                cursor_line += 1;
                                cursor_col = 0;

                                // Re-render the content
                                render_content(&mut stdout, &content, cursor_line, cursor_col).unwrap();
                            }
                            KeyCode::Right => {

                                move_cursor( &mut stdout, &content,  &mut cursor_col,&mut cursor_line, Direction::Right).unwrap();

                            }
                            KeyCode::Left => {
                                move_cursor( &mut stdout, &content,  &mut cursor_col,&mut cursor_line, Direction::Left).unwrap();
                            }

                            KeyCode::Up => {
                                move_cursor(&mut stdout, &content, &mut cursor_col,&mut cursor_line, Direction::Up).unwrap()
                            }
                            KeyCode::Down => {
                                move_cursor( &mut stdout, &content,  &mut cursor_col,&mut cursor_line, Direction::Down).unwrap();
                            }


                            KeyCode::Esc => {
                                let filename = format!("{}.txt", index);
                                    let mut file = OpenOptions::new()
                                        .write(true)
                                        .truncate(true)
                                        .open(&filename).unwrap();
                                    let file_content = content.join("\n");
                                    file.write_all(file_content.as_bytes()).unwrap();

                                    content.clear();

                                    // Reset cursor position
                                    cursor_line = 1;
                                    cursor_col = 0;

                                    // Switch back to display mode
                                    state = AppState::Display;
                                    render_rooms(&mut stdout, &room_names).unwrap();
                                }
                            _ => {}
                            }
                        }
                        _ => {}
                        
                    }
                }
            }
       } 
        stdout.flush().unwrap();
    } 
}   



fn read_number_input(min: u8, max: u8) -> Option<u8> {
    let mut number_str = String::new();
    let mut stdout = stdout();
 

    loop {
        if let Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press,.. }) = read().unwrap() {
            let (w, h) = terminal::size().unwrap();
            match code {
                KeyCode::Char(c) if c.is_digit(10) => {
                    number_str.push(c);
                    print!("{}", c);
                    stdout.flush().unwrap();
                    
                }
                KeyCode::Enter => {
                    println!();
                    if let Ok(number) = number_str.parse::<u8>() {
                        if number >= min && number <= max {
                            return Some(number);
                        } else {
                            println!("Please enter a number between {} and {}", min, max);
                            number_str.clear();
                            print!("Enter index ({}-{}): ", min, max);
                            stdout.flush().unwrap();
                        }
                    } else {
                        println!("Invalid input. Please enter a number.");
                        number_str.clear();
                        print!("Enter index ({}-{}): ", min, max);
                        stdout.flush().unwrap();
                    }
                }
                KeyCode::Esc => {
                    // User pressed Esc, cancel input
                    return None;
                }
                KeyCode::Backspace => {
                    if number_str.pop().is_some() {
                        queue!(
                            stdout,
                            MoveLeft(1), // Adjust cursor position
                            Clear(ClearType::UntilNewLine)
                        )
                        .unwrap();
                        stdout.flush().unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}

fn render_content(stdout: &mut Stdout, content: &Vec<String>, cursor_line: usize, cursor_col: usize) -> Result<()> {
    queue!(stdout, Clear(ClearType::All), Clear(ClearType::Purge), MoveTo(0, 0)).unwrap();
    for (i, line) in content.iter().enumerate() {
        queue!(stdout, MoveTo(0, i as u16), Print(line)).unwrap();
    }
    queue!(stdout, MoveTo(cursor_col as u16, cursor_line as u16)).unwrap();
    //stdout.flush().unwrap();
    Ok(())
}


fn read_string_input() -> Option<String> {
    let mut input_str = String::new();
    let mut stdout = stdout();

    loop {
        if let Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, kind: KeyEventKind::Press,.. }) = read().unwrap() {
            match code {
                KeyCode::Char(c) => {
                    input_str.push(c);
                    print!("{}", c);
                    //stdout.flush().unwrap();
                }
                KeyCode::Enter => {
                    println!();
                    return Some(input_str);
                }
                KeyCode::Esc => {
                    // User pressed Esc, cancel input
                    return None;
                }
                KeyCode::Backspace => {
                    if input_str.pop().is_some() {
                        queue!(
                            stdout,
                            MoveLeft(1),
                            Clear(ClearType::UntilNewLine)
                        )
                        .unwrap();
                        
                    }
                }
                _ => {}
            }
        }
        //stdout.flush().unwrap();
    }
    
}

fn render_rooms(stdout: &mut Stdout, room_names: &Vec<String>) -> Result<()> {
    queue!(stdout, Clear(ClearType::All), Clear(ClearType::Purge), MoveTo(0, 0))?;
    for (i, name) in room_names.iter().enumerate() {
        let room_number = i + 1;
        queue!(
            stdout,
            MoveTo(0, i as u16),
            Clear(ClearType::CurrentLine),
            Print(format!("{}: {}", room_number, name))
        )?;
    }
    //stdout.flush()?;
    Ok(())
}

fn update_cursor(stdout: &mut Stdout, cursor_col: usize, cursor_line: usize) -> Result<()> {
    queue!(stdout, MoveTo(cursor_col as u16, cursor_line as u16))?;
    //stdout.flush()?;
    Ok(())
}

fn move_cursor(
    stdout: &mut Stdout,
    content: &Vec<String>,
    cursor_col: &mut usize,
    cursor_line: &mut usize,
    direction: Direction,
) -> Result<()> {
    match direction {
        Direction::Left => {
            if *cursor_col > 0 {
                *cursor_col -= 1;
            } else if *cursor_line > 1 {
                *cursor_line -= 1;
                *cursor_col = content[*cursor_line].len();
            }
        }
        Direction::Right => {
            if *cursor_col < content[*cursor_line].len() {
                *cursor_col += 1;
            } else if *cursor_line + 1 < content.len() {
                *cursor_line += 1;
                *cursor_col = 0;
            }
        }
        Direction::Up => {
            if *cursor_line > 1 {
                *cursor_line -= 1;
                *cursor_col = (*cursor_col).min(content[*cursor_line].len());
            }
        }
        Direction::Down => {
            if *cursor_line + 1 < content.len() {
                *cursor_line += 1;
                *cursor_col = (*cursor_col).min(content[*cursor_line].len());
            }
        }
    }
    update_cursor(stdout, *cursor_col, *cursor_line)
}

fn export (stdout: &mut Stdout) {
    let mut notes = String::new();
    let mut new_file_name = String::new();
    
    let (_w, h) = crossterm::terminal::size().unwrap();
    stdout
        .queue(MoveTo(0, h - 2)).unwrap()
        .queue(Clear(ClearType::CurrentLine)).unwrap()
        .queue(Print("Enter Export File Name: ")).unwrap();
    stdout.flush().unwrap();

    let new_file_name = read_string_input().unwrap();

    match File::create(&new_file_name) {
        Ok(mut export_file) => {
            for i in 1..=11 {
                let filename = format!("{}.txt", i);
                let file = match File::open(&filename) {
                    Ok(f) => f,
                    Err(_) => continue,
                };
                let mut reader = BufReader::new(file);
                let all_lines: Vec<String> = reader
                    .lines()
                    .filter_map(|line| line.ok())
                    .collect();

                // Append all lines to notes
                if !all_lines.is_empty() {
                    notes.push_str(&all_lines.join("\n"));
                    notes.push_str("\n\n"); // Add additional newlines between files
                }

                // Preserve the first line
                let first_line = if !all_lines.is_empty() {
                    &all_lines[0]
                } else {
                    ""
                };
                let mut file = match OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(&filename) {
                        Ok(f) => f,
                        Err(e) => {
                            stdout
                            .queue(MoveTo(0, h - 1)).unwrap()
                            .queue(Clear(ClearType::CurrentLine)).unwrap()
                            .queue(Print(format!("Failed to open '{}': {}", filename, e))).unwrap();
                        stdout.flush().unwrap();
                        continue;
                        }
                    };

                    if let Err(e) = writeln!(file, "{}", first_line) {
                        // Handle error if writing fails
                        stdout
                            .queue(MoveTo(0, h - 1)).unwrap()
                            .queue(Clear(ClearType::CurrentLine)).unwrap()
                            .queue(Print(format!("Failed to write to '{}': {}", filename, e))).unwrap();
                        stdout.flush().unwrap();
                    }
            }
            if let Err(e) = export_file.write_all(notes.as_bytes()) {
                stdout
                    .queue(MoveTo(0, h - 1)).unwrap()
                    .queue(Clear(ClearType::CurrentLine)).unwrap()
                    .queue(Print(format!("Failed to write to '{}': {}", new_file_name, e))).unwrap();
                stdout.flush().unwrap();
                return;
            }

            // Display success message
            stdout
                .queue(MoveTo(0, h - 1)).unwrap()
                .queue(Clear(ClearType::CurrentLine)).unwrap()
                .queue(Print(format!("Exported to '{}'", new_file_name))).unwrap();
            stdout.flush().unwrap();  
            
        },
        Err(e) => {
            stdout
                .queue(MoveTo(0, h - 1)).unwrap()
                .queue(Clear(ClearType::CurrentLine)).unwrap()
                .queue(Print(format!("Failed to create file '{}': {}", new_file_name, e))).unwrap();
            stdout.flush().unwrap();
            return;
        }
    }

    // match fs::write(&new_file_name, notes) {
    //     Ok(_) => {
    //         stdout
    //             .queue(MoveTo(0, h - 1)).unwrap()
    //             .queue(Clear(ClearType::CurrentLine)).unwrap()
    //             .queue(Print(format!("Exported to '{}'", new_file_name))).unwrap();
    //         stdout.flush().unwrap();
    //     },
    //     Err(e) => {
    //         stdout
    //             .queue(MoveTo(0, h - 1)).unwrap()
    //             .queue(Clear(ClearType::CurrentLine)).unwrap()
    //             .queue(Print(format!("Failed to write to file '{}': {}", new_file_name, e))).unwrap();
    //         stdout.flush().unwrap();
    //     }
    // }
}