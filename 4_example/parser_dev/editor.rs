use std::fmt::Write as FmtWrite;
use std::fs::{File, read_to_string};
use std::io::{self, Cursor, Write};
use std::path::PathBuf;

use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use ratatui::{Frame, Terminal};
use ratatui::backend::{Backend, CrosstermBackend};
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use tui_textarea::TextArea;

use pie::Pie;
use pie::tracker::writing::WritingTracker;

use crate::Args;
use crate::task::{Outputs, Tasks};

/// Live parser development editor.
pub struct Editor {
  pie: Pie<Tasks, Result<Outputs, String>, WritingTracker<Cursor<Vec<u8>>>>,
  buffers: Vec<Buffer>,
  active_buffer: usize,
  rule_name: String,
}

impl Editor {
  /// Create a new editor from `args`.
  pub fn new(args: Args) -> Result<Self, io::Error> {
    let tracker = WritingTracker::new(Cursor::new(Vec::new()));
    let pie = Pie::with_tracker(tracker);

    let mut buffers = Vec::with_capacity(1 + args.program_file_paths.len());
    buffers.push(Buffer::new(args.grammar_file_path)?); // First buffer is always the grammar buffer.
    for path in args.program_file_paths {
      buffers.push(Buffer::new(path)?); // Subsequent buffers are always program buffers.
    }

    let mut editor = Self { pie, buffers, active_buffer: 0, rule_name: args.rule_name, };
    editor.save_and_update_buffers();
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

    // Draw in a loop until a quit is requested or an error occurs.
    let result = loop {
      match self.draw(&mut terminal) {
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

  fn draw<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result<bool, io::Error> {
    terminal.draw(|frame| {
      let root_areas = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![Constraint::Percentage(80), Constraint::Percentage(20), Constraint::Min(1)])
        .split(frame.size());
      let buffer_areas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(root_areas[0]);

      // Draw grammar buffer on the left (`buffer_areas[0]`).
      self.buffers[0].draw(frame, buffer_areas[0], self.active_buffer == 0);

      { // Draw program buffers on the right (`buffer_areas[1]`).
        let num_buffers = self.buffers.len() - 1;
        let areas = Layout::default()
          .direction(Direction::Vertical)
          .constraints(vec![Constraint::Ratio(1, num_buffers as u32); num_buffers])
          .split(buffer_areas[1]);
        for ((buffer, area), i) in self.buffers[1..].iter_mut().zip(areas.iter()).zip(1..) {
          buffer.draw(frame, *area, self.active_buffer == i);
        }
      }

      { // Draw build log on the bottom (`root_areas[1]`).
        let text = Text::raw(String::from_utf8_lossy(&self.pie.tracker().writer().get_ref()));

        // Scroll down to last line, but that hides the entire build log.
        let scroll = text.height() as u16;
        // Scroll up the height of the build log area, making it visible. Use saturating sub to prevent overflows.
        let scroll = scroll.saturating_sub(root_areas[1].height);
        // Scroll down 2 lines due to the top and bottom border taking up 2 lines.
        let scroll = scroll + 2;

        let widget = Paragraph::new(text)
          .block(Block::default().title("Build log").borders(Borders::ALL))
          .scroll((scroll, 0));
        frame.render_widget(widget, root_areas[1]);
      };

      // Draw status line on the last line (`root_areas[2]`).
      let status = Paragraph::new("Live Parser Development. Press Esc to quit, ^X to switch active \
                                            buffer, ^S to save modified buffers and test changes, ^U to undo.");
      frame.render_widget(status, root_areas[2]);
    })?;

    match crossterm::event::read()? {
      Event::Key(key) if key.kind == KeyEventKind::Release => return Ok(true), // Skip releases.
      Event::Key(key) if key.code == KeyCode::Esc => return Ok(false),
      Event::Key(key) if key.code == KeyCode::Char('x') && key.modifiers.contains(KeyModifiers::CONTROL) => {
        self.active_buffer = (self.active_buffer + 1) % self.buffers.len();
      }
      Event::Key(key) if key.code == KeyCode::Char('s') && key.modifiers.contains(KeyModifiers::CONTROL) => {
        self.save_and_update_buffers();
      },
      event => self.buffers[self.active_buffer].process_event(event), // Forward to current buffer.
    };

    Ok(true)
  }

  fn save_and_update_buffers(&mut self) {
    for buffer in &mut self.buffers {
      buffer.status_mut().clear();
    }

    for buffer in &mut self.buffers {
      if let Err(error) = buffer.save_if_modified() {
        let _ = write!(buffer.status_mut(), "Saving file failed: {}", error);
      }
    }

    let mut session = self.pie.new_session();

    let grammar_buffer = &mut self.buffers[0];
    let compile_grammar_task = Tasks::CompileGrammar(grammar_buffer.path().clone());
    match session.require(&compile_grammar_task) {
      Err(error) => {
        let _ = write!(grammar_buffer.status_mut(), "{}", error);
        return; // Skip parsing if compiling grammar failed.
      }
      _ => {}
    }

    let compile_grammar_task = Box::new(compile_grammar_task);
    for buffer in &mut self.buffers[1..] {
      match session.require(&Tasks::Parse(compile_grammar_task.clone(), buffer.path().clone(), self.rule_name.clone())) {
        Err(error) => { let _ = write!(buffer.status_mut(), "{}", error); },
        Ok(Outputs::Parsed(Some(output))) => { let _ = write!(buffer.status_mut(), "Parsing succeeded: {}", output); },
        _ => {}
      }
    }
  }
}


struct Buffer {
  path: PathBuf,
  text_area: TextArea<'static>,
  modified: bool,
  status: String,
}

impl Buffer {
  fn new(path: PathBuf) -> Result<Self, io::Error> {
    let text = read_to_string(&path)?;

    let mut text_area = TextArea::from(text.lines());
    text_area.set_line_number_style(Style::default());

    Ok(Self { path, text_area, modified: false, status: String::default() })
  }

  fn draw(&mut self, frame: &mut Frame, area: Rect, active: bool) {
    let mut cursor_line_style = Style::default();
    let mut cursor_style = Style::default();
    let mut block = Block::default().borders(Borders::ALL);
    let mut block_style = Style::default();

    if active {
      cursor_line_style = cursor_line_style.add_modifier(Modifier::UNDERLINED);
      cursor_style = cursor_style.add_modifier(Modifier::REVERSED);
      block_style = block_style.fg(Color::Gray);
    }

    block = block.style(block_style);
    if let Some(file_name) = self.path.file_name() {
      block = block.title(format!("{}", file_name.to_string_lossy()))
    }
    if self.modified {
      block = block.title("[modified]");
    }

    self.text_area.set_cursor_line_style(cursor_line_style);
    self.text_area.set_cursor_style(cursor_style);
    self.text_area.set_block(block);

    let areas = Layout::default()
      .direction(Direction::Vertical)
      .constraints(vec![Constraint::Percentage(80), Constraint::Min(7)])
      .split(area);

    frame.render_widget(self.text_area.widget(), areas[0]);

    let status = Paragraph::new(self.status.clone())
      .wrap(Wrap::default())
      .block(Block::default().style(block_style).borders(Borders::ALL - Borders::TOP));
    frame.render_widget(status, areas[1]);
  }

  fn process_event(&mut self, event: Event) {
    self.modified |= self.text_area.input(event);
  }

  fn save_if_modified(&mut self) -> Result<(), io::Error> {
    if !self.modified {
      return Ok(());
    }
    let mut file = io::BufWriter::new(File::create(&self.path)?);
    for line in self.text_area.lines() {
      file.write_all(line.as_bytes())?;
      file.write_all(b"\n")?;
    }
    file.flush()?;
    self.modified = false;
    Ok(())
  }

  fn path(&self) -> &PathBuf { &self.path }

  fn status_mut(&mut self) -> &mut String { &mut self.status }
}
