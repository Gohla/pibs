use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::Terminal;
use ratatui::widgets::Paragraph;

use crate::Args;

/// Live parser development editor.
pub struct Editor {}

impl Editor {
  /// Create a new editor from `args`.
  pub fn new(_args: Args) -> Result<Self, io::Error> {
    Ok(Self {})
  }

  /// Run the editor, drawing it into an alternate screen of the terminal.
  pub fn run(&mut self) -> Result<(), io::Error> {
    // Setup terminal for GUI rendering.
    enable_raw_mode()?;
    let mut backend = CrosstermBackend::new(io::stdout());
    crossterm::execute!(backend, EnterAlternateScreen, EnableMouseCapture)?;
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    // Draw and process events in a loop until a quit is requested or an error occurs.
    let result = loop {
      match self.draw_and_process_event(&mut terminal) {
        Ok(false) => break Ok(()), // Quit requested
        Err(e) => break Err(e), // Error
        _ => {},
      }
    };

    // First undo our changes to the terminal.
    disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;
    // Then present the result to the user.
    result
  }

  fn draw_and_process_event<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<bool, io::Error> {
    terminal.draw(|frame| {
      frame.render_widget(Paragraph::new("Hello World! Press Esc to exit."), frame.size());
    })?;

    match crossterm::event::read()? {
      Event::Key(key) if key.kind == KeyEventKind::Release => return Ok(true), // Skip releases.
      Event::Key(key) if key.code == KeyCode::Esc => return Ok(false),
      _ => {}
    };

    Ok(true)
  }
}
