use std::{
    io::{self},
    sync::mpsc,
    thread,
    time::Duration,
};

use crossterm::{
    event::{KeyCode, KeyEventKind},
    style::Stylize,
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    symbols::border,
    text::{Line, Span},
    widgets::{Block, Widget},
};

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let mut app = App {
        exit: false,
        progress_bar_color: Color::Green,
        background_progress: 0.0,
    };

    let (tx, rx) = mpsc::channel::<Event>();

    let tx_clone = tx.clone();
    thread::spawn(move || {
        handle_input_event(tx_clone);
    });
    thread::spawn(move || {
        run_background_thread(tx);
    });
    let app_result = app.run(&mut terminal, rx);
    ratatui::restore();
    app_result
}

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
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_background_thread(tx: mpsc::Sender<Event>) {
    let mut progress = 0_f64;
    let increment = 0.01;
    loop {
        thread::sleep(Duration::from_millis(100));
        progress += increment;
        progress = progress.min(1_f64);
        tx.send(Event::Progress(progress)).unwrap();
    }
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.on_key_event(key_event)?,
                Event::Progress(progress) => self.background_progress = progress,
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn on_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
            self.exit = true;
            println!("{}", "Exiting application...".red());
        } else if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('c') {
            if self.progress_bar_color == Color::Green {
                self.progress_bar_color = Color::Yellow;
            } else {
                self.progress_bar_color = Color::Green;
            }
        }
        Ok(())
    }
}

impl Widget for &App {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
    where
        Self: Sized,
    {
        let vertical_layout =
            Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
        let [title_area, guage_area] = vertical_layout.areas(area);

        Line::raw("Process Overview").render(title_area, buf);

        let instructions = Line::from(vec![
            Span::styled("Change color", Style::default()),
            Span::styled(
                "<C>",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" Quit ", Style::default()),
            Span::styled(
                "<q>",
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
        .centered();

        let block = Block::bordered()
            .title(Line::from("Background Processes"))
            .title_bottom(instructions)
            .border_set(border::THICK);

        let progress_bar = ratatui::widgets::Gauge::default()
            .gauge_style(Style::default().fg(self.progress_bar_color))
            .block(block)
            .label(format!(
                "Process 1: {:.0}%",
                self.background_progress * 100.0
            ))
            .ratio(self.background_progress);

        progress_bar.render(
            Rect {
                x: guage_area.left(),
                y: guage_area.top(),
                width: guage_area.width,
                height: 3,
            },
            buf,
        );
    }
}

