use std::io::stdout;

use crossterm::event::poll;
use crossterm::{
    cursor::{position, MoveUp},
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, Clear, ClearType},
    Result,
};
use std::time::Duration;

// Resize events can occur in batches.
// With a simple loop they can be flushed.
// This function will keep the first and last resize event.
fn flush_resize_events(event: Event) -> ((u16, u16), (u16, u16)) {
    if let Event::Resize(x, y) = event {
        let mut last_resize = (x, y);
        while let Ok(true) = poll(Duration::from_millis(50)) {
            if let Ok(Event::Resize(x, y)) = read() {
                last_resize = (x, y);
            }
        }

        return ((x, y), last_resize);
    }
    ((0, 0), (0, 0))
}

pub struct Select<'a, T>
where
    T: AsRef<str>,
{
    options: &'a Vec<&'a T>,
    highlighted_index: usize,
}

impl<'a, T> Select<'a, T>
where
    T: AsRef<str>,
{
    pub fn new(options: &'a Vec<&'a T>) -> Self {
        Self {
            options,
            highlighted_index: 0,
        }
    }

    pub fn display(&mut self) -> Result<&'a T> {
        enable_raw_mode()?;

        self.render()?;

        loop {
            let mut stdout = stdout();
            let event = read()?;

            execute!(
                stdout,
                MoveUp((self.options.len()) as u16),
                Clear(ClearType::FromCursorDown)
            )?;

            if event == Event::Key(KeyCode::Down.into()) {
                self.next();
            }

            if event == Event::Key(KeyCode::Up.into()) {
                self.prev();
            }

            self.render()?;

            if event == Event::Key(KeyCode::Char('c').into()) {
                println!("Cursor position: {:?}\r", position());
            }

            if let Event::Resize(_, _) = event {
                let (original_size, new_size) = flush_resize_events(event);
                println!("Resize from: {:?}, to: {:?}", original_size, new_size);
            }

            if event == Event::Key(KeyCode::Enter.into()) {
                break;
            }

            if event == Event::Key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)) {
                std::process::exit(0);
            }
        }

        disable_raw_mode()?;

        Ok(self.options[self.highlighted_index])
    }

    fn next(&mut self) {
        if self.highlighted_index < self.options.len() - 1 {
            self.highlighted_index += 1;
        } else {
            self.highlighted_index = 0;
        }
    }

    fn prev(&mut self) {
        if self.highlighted_index > 0 {
            self.highlighted_index -= 1;
        } else {
            self.highlighted_index = self.options.len() - 1;
        }
    }

    fn render(&self) -> Result<()> {
        let mut stdout = stdout();
        for (index, option) in self.options.iter().enumerate() {
            if index == self.highlighted_index {
                execute!(
                    stdout,
                    SetForegroundColor(Color::Rgb {
                        r: 28,
                        g: 234,
                        b: 213
                    }),
                    Print("> "),
                    ResetColor,
                    Print(option.as_ref())
                )?;
            } else {
                execute!(stdout, Print(option.as_ref()))?;
            };
            execute!(stdout, Print("\r\n"))?;
        }

        Ok(())
    }
}
