#![allow(dead_code)]

use std::fs::{File, read_to_string};
use std::io::{self, Write};
use std::path::PathBuf;

use crossterm::event::Event;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_textarea::TextArea;

/// Editable text buffer for a file.
pub struct Buffer {
  path: PathBuf,
  editor: TextArea<'static>,
  feedback: String,
  modified: bool,
}

impl Buffer {
  /// Create a new [`Buffer`] for file at `path`.
  ///
  /// # Errors
  ///
  /// Returns an error when reading file at `path` fails.
  pub fn new(path: PathBuf) -> Result<Self, io::Error> {
    let text = read_to_string(&path)?;
    let mut editor = TextArea::from(text.lines());

    // Enable line numbers. Default style = no additional styling (inherit).
    editor.set_line_number_style(Style::default());

    Ok(Self { path, editor, feedback: String::default(), modified: false })
  }

  /// Draws this buffer with `frame` into `area`, highlighting it if it is `active`.
  pub fn draw(&mut self, frame: &mut Frame, area: Rect, active: bool) {
    // Determine and set styles based on whether this buffer is active. Default style = no additional styling (inherit).
    let mut cursor_line_style = Style::default();
    let mut cursor_style = Style::default();
    let mut block_style = Style::default();
    if active { // Highlight active editor.
      cursor_line_style = cursor_line_style.add_modifier(Modifier::UNDERLINED);
      cursor_style = cursor_style.add_modifier(Modifier::REVERSED);
      block_style = block_style.fg(Color::Gray);
    }
    self.editor.set_cursor_line_style(cursor_line_style);
    self.editor.set_cursor_style(cursor_style);

    // Create and set the block for the text editor, bordering it and providing a title.
    let mut block = Block::default().borders(Borders::ALL).style(block_style);
    if let Some(file_name) = self.path.file_name() { // Add file name as title.
      block = block.title(format!("{}", file_name.to_string_lossy()))
    }
    if self.modified { // Add modified to title.
      block = block.title("[modified]");
    }
    self.editor.set_block(block);

    // Split area up into a text editor (80% of available space), and feedback text (minimum of 7 lines).
    let areas = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Percentage(80), Constraint::Min(7)])
      .split(area);
    // Render text editor into first area (`areas[0]`).
    frame.render_widget(self.editor.widget(), areas[0]);
    // Render feedback text into second area (`areas[1]`).
    let feedback = Paragraph::new(self.feedback.clone())
      .wrap(Wrap::default())
      .block(Block::default().style(block_style).borders(Borders::ALL - Borders::TOP));
    frame.render_widget(feedback, areas[1]);
  }

  /// Process `event`, updating whether this buffer is modified.
  pub fn process_event(&mut self, event: Event) {
    self.modified |= self.editor.input(event);
  }

  /// Save this buffer to its file if it is modified. Does nothing if not modified. Sets as unmodified when successful.
  ///
  /// # Errors
  ///
  /// Returns an error if writing buffer text to the file fails.
  pub fn save_if_modified(&mut self) -> Result<(), io::Error> {
    if !self.modified {
      return Ok(());
    }
    let mut file = io::BufWriter::new(File::create(&self.path)?);
    for line in self.editor.lines() {
      file.write_all(line.as_bytes())?;
      file.write_all(b"\n")?;
    }
    file.flush()?;
    self.modified = false;
    Ok(())
  }

  /// Gets the file path of this buffer.
  pub fn path(&self) -> &PathBuf { &self.path }

  /// Gets the mutable feedback text of this buffer.
  pub fn feedback_mut(&mut self) -> &mut String { &mut self.feedback }
}
