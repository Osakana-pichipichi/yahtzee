use crate::app::{App, AppState, CursorPos, GamePhase};
use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
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

const BOXES_CELL_WIDTH: usize = 20;
const SCORE_CELL_WIDTH: usize = 11;

pub fn draw_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    match app.state {
        AppState::Play(..) => draw_play_ui(f, app),
        AppState::Result => draw_result_ui(f, app),
    }
}

fn draw_play_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
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
    let play = if let AppState::Play(play) = &app.state {
        play
    } else {
        panic!()
    };

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

    for (i, &d) in play.hand.get_dice().iter().enumerate() {
        let text = match (&play.game_phase, play.is_held[i]) {
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
            .block(match (&play.game_phase, play.is_held[i]) {
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
    let play = if let AppState::Play(play) = &app.state {
        play
    } else {
        panic!()
    };

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

    for (i, &d) in play.hand.get_dice().iter().enumerate() {
        let text = match (&play.game_phase, play.is_held[i]) {
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
            .block(match (&play.game_phase, play.is_held[i]) {
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
    let play = if let AppState::Play(play) = &app.state {
        play
    } else {
        panic!()
    };

    let mut score_rows = enum_iterator::all::<Boxes>()
        .map(|b| {
            Row::new(
                vec![Cell::from(format!("{:>1$}", b, BOXES_CELL_WIDTH))]
                    .into_iter()
                    .chain(
                        (0..app.num_players)
                            .map(|pid| {
                                let st = &app.scores[pid];
                                let hand = &play.hand;
                                let player = play.player_id;
                                let pos = CursorPos::Table(b);

                                let text = if st.has_score_in(b) {
                                    format!("{:>1$}", st.get_score(b).unwrap(), SCORE_CELL_WIDTH)
                                } else if hand.get_dice().len() < Hand::DICE_NUM {
                                    String::new()
                                } else if pid == player {
                                    format!("{:>1$}", scoring(b, hand.get_dice()), SCORE_CELL_WIDTH)
                                } else {
                                    String::new()
                                };

                                let style = if pid == player && !st.has_score_in(b) {
                                    let mut style = Style::default().fg(Color::Rgb(255, 215, 0));
                                    if app.cursor_pos == pos {
                                        style = style.fg(Color::Black).bg(Color::Rgb(255, 215, 0));
                                    }
                                    style
                                } else {
                                    Style::default()
                                };

                                Cell::from(text).style(style)
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
        })
        .collect::<Vec<_>>();
    let bonus_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Bonus", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain((0..app.num_players).map(|pid| {
                let dice = play.hand.get_dice();
                let st = &app.scores[pid];
                let is_playing = pid == play.player_id;

                let mut bstext = format!("{:>2}", "");
                let mut bsstyle = Style::default();
                let us = st.get_total_upper_score();
                let mut ustext = format!("{:>3}", us);
                let mut usstyle = Style::default();

                if let Some(score) = st.calculate_bonus() {
                    bstext = format!("{:>2}", score);
                } else if let (&CursorPos::Table(b), true) = (&app.cursor_pos, is_playing) {
                    let score = scoring(b, dice);
                    if let Some(score) = st.calculate_bonus_if_filled_by(b, score) {
                        bstext = format!("{:>2}", score);
                        bsstyle = bsstyle.fg(Color::Rgb(255, 215, 0));
                    }
                }

                if let (&CursorPos::Table(b), true) = (&app.cursor_pos, is_playing) {
                    let score = scoring(b, dice);
                    let ifus = st.get_total_upper_score_if_filled_by(b, score);
                    if ifus > us {
                        ustext = format!("{:>3}", ifus);
                        usstyle = usstyle.fg(Color::Rgb(255, 215, 0));
                    }
                }

                Cell::from(Spans::from(vec![
                    Span::styled(bstext, bsstyle),
                    Span::raw(" ("),
                    Span::styled(ustext, usstyle),
                    Span::raw(format!("/{:>2})", ScoreTable::BONUS_THRESHOLD)),
                ]))
            })),
    );
    score_rows.push(bonus_cell);
    let total_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Total", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain((0..app.num_players).map(|pid| {
                let dice = play.hand.get_dice();
                let st = &app.scores[pid];
                let is_playing = pid == play.player_id;
                let total_score = st.get_total_score();
                let mut text = format!("{:>1$}", total_score, SCORE_CELL_WIDTH);
                let mut style = Style::default();
                if let (&CursorPos::Table(b), true) = (&app.cursor_pos, is_playing) {
                    let score = scoring(b, dice);

                    let if_total_score = st.get_total_score_if_filled_by(b, score);
                    if if_total_score > total_score {
                        text = format!("{:>1$}", if_total_score, SCORE_CELL_WIDTH);
                        style = style.fg(Color::Rgb(255, 215, 0));
                    }
                }

                Cell::from(text).style(style)
            })),
    );
    score_rows.push(total_cell);
    let score_header = Row::new(
        vec![Cell::from(String::from(""))].into_iter().chain(
            (0..app.num_players)
                .map(|pid| {
                    let text = format!("{:^1$}", format!("Player{}", pid), SCORE_CELL_WIDTH);
                    let style = if pid == play.player_id {
                        Style::default().fg(Color::Black).bg(Color::LightYellow)
                    } else {
                        Style::default()
                    };
                    Cell::from(text).style(style)
                })
                .collect::<Vec<_>>(),
        ),
    );
    let score_table_width = (0..(app.num_players + 1))
        .map(|x| {
            Constraint::Length(if x == 0 {
                BOXES_CELL_WIDTH as u16
            } else {
                SCORE_CELL_WIDTH as u16
            })
        })
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
        .column_spacing(1);

    f.render_widget(score_block, chunk);
}

fn draw_result_ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    /* Distribute the screen */
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.size());

    draw_result(f, app, chunks[0]);
    draw_result_score_table(f, app, chunks[1]);
}

fn draw_result<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    match app.state {
        AppState::Result => (),
        _ => panic!(),
    }

    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunk);

    let height = (app.num_players as u16) + 2;
    let width = chunk.width - 4;

    let text_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, width + 2, height + 2));

    let mut results = (0..app.num_players)
        .map(|i| (i, app.scores[i].get_total_score()))
        .collect::<Vec<_>>();
    results.sort_by(|(.., left), (.., right)| left.cmp(right).reverse());
    let mut results = results
        .iter()
        .map(|(pid, score)| {
            let rank = results.iter().position(|(.., s)| s == score).unwrap() + 1;
            Spans::from(Span::styled(
                format!("{:^1$}", format!("{}. Player{}", rank, pid), width as usize),
                Style::default(),
            ))
        })
        .collect::<Vec<_>>();
    results.extend([
        Spans::from(Span::raw("")),
        Spans::from(Span::styled(
            format!("{:^1$}", "Press ENTER to exit.", width as usize),
            Style::default(),
        )),
    ]);
    let text = Paragraph::new(results)
        .block(Block::default().borders(Borders::ALL))
        .style(match app.cursor_pos {
            CursorPos::Role => Style::default().fg(Color::DarkGray).bg(Color::White),
            _ => Style::default(),
        })
        .alignment(Alignment::Center);
    f.render_widget(text, text_chunk[0]);
}

fn draw_result_score_table<B: Backend>(f: &mut Frame<B>, app: &App, chunk: Rect) {
    match app.state {
        AppState::Result => (),
        _ => panic!(),
    }

    let mut score_rows = enum_iterator::all::<Boxes>()
        .map(|b| {
            Row::new(
                vec![Cell::from(format!("{:>1$}", b, BOXES_CELL_WIDTH))]
                    .into_iter()
                    .chain(
                        (0..app.num_players)
                            .map(|pid| {
                                let st = &app.scores[pid];
                                let score = st.get_score(b).unwrap();
                                let text = format!("{:>1$}", score, SCORE_CELL_WIDTH);

                                Cell::from(text).style(Style::default())
                            })
                            .collect::<Vec<_>>(),
                    ),
            )
        })
        .collect::<Vec<_>>();
    let bonus_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Bonus", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain((0..app.num_players).map(|pid| {
                let st = &app.scores[pid];
                let bstext = format!("{:>2}", st.calculate_bonus().unwrap());
                let ustext = format!("{:>3}", st.get_total_upper_score());

                Cell::from(Spans::from(vec![
                    Span::styled(bstext, Style::default()),
                    Span::raw(" ("),
                    Span::styled(ustext, Style::default()),
                    Span::raw(format!("/{:>2})", ScoreTable::BONUS_THRESHOLD)),
                ]))
            })),
    );
    score_rows.push(bonus_cell);
    let total_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Total", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain((0..app.num_players).map(|pid| {
                let st = &app.scores[pid];
                let total_score = st.get_total_score();
                let text = format!("{:>1$}", total_score, SCORE_CELL_WIDTH);

                Cell::from(text).style(Style::default())
            })),
    );
    score_rows.push(total_cell);
    let score_header = Row::new(
        vec![Cell::from(String::from(""))].into_iter().chain(
            (0..app.num_players)
                .map(|pid| {
                    let text = format!("{:^1$}", format!("Player{}", pid), SCORE_CELL_WIDTH);

                    Cell::from(text).style(Style::default())
                })
                .collect::<Vec<_>>(),
        ),
    );
    let score_table_width = (0..(app.num_players + 1))
        .map(|x| {
            Constraint::Length(if x == 0 {
                BOXES_CELL_WIDTH as u16
            } else {
                SCORE_CELL_WIDTH as u16
            })
        })
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
        .column_spacing(1);

    f.render_widget(score_block, chunk);
}

fn create_centerd_rect(base_rect: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        base_rect.x + (base_rect.width - width) / 2,
        base_rect.y + (base_rect.height - height) / 2,
        width,
        height,
    )
}
