use std::io;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::Terminal;
use ratatui::widgets::Paragraph;

use crate::Args;
use crate::editor::buffer::Buffer;

mod buffer;

/// Live parser development editor.
pub struct Editor {
  buffers: Vec<Buffer>,
  active_buffer: usize,
}

impl Editor {
  /// Create a new editor from `args`.
  ///
  /// # Errors
  ///
  /// Returns an error when creating a buffer fails.
  pub fn new(args: Args) -> Result<Self, io::Error> {
    let mut buffers = Vec::with_capacity(1 + args.program_file_paths.len());
    buffers.push(Buffer::new(args.grammar_file_path)?); // First buffer is always the grammar buffer.
    for path in args.program_file_paths {
      buffers.push(Buffer::new(path)?); // Subsequent buffers are always example program buffers.
    }

    let editor = Self { buffers, active_buffer: 0, };
    Ok(editor)
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
      let root_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(100), Constraint::Min(1)])
        .split(frame.size());
      let buffer_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root_areas[0]);

      // Draw grammar buffer on the left (`buffer_areas[0]`).
      self.buffers[0].draw(frame, buffer_areas[0], self.active_buffer == 0);

      { // Draw example program buffers on the right (`buffer_areas[1]`).
        let num_program_buffers = self.buffers.len() - 1;
        // Split vertical space between example program buffers.
        let program_buffer_areas = Layout::default()
          .direction(Direction::Vertical)
          .constraints(vec![Constraint::Ratio(1, num_program_buffers as u32); num_program_buffers])
          .split(buffer_areas[1]);
        for ((buffer, area), i) in self.buffers[1..].iter_mut().zip(program_buffer_areas.iter()).zip(1..) {
          buffer.draw(frame, *area, self.active_buffer == i);
        }
      }

      // Draw help line on the last line (`root_areas[1]`).
      let help = Paragraph::new("Interactive Parser Development. Press Esc to quit, ^T to switch active \
                                            buffer.");
      frame.render_widget(help, root_areas[1]);
    })?;

    match crossterm::event::read()? {
      Event::Key(key) if key.kind == KeyEventKind::Release => return Ok(true), // Skip releases.
      Event::Key(key) if key.code == KeyCode::Esc => return Ok(false),
      Event::Key(key) if key.code == KeyCode::Char('t') && key.modifiers.contains(KeyModifiers::CONTROL) => {
        self.active_buffer = (self.active_buffer + 1) % self.buffers.len();
      }
      event => self.buffers[self.active_buffer].process_event(event), // Otherwise: forward to current buffer.
    };

    Ok(true)
  }
}
