use std::time::Duration;

use ratatui::{buffer::{Buffer, Cell}, crossterm::{event::{self, Event, KeyCode, KeyEventKind}, terminal}, style::Color, DefaultTerminal, Frame};

// TODO: for the ratatui renderer
//
pub fn run_ratatui() -> Result<(), std::io::Error> {
    let mut terminal = ratatui::init();
    let dur = Duration::new(1, 0);
    loop {
        terminal.draw(render)?;

        if event::poll(dur)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if let KeyCode::Char(c) = key.code {
                        if c == 'q' {
                            break Ok(());
                        }
                    }
                }
            }
        }
    }
}

// will probably have to split each char
fn render(frame: &mut Frame) {
    let area = ratatui::layout::Rect { x: 3, y: 1, width: 1, height: 4 };
    let mut thing = Cell::new("x");
    thing.set_bg(Color::from_u32(0x60ff6060));
    *frame.buffer_mut() = Buffer::filled(area, thing)
}
