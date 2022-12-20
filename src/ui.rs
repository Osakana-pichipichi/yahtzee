use crate::app::{App, CursorPos, GamePhase};
use crate::hand::Hand;
use crate::scoring::{box_name, scoring, Boxes};
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

#[rustfmt::skip]
/* Skip rustfmt for a clearer view of how it will appear on the screen */
const DICE_STR: [[&str; 3]; 6] = [
    [
        "     ",
        "  *  ",
        "     ",
    ],
    [
        " *   ",
        "     ",
        "   * "
    ],
    [
        " *   ",
        "  *  ",
        "   * ",
    ],
    [
        " * * ",
        "     ",
        " * * ",
    ],
    [
        " * * ",
        "  *  ",
        " * * ",
    ],
    [
        " * * ",
        " * * ",
        " * * ",
    ],
];

pub fn draw_play_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    /* Distribute the screen */
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.size());

    draw_playing_ground(f, app, chunks[0]);
    draw_score_table(f, app, chunks[1]);
}

fn draw_playing_ground<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(chunk);

    draw_role_block(f, app, chunks[0]);
    draw_hand_block(f, app, chunks[1]);
    draw_dust_block(f, app, chunks[2]);
}

fn draw_role_block<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let block = Block::default().title("Role").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let role_button_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, 13, 3));
    let text = Paragraph::new(Spans::from(Span::styled("Role!", Style::default())))
        .block(Block::default().borders(Borders::ALL))
        .style(match app.cursor_pos {
            CursorPos::Role => Style::default().fg(Color::DarkGray).bg(Color::White),
            _ => Style::default(),
        })
        .alignment(Alignment::Center);
    f.render_widget(text, role_button_chunk[0]);
}

fn draw_hand_block<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let block = Block::default().title("Dice").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let dice_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
        ])
        .split(create_centerd_rect(chunk, 39, 5));

    for (i, &d) in app.play.hand.get_dice().iter().enumerate() {
        let text = match (&app.play.game_phase, app.play.is_held[i]) {
            (GamePhase::Roll(..), ..) | (.., true) => vec![
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][0],
                    Style::default(),
                )),
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][1],
                    Style::default(),
                )),
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][2],
                    Style::default(),
                )),
            ],
            _ => vec![],
        };
        let text = Paragraph::new(text)
            .block(match (&app.play.game_phase, app.play.is_held[i]) {
                (GamePhase::Roll(..), ..) | (.., true) => Block::default().borders(Borders::ALL),
                _ => Block::default(),
            })
            .style(match app.cursor_pos {
                CursorPos::Hand(pos) => {
                    if i == pos {
                        Style::default().fg(Color::DarkGray).bg(Color::White)
                    } else {
                        Style::default()
                    }
                }

                _ => Style::default(),
            })
            .alignment(Alignment::Center);
        f.render_widget(text, dice_chunks[2 * i]);
    }
}

fn draw_dust_block<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let block = Block::default().title("Dust").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let dice_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
            Constraint::Length(1),
            Constraint::Length(7),
        ])
        .split(create_centerd_rect(chunk, 39, 5));

    for (i, &d) in app.play.hand.get_dice().iter().enumerate() {
        let text = match (&app.play.game_phase, app.play.is_held[i]) {
            (GamePhase::Roll(..), ..) | (.., true) => vec![],
            _ => vec![
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][0],
                    Style::default(),
                )),
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][1],
                    Style::default(),
                )),
                Spans::from(Span::styled(
                    DICE_STR[(d - 1) as usize][2],
                    Style::default(),
                )),
            ],
        };
        let text = Paragraph::new(text)
            .block(match (&app.play.game_phase, app.play.is_held[i]) {
                (GamePhase::Roll(..), ..) | (.., true) => Block::default(),
                _ => Block::default().borders(Borders::ALL),
            })
            .style(match app.cursor_pos {
                CursorPos::Dust(pos) => {
                    if i == pos {
                        Style::default().fg(Color::DarkGray).bg(Color::White)
                    } else {
                        Style::default()
                    }
                }

                _ => Style::default(),
            })
            .alignment(Alignment::Center);
        f.render_widget(text, dice_chunks[2 * i]);
    }
}

fn draw_score_table<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    let score_rows = enum_iterator::all::<Boxes>()
        .map(|b| {
            Row::new(
                vec![Cell::from(box_name(b).to_string())].into_iter().chain(
                    (0..app.num_players)
                        .map(|pid| {
                            let st = &app.scores[pid];
                            let hand = &app.play.hand;
                            let player = app.play.player_id;
                            let pos = CursorPos::Table(b);

                            let text = if st.is_filled(b) {
                                format!("{}", st.get_score(b))
                            } else if hand.get_dice().len() < Hand::DICE_NUM {
                                String::new()
                            } else if pid == player {
                                format!("{}", scoring(b, hand.get_dice()))
                            } else {
                                String::new()
                            };

                            let mut style = if app.cursor_pos == pos && pid == player {
                                Style::default().fg(Color::DarkGray).bg(Color::White)
                            } else {
                                Style::default()
                            };
                            style = if !st.is_filled(b) && pid == player {
                                style.fg(Color::Yellow)
                            } else {
                                style
                            };

                            Cell::from(text).style(style)
                        })
                        .collect::<Vec<_>>()
                        .into_iter(),
                ),
            )
        })
        .collect::<Vec<_>>();
    let score_header = Row::new(
        vec![String::from("")].into_iter().chain(
            (0..app.num_players)
                .map(|x| format!("Player{}", x))
                .collect::<Vec<_>>()
                .into_iter(),
        ),
    );
    let score_table_width = (0..(app.num_players + 1))
        .map(|x| Constraint::Length(if x == 0 { 20 } else { 7 }))
        .collect::<Vec<_>>();
    let score_block = Table::new(score_rows)
        .style(
            Style::default()
                .fg(Color::White)
                .remove_modifier(Modifier::BOLD),
        )
        .header(score_header)
        .block(Block::default().title("SCORE").borders(Borders::ALL))
        .widths(&score_table_width)
        .column_spacing(1)
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .fg(Color::DarkGray)
                .bg(Color::White),
        )
        .highlight_symbol(">>");

    f.render_widget(score_block, chunk);
}

fn create_centerd_rect(base_rect: Rect, width: u16, height: u16) -> Rect {
    Rect {
        x: base_rect.x + (base_rect.width - width) / 2,
        y: base_rect.y + (base_rect.height - height) / 2,
        width,
        height,
    }
}
