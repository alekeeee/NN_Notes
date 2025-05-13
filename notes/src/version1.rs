use crossterm::{
    cursor::{
        self, DisableBlinking, EnableBlinking, MoveDown, MoveRight, MoveTo, MoveUp, SavePosition,MoveLeft
    },
    event::{poll, read, Event, KeyCode},
    execute,
    queue,
    style::Print,
    terminal::{self, disable_raw_mode, enable_raw_mode, Clear, ClearType, ClearType::All},
    Command, ExecutableCommand, QueueableCommand,
    event::{KeyEvent, KeyModifiers},
};

use std::{
    collections::HashMap,
    fmt,
    fs::{self, File, read_to_string},
    io::{self, Read, Write, stdout},
    path::Path,
    thread,
    time::Duration,
};



#[derive(Eq, Hash, PartialEq)]
struct Resident{
    name: String,
    prompt: u8,
}

impl Resident {
    fn new(name: &str, prompt: u8) -> Resident {
        Resident { name: name.to_string(), prompt: prompt}
    }
}

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

fn main() {
    let mut stdout = stdout();
    let _ = enable_raw_mode().unwrap();
    let mut index = 1;
    let _ = stdout.execute(Clear(All));
    let _ = stdout.execute(MoveTo(0,0));
    let mut room: HashMap<Resident, u8> = HashMap::new();
    let mut note = String::new();
    let mut quit = false;
    let mut editing = false;
    let mut content = String::new();
    let mut display = true;

       for i in 1..=11 {
        let filename = format!("{}.txt", i);
        if Path::new(&filename).exists(){
        } else {
           let file = File::create(&filename);
            println!("{} created", filename);
        }
        let new = Resident::new(&filename.to_string(), 0);
        room.insert(new, i);
        
       }
    
    while !quit {
        while poll(Duration::ZERO).unwrap() {
            let event = read().unwrap();
            if editing {
                match event {
                    Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, ..}) => match code {
                        KeyCode::Char(c) => {
                            content.push(c);
                            queue!(stdout,Print(c)).unwrap();
                        }
                        KeyCode::Backspace => {
                            if content.pop().is_some(){
                                queue!(stdout, MoveLeft(1), Print(' '), MoveLeft(1)).unwrap();
                            }
                        }
                        KeyCode::Enter => {
                            content.push('\n');
                            queue!(stdout, Print('\n')).unwrap();
                        }
                        KeyCode::Esc => {
                            let filename = format!("{}.txt", index);
                            let mut file = File::create(&filename).unwrap();
                            file.write_all(content.as_bytes()).unwrap();
                            editing = false;
                            queue!(stdout, Clear(All), MoveTo(0,0), Print("Content saved.")).unwrap();
                    
                        }
                        _ => {}
                    },
                    _ => {}
                }
            } else {
                match event {
                    Event::Key(KeyEvent { code: KeyCode::Char('q'), modifiers: KeyModifiers::CONTROL, .. }) => {
                        disable_raw_mode().unwrap();
                        quit = true;
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char('l'), modifiers: KeyModifiers::CONTROL, .. }) => {
                        let (_w, h) = terminal::size().unwrap();
                        queue!(stdout, MoveTo(0, h - 1), Clear(ClearType::CurrentLine)).unwrap();
                        queue!(stdout, Print("Enter index (1-11): ")).unwrap();
                        stdout.flush().unwrap();
            
                        let new_index = read_number_input(1, 11).unwrap_or(index);
                        index = new_index;
                        let filename = format!("{}.txt", index);
                        let mut data = File::open(&filename).unwrap();
                        content.clear();
                        data.read_to_string(&mut content).unwrap();
                        queue!(stdout, Clear(All), MoveTo(0,0), Print(&content));
                        editing = true;

                    }
                    Event::Key(KeyEvent { code: KeyCode::Right, modifiers: NONE, ..}) => {
                        stdout.queue(MoveRight(1)).unwrap();
                    }
                    Event::Key(KeyEvent { code: KeyCode::Char('n'), modifiers: KeyModifiers::CONTROL, ..}) => {
                        //edit specified resident name and prompt
                    }
                    _ => {}

                }
            }
        for person in room {
            println!("{}", person)
        } 
        } 
        stdout.flush().unwrap();

    } 
}   



fn read_number_input(min: u8, max: u8) -> Option<u8> {
    let mut number_str = String::new();
 

    loop {
        if let Event::Key(KeyEvent { code, modifiers: KeyModifiers::NONE, .. }) = read().unwrap() {
            let (w, h) = terminal::size().unwrap();
            match code {
                KeyCode::Char(c) if c.is_digit(10) => {
                    number_str.push(c);
                    print!("{}", c);
                    stdout().flush().unwrap();
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
                            stdout().flush().unwrap();
                        }
                    } else {
                        println!("Invalid input. Please enter a number.");
                        number_str.clear();
                        print!("Enter index ({}-{}): ", min, max);
                        stdout().flush().unwrap();
                    }
                }
                KeyCode::Esc => {
                    // User pressed Esc, cancel input
                    return None;
                }
                KeyCode::Backspace => {
                    if number_str.pop().is_some() {
                        queue!(
                            stdout(),
                            MoveTo((number_str.len() + 20) as u16, h - 1), // Adjust cursor position
                            Clear(ClearType::UntilNewLine)
                        )
                        .unwrap();
                        stdout().flush().unwrap();
                    }
                }
                _ => {}
            }
        }
    }
}


fn render_room(stdout: &mut std::io::Stdout, room: &HashMap<Resident, u8>) -> crossterm::Result<()> {
    // Clear the screen
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    // Iterate over the room and display each Resident
    for resident in room {
        queue!(stdout, Print(format!("{}", resident)))?;
    }
    stdout.flush()?;
    Ok(())
}
