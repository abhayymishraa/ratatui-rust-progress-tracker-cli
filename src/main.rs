use std::{io, sync::mpsc, thread, time::Duration};

use crossterm::{
    event::{KeyCode, KeyEventKind},
    style::Stylize,
};

use ratatui::{
    DefaultTerminal,
    layout::{Constraint, Layout, Rect},
    prelude::*,
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Gauge, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App::default();

    let (tx, rx) = mpsc::channel::<Event>();

    let tx_input = tx.clone();
    thread::spawn(move || handle_input_event(tx_input));

    thread::spawn(move || run_background_thread(tx));

    let result = app.run(&mut terminal, rx);
    ratatui::restore();
    result
}

#[derive(Default)]
pub struct App {
    exit: bool,
    progress_bar_color: Color,
    background_progress: f64,
}

enum Event {
    Input(crossterm::event::KeyEvent),
    Progress(f64),
}

fn handle_input_event(tx: mpsc::Sender<Event>) {
    while let Ok(event) = crossterm::event::read() {
        if let crossterm::event::Event::Key(key_event) = event {
            let _ = tx.send(Event::Input(key_event));
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress: f64 = 0.0;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress = (progress + 0.01).min(1.0);
        let _ = tx.send(Event::Progress(progress));
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|f| self.draw(f))?;
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key(key_event)?,
                Event::Progress(p) => self.background_progress = p,
            }
        }
        Ok(())
    }

    fn handle_key(&mut self, key: crossterm::event::KeyEvent) -> io::Result<()> {
        if key.kind == KeyEventKind::Press {
            match key.code {
                KeyCode::Char('q') => {
                    self.exit = true;
                    println!("{}", "Exiting application...".red());
                }
                KeyCode::Char('c') => {
                    self.progress_bar_color = match self.progress_bar_color {
                        Color::Green => Color::Yellow,
                        _ => Color::Green,
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([Constraint::Length(3), Constraint::Min(5)]);
        let [title_area, gauge_area] = layout.areas(area);

        self.draw_title(title_area, buf);
        self.draw_progress_bar(gauge_area, buf);
    }
}

impl App {
    fn draw_title(&self, area: Rect, buf: &mut Buffer) {
        Line::from(vec![Span::styled(
            "🛠️  Process Overview",
            Style::default().add_modifier(Modifier::BOLD),
        )])
        .centered()
        .render(area, buf);
    }

    fn draw_progress_bar(&self, area: Rect, buf: &mut Buffer) {
        let instructions = Line::from(vec![
            Span::styled("Press ", Style::default()),
            Span::styled(
                "C",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to toggle color | ", Style::default()),
            Span::styled(
                "Q",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to quit", Style::default()),
        ])
        .centered();

        let block = Block::bordered()
            .title("Background Processes")
            .title_bottom(instructions)
            .border_set(border::THICK)
            .style(Style::default());

        let gauge = Gauge::default()
            .block(block)
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .label(Span::styled(
                format!("{:.0}%", self.background_progress * 100.0),
                Style::default().add_modifier(Modifier::BOLD),
            ))
            .ratio(self.background_progress);

        gauge.render(
            Rect {
                x: area.left(),
                y: area.top(),
                width: area.width,
                height: 3,
            },
            buf,
        );
    }
}
