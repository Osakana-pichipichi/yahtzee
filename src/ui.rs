use crate::app::{App, AppState, AppStateError, CursorPos, GamePhase};
use crate::hand::Hand;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table},
    Frame,
};

const DICE_KINDS: usize = 6;
const DICE_STR_HEIGHT: usize = 3;

#[rustfmt::skip]
/* Skip rustfmt for a clearer view of how it will appear on the screen */
const DICE_STR: [[&str; DICE_STR_HEIGHT]; DICE_KINDS] = [
    [
        "       ",
        "   *   ",
        "       ",
    ],
    [
        " *     ",
        "       ",
        "     * ",
    ],
    [
        " *     ",
        "   *   ",
        "     * ",
    ],
    [
        " *   * ",
        "       ",
        " *   * ",
    ],
    [
        " *   * ",
        "   *   ",
        " *   * ",
    ],
    [
        " *   * ",
        " *   * ",
        " *   * ",
    ],
];

const DICE_STR_WIDTH: usize = {
    let width = DICE_STR[0][0].len();

    /* check the length for each elemnts */
    let mut d = 0;
    while d < DICE_KINDS {
        let mut h = 0;
        while h < DICE_STR_HEIGHT {
            if DICE_STR[d][h].len() != width {
                panic!("length of dice strings are not aligned.");
            }
            h += 1;
        }
        d += 1;
    }

    width
};
const BOXES_CELL_WIDTH: usize = 20;
const SCORE_CELL_WIDTH: usize = 11;

const DICE_MARGIN: u16 = 1;
const HAND_MARGIN: u16 = 1;
const DUST_MARGIN: u16 = 1;

pub fn draw_ui(f: &mut Frame, app: &App) {
    match app.state {
        AppState::Play(..) => draw_play_ui(f, app),
        AppState::Result => draw_result_ui(f, app),
    }
}

fn draw_play_ui(f: &mut Frame, app: &App) {
    /* Distribute the screen */
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    draw_playing_ground(f, app, chunks[0]);
    draw_score_table(f, app, chunks[1]);
}

fn draw_playing_ground(f: &mut Frame, app: &App, chunk: Rect) {
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

fn draw_role_block(f: &mut Frame, app: &App, chunk: Rect) {
    let block = Block::default().title("Role").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let role_button_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, 13, 3));
    let text = Paragraph::new(Line::from(Span::styled("Role!", Style::default())))
        .block(Block::default().borders(Borders::ALL))
        .style(match app.cursor_pos {
            CursorPos::Role => Style::default().fg(Color::DarkGray).bg(Color::White),
            _ => Style::default(),
        })
        .alignment(Alignment::Center);
    f.render_widget(text, role_button_chunk[0]);
}

fn draw_hand_block(f: &mut Frame, app: &App, chunk: Rect) {
    let block = Block::default().title("Dice").borders(Borders::ALL);
    f.render_widget(block, chunk);

    match app.get_play_data() {
        Ok(play) => {
            for (i, &d) in play.hand.get_dice().iter().enumerate() {
                let dice_width = DICE_STR_WIDTH as u16 + DICE_MARGIN * 2;
                let dice_num = Hand::DICE_NUM as u16;
                let rect_width = dice_width * dice_num + HAND_MARGIN * (dice_num - 1);
                let rect_height = DICE_STR_HEIGHT as u16 + HAND_MARGIN * 2;
                let dice_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(dice_width),
                        Constraint::Length(HAND_MARGIN),
                        Constraint::Length(dice_width),
                        Constraint::Length(HAND_MARGIN),
                        Constraint::Length(dice_width),
                        Constraint::Length(HAND_MARGIN),
                        Constraint::Length(dice_width),
                        Constraint::Length(HAND_MARGIN),
                        Constraint::Length(dice_width),
                    ])
                    .split(create_centerd_rect(chunk, rect_width, rect_height));

                let text = match (&play.game_phase, play.is_held[i]) {
                    (GamePhase::Roll(..), ..) | (.., true) => (0..DICE_STR_HEIGHT)
                        .map(|h| {
                            Line::from(Span::styled(
                                DICE_STR[(d - 1) as usize][h],
                                Style::default(),
                            ))
                        })
                        .collect(),
                    _ => vec![],
                };
                let text = Paragraph::new(text)
                    .block(match (&play.game_phase, play.is_held[i]) {
                        (GamePhase::Roll(..), ..) | (.., true) => {
                            Block::default().borders(Borders::ALL)
                        }
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
        Err(e) => match e.downcast_ref::<AppStateError>().unwrap() {
            AppStateError::NoPlayData | AppStateError::UnexpectedState => {}
            _ => panic!("{}", e),
        },
    }
}

fn draw_dust_block(f: &mut Frame, app: &App, chunk: Rect) {
    let block = Block::default().title("Dust").borders(Borders::ALL);
    f.render_widget(block, chunk);

    match app.get_play_data() {
        Ok(play) => {
            let dice_width = DICE_STR_WIDTH as u16 + DICE_MARGIN * 2;
            let dice_num = Hand::DICE_NUM as u16;
            let rect_width = dice_width * dice_num + DUST_MARGIN * (dice_num - 1);
            let rect_height = DICE_STR_HEIGHT as u16 + DUST_MARGIN * 2;
            let dice_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(dice_width),
                    Constraint::Length(DUST_MARGIN),
                    Constraint::Length(dice_width),
                    Constraint::Length(DUST_MARGIN),
                    Constraint::Length(dice_width),
                    Constraint::Length(DUST_MARGIN),
                    Constraint::Length(dice_width),
                    Constraint::Length(DUST_MARGIN),
                    Constraint::Length(dice_width),
                ])
                .split(create_centerd_rect(chunk, rect_width, rect_height));

            for (i, &d) in play.hand.get_dice().iter().enumerate() {
                let text = match (&play.game_phase, play.is_held[i]) {
                    (GamePhase::Roll(..), ..) | (.., true) => vec![],
                    _ => (0..DICE_STR_HEIGHT)
                        .map(|h| {
                            Line::from(Span::styled(
                                DICE_STR[(d - 1) as usize][h],
                                Style::default(),
                            ))
                        })
                        .collect(),
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
        Err(e) => match e.downcast_ref::<AppStateError>().unwrap() {
            AppStateError::NoPlayData | AppStateError::UnexpectedState => {}
            _ => panic!("{}", e),
        },
    }
}

fn draw_score_table(f: &mut Frame, app: &App, chunk: Rect) {
    let is_playing = |pid: usize| -> bool {
        match app.get_play_data() {
            Ok(play) => pid == play.player_id,
            _ => false,
        }
    };
    let dislay_dice = |pid: usize| -> Option<&[u32]> {
        match app.get_play_data() {
            Ok(p) if is_playing(pid) => Some(p.hand.get_dice()),
            _ => None,
        }
    };

    let mut score_rows = enum_iterator::all::<Boxes>()
        .map(|b| {
            Row::new(
                vec![Cell::from(format!("{:>1$}", b, BOXES_CELL_WIDTH))]
                    .into_iter()
                    .chain(
                        (0..app.get_game_data().get_num_players())
                            .map(|pid| {
                                let st = &app.get_game_data().get_score_table(pid);
                                let is_playing = is_playing(pid);
                                let dice = dislay_dice(pid);
                                let pos = CursorPos::Table(b);

                                let text = if st.has_score_in(b) {
                                    format!("{:>1$}", st.get_score(b).unwrap(), SCORE_CELL_WIDTH)
                                } else if let Some(d) = dice {
                                    format!("{:>1$}", scoring(b, d), SCORE_CELL_WIDTH)
                                } else {
                                    String::new()
                                };

                                let style = if is_playing && !st.has_score_in(b) {
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
            .chain((0..app.get_game_data().get_num_players()).map(|pid| {
                let st = &app.get_game_data().get_score_table(pid);
                let is_playing = is_playing(pid);
                let dice = dislay_dice(pid);
                let pos = &app.cursor_pos;

                let mut bstext = format!("{:>2}", "");
                let mut bsstyle = Style::default();
                let us = st.get_total_upper_score();
                let mut ustext = format!("{:>3}", us);
                let mut usstyle = Style::default();

                if let Some(score) = st.calculate_bonus() {
                    bstext = format!("{:>2}", score);
                } else if let (&CursorPos::Table(b), true, Some(d)) = (pos, is_playing, dice) {
                    let score = scoring(b, d);
                    if let Some(score) = st.calculate_bonus_if_filled_by(b, score) {
                        bstext = format!("{:>2}", score);
                        bsstyle = bsstyle.fg(Color::Rgb(255, 215, 0));
                    }
                }

                if let (&CursorPos::Table(b), true, Some(d)) = (pos, is_playing, dice) {
                    let score = scoring(b, d);
                    let ifus = st.get_total_upper_score_if_filled_by(b, score);
                    if ifus > us {
                        ustext = format!("{:>3}", ifus);
                        usstyle = usstyle.fg(Color::Rgb(255, 215, 0));
                    }
                }

                Cell::from(Line::from(vec![
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
            .chain((0..app.get_game_data().get_num_players()).map(|pid| {
                let st = &app.get_game_data().get_score_table(pid);
                let is_playing = is_playing(pid);
                let dice = dislay_dice(pid);
                let pos = &app.cursor_pos;
                let total_score = st.get_total_score();
                let mut text = format!("{:>1$}", total_score, SCORE_CELL_WIDTH);
                let mut style = Style::default();
                if let (&CursorPos::Table(b), true, Some(d)) = (pos, is_playing, dice) {
                    let score = scoring(b, d);

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
            (0..app.get_game_data().get_num_players())
                .map(|pid| {
                    let text = format!("{:^1$}", format!("Player{}", pid), SCORE_CELL_WIDTH);
                    let style = if is_playing(pid) {
                        Style::default().fg(Color::Black).bg(Color::LightYellow)
                    } else {
                        Style::default()
                    };
                    Cell::from(text).style(style)
                })
                .collect::<Vec<_>>(),
        ),
    );
    let score_table_width = (0..(app.get_game_data().get_num_players() + 1))
        .map(|x| {
            Constraint::Length(if x == 0 {
                BOXES_CELL_WIDTH as u16
            } else {
                SCORE_CELL_WIDTH as u16
            })
        })
        .collect::<Vec<_>>();
    let score_block = Table::new(score_rows, &score_table_width)
        .style(
            Style::default()
                .fg(Color::White)
                .remove_modifier(Modifier::BOLD),
        )
        .header(score_header)
        .block(Block::default().title("SCORE").borders(Borders::ALL))
        .column_spacing(1);

    f.render_widget(score_block, chunk);
}

fn draw_result_ui(f: &mut Frame, app: &App) {
    /* Distribute the screen */
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(f.size());

    draw_result(f, app, chunks[0]);
    draw_result_score_table(f, app, chunks[1]);
}

fn draw_result(f: &mut Frame, app: &App, chunk: Rect) {
    match app.state {
        AppState::Result => (),
        _ => panic!(),
    }

    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunk);

    let height = (app.get_game_data().get_num_players() as u16) + 2;
    let width = chunk.width - 4;

    let text_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, width + 2, height + 2));

    let mut results = (0..app.get_game_data().get_num_players())
        .map(|i| (i, app.get_game_data().get_score_table(i).get_total_score()))
        .collect::<Vec<_>>();
    results.sort_by(|(.., left), (.., right)| left.cmp(right).reverse());
    let mut results = results
        .iter()
        .map(|(pid, score)| {
            let rank = results.iter().position(|(.., s)| s == score).unwrap() + 1;
            Line::from(Span::styled(
                format!("{:^1$}", format!("{}. Player{}", rank, pid), width as usize),
                Style::default(),
            ))
        })
        .collect::<Vec<_>>();
    results.extend([
        Line::from(Span::raw("")),
        Line::from(Span::styled(
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

fn draw_result_score_table(f: &mut Frame, app: &App, chunk: Rect) {
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
                        (0..app.get_game_data().get_num_players())
                            .map(|pid| {
                                let st = app.get_game_data().get_score_table(pid);
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
            .chain((0..app.get_game_data().get_num_players()).map(|pid| {
                let st = &app.get_game_data().get_score_table(pid);
                let bstext = format!("{:>2}", st.calculate_bonus().unwrap());
                let ustext = format!("{:>3}", st.get_total_upper_score());

                Cell::from(Line::from(vec![
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
            .chain((0..app.get_game_data().get_num_players()).map(|pid| {
                let st = &app.get_game_data().get_score_table(pid);
                let total_score = st.get_total_score();
                let text = format!("{:>1$}", total_score, SCORE_CELL_WIDTH);

                Cell::from(text).style(Style::default())
            })),
    );
    score_rows.push(total_cell);
    let score_header = Row::new(
        vec![Cell::from(String::from(""))].into_iter().chain(
            (0..app.get_game_data().get_num_players())
                .map(|pid| {
                    let text = format!("{:^1$}", format!("Player{}", pid), SCORE_CELL_WIDTH);

                    Cell::from(text).style(Style::default())
                })
                .collect::<Vec<_>>(),
        ),
    );
    let score_table_width = (0..(app.get_game_data().get_num_players() + 1))
        .map(|x| {
            Constraint::Length(if x == 0 {
                BOXES_CELL_WIDTH as u16
            } else {
                SCORE_CELL_WIDTH as u16
            })
        })
        .collect::<Vec<_>>();
    let score_block = Table::new(score_rows, &score_table_width)
        .style(
            Style::default()
                .fg(Color::White)
                .remove_modifier(Modifier::BOLD),
        )
        .header(score_header)
        .block(Block::default().title("SCORE").borders(Borders::ALL))
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
