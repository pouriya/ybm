use crate::{
    otp::{maybe_update_otps, update_otps},
    settings::{Account, Config, Theme},
};
use anyhow::Context;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, Paragraph, Widget},
    Frame, Terminal,
};
use std::{collections::HashMap, io, time::Duration};

pub fn run(config: &mut Config) -> anyhow::Result<()> {
    enable_raw_mode().context("Could not enable terminal raw mode")?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen).context("Could not prepare to setup terminal")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Could not setup terminal")?;
    let result = event_loop(&mut terminal, config);
    disable_raw_mode().context("Could not disable terminal raw mode")?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)
        .context("Could not undo terminal setup")?;
    terminal.show_cursor().context("Could not show cursor")?;
    result
}

fn event_loop<B: Backend>(terminal: &mut Terminal<B>, config: &mut Config) -> anyhow::Result<()> {
    let event_timeout = Duration::from_millis(100);
    let max_length = config
        .accounts
        .iter()
        .fold(0, |max_length, (name, account)| {
            let length = name.len() + account.length as usize;
            if length > max_length {
                length
            } else {
                max_length
            }
        });
    update_otps(config.accounts.iter_mut());
    loop {
        let (seconds, percentage) = maybe_update_otps(config.accounts.iter_mut());
        terminal
            .draw(|frame| {
                ui(
                    &config.accounts,
                    &config.theme,
                    percentage,
                    seconds,
                    max_length,
                    frame,
                )
            })
            .context("Could not draw to TUI")?;
        if event::poll(event_timeout).context("Could not wait for next event")? {
            if let Event::Key(key) = event::read().context("Could not read next event")? {
                if [KeyCode::Char('q'), KeyCode::Esc].contains(&key.code) {
                    return Ok(());
                }
            }
        }
    }
}

fn ui(
    accounts: &HashMap<String, Account>,
    theme: &Theme,
    percentage: f64,
    _seconds: f64,
    max_length: usize,
    frame: &mut Frame,
) {
    let size = frame.size();
    let vertical_layout = Layout::vertical([Constraint::Fill(1), Constraint::Max(1)]);
    let [accounts_space, gauge_space] = vertical_layout.areas(size);

    let row_count = accounts_space.rows().count();
    let mut lines = if row_count > 2 && (row_count / 2) > accounts.len() {
        (0..(row_count / 2) - accounts.len())
            .map(|_| Line::raw(""))
            .collect()
    } else {
        Vec::new()
    };
    lines.append(
        &mut accounts
            .iter()
            .map(|(name, account)| {
                let spaces = " ".repeat(max_length - (name.len() + account.length as usize) + 1);
                Line::from(vec![
                    Span::styled(
                        name.to_uppercase(),
                        Style::new()
                            .fg(theme.name.into())
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(spaces, Style::new()),
                    Span::styled(
                        account.code.to_string(),
                        Style::new().fg(if percentage < 5.0 {
                            Color::Red
                        } else if percentage < 10.0 {
                            Color::LightRed
                        } else if percentage < 20.0 {
                            Color::Yellow
                        } else {
                            theme.code.into()
                        }),
                    )
                    .add_modifier(if percentage < 10.0 {
                        Modifier::RAPID_BLINK
                    } else if percentage < 20.0 {
                        Modifier::SLOW_BLINK
                    } else {
                        Modifier::HIDDEN
                    }),
                ])
            })
            .collect(),
    );
    let paragraph = Paragraph::new(lines)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .style(Style::default()),
        )
        .centered();
    paragraph.render(accounts_space, frame.buffer_mut());
    Gauge::default()
        .label(Span::default())
        .block(Block::default().borders(Borders::LEFT | Borders::RIGHT))
        .gauge_style(if percentage < 5.0 {
            Color::Red
        } else if percentage < 10.0 {
            Color::LightRed
        } else if percentage < 20.0 {
            Color::Yellow
        } else {
            Color::Green
        })
        .ratio(percentage / 100.0)
        .render(gauge_space, frame.buffer_mut());
}
