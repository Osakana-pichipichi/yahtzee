use crate::app::{
    App, AppState, AppStateError, NumPlayersSelection, PlayCursorPos, StartMenuSelection,
    HIGHEST_PLAYER_ID, LOWEST_PLAYER_ID,
};
use crate::assets;
use crate::hand::{Hand, HandOpError};
use crate::play::PlayPhase;
use crate::score_table::ScoreTable;
use crate::scoring::{scoring, Boxes};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
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

const FRAME_MARGIN: u16 = 1;

const SELECTION_MARGIN: u16 = 1;

const DICE_MARGIN: u16 = 1;
const HAND_MARGIN: u16 = 1;
const DUST_MARGIN: u16 = 1;

pub fn draw_ui(f: &mut Frame, app: &App) {
    match app.get_state() {
        AppState::StartMenu(..) => draw_start_menu(f, app),
        AppState::SelectNumPlayers(..) => draw_select_number_of_players(f, app),
        AppState::Play(..) => draw_play_ui(f, app),
        AppState::Result => draw_result_ui(f, app),
    }
}

fn draw_start_menu(f: &mut Frame, app: &App) {
    let chunk = drwa_logo_and_frame(f);
    draw_start_menu_selections(f, app, chunk);
}

fn draw_select_number_of_players(f: &mut Frame, app: &App) {
    let chunk = drwa_logo_and_frame(f);
    draw_selections_for_number_of_players(f, app, chunk);
}

fn drwa_logo_and_frame(f: &mut Frame) -> Rect {
    /* Draw frame border */
    let chunk = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .split(f.size());
    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunk[0]);

    /* Distribute the screen */
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .margin(FRAME_MARGIN)
        .split(chunk[0]);

    draw_logo(f, chunks[0]);

    chunks[1]
}

fn draw_logo(f: &mut Frame, chunk: Rect) {
    let logo_chunk = Layout::default()
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(
            chunk,
            chunk.width,
            assets::MENU_LOGO_HEIGHT as u16,
        ));
    let lines: Vec<_> = assets::MENU_LOGO_STR
        .iter()
        .map(|&s| Line::from(Span::styled(s, Style::default())))
        .collect();
    let text = Paragraph::new(lines).alignment(Alignment::Center);
    f.render_widget(text, logo_chunk[0]);
}

fn draw_start_menu_selections(f: &mut Frame, app: &App, chunk: Rect) {
    let AppState::StartMenu(pos) = app.get_state() else {
        panic!("Unexpected state")
    };
    let choices = [StartMenuSelection::Play, StartMenuSelection::Exit];
    let choices: Vec<_> = choices
        .iter()
        .map(|c| {
            Line::from(Span::styled(
                format!("{}", c),
                if pos == c {
                    Style::default().fg(Color::DarkGray).bg(Color::White)
                } else {
                    Style::default()
                },
            ))
        })
        .collect();
    draw_selections(f, chunk, choices);
}

fn draw_selections_for_number_of_players(f: &mut Frame, app: &App, chunk: Rect) {
    let AppState::SelectNumPlayers(pos) = app.get_state() else {
        panic!("Unexpected state")
    };
    let choices: Vec<_> = (LOWEST_PLAYER_ID..=HIGHEST_PLAYER_ID)
        .map(NumPlayersSelection::NumPlayers)
        .chain([NumPlayersSelection::Back])
        .collect();
    let choices: Vec<_> = choices
        .iter()
        .map(|c| {
            Line::from(Span::styled(
                format!("{}", c),
                if pos == c {
                    Style::default().fg(Color::DarkGray).bg(Color::White)
                } else {
                    Style::default()
                },
            ))
        })
        .collect();
    draw_selections(f, chunk, choices);
}

fn draw_selections<'a, I>(f: &mut Frame, chunk: Rect, choices: I)
where
    I: IntoIterator,
    I::Item: Into<Text<'a>>,
{
    let choices: Vec<_> = choices.into_iter().map(|c| c.into()).collect();
    let choice_height = 1;
    let max_choice_str_len = choices.iter().map(|c| c.width()).max().unwrap() as u16;
    let rect_width = max_choice_str_len + SELECTION_MARGIN * 2;
    let choice_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints((0..(2 * choices.len() - 1)).map(|_| Constraint::Length(choice_height)))
        .split(create_centerd_rect(chunk, rect_width, chunk.height));

    for (choice, &chunk) in choices.into_iter().zip(choice_chunks.iter().step_by(2)) {
        let text = Paragraph::new(choice)
            .block(Block::default())
            .alignment(Alignment::Center);
        f.render_widget(text, chunk);
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

    draw_roll_block(f, app, chunks[0]);
    draw_hand_block(f, app, chunks[1]);
    draw_dust_block(f, app, chunks[2]);
}

fn draw_roll_block(f: &mut Frame, app: &App, chunk: Rect) {
    let block = Block::default().title("Roll").borders(Borders::ALL);
    f.render_widget(block, chunk);

    let roll_button_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, 13, 3));
    let text = Paragraph::new(Line::from(Span::styled("Roll!", Style::default())))
        .block(Block::default().borders(Borders::ALL))
        .style(match app.get_state().get_play_cursor_pos().unwrap() {
            PlayCursorPos::Roll => Style::default().fg(Color::DarkGray).bg(Color::White),
            _ => Style::default(),
        })
        .alignment(Alignment::Center);
    f.render_widget(text, roll_button_chunk[0]);
}

fn draw_hand_block(f: &mut Frame, app: &App, chunk: Rect) {
    let block = Block::default().title("Dice").borders(Borders::ALL);
    f.render_widget(block, chunk);

    match app.get_state().get_play_data() {
        Ok(play) => {
            let dice = HandOpError::unwrap_pips(play.get_hand().get_pips());
            for (i, d) in dice.iter().enumerate() {
                let hand = play.get_hand();
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

                let text = match (play.get_phase(), hand.is_held(i).unwrap()) {
                    (PlayPhase::Roll(..), ..) | (.., true) => (0..DICE_STR_HEIGHT)
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
                    .block(match (play.get_phase(), hand.is_held(i).unwrap()) {
                        (PlayPhase::Roll(..), ..) | (.., true) => {
                            Block::default().borders(Borders::ALL)
                        }
                        _ => Block::default(),
                    })
                    .style(match app.get_state().get_play_cursor_pos().unwrap() {
                        &PlayCursorPos::Hand(pos) => {
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

    match app.get_state().get_play_data() {
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

            let dice = HandOpError::unwrap_pips(play.get_hand().get_pips());
            for (i, d) in dice.iter().enumerate() {
                let hand = play.get_hand();
                let text = match (play.get_phase(), hand.is_held(i).unwrap()) {
                    (PlayPhase::Roll(..), ..) | (.., true) => vec![],
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
                    .block(match (play.get_phase(), hand.is_held(i).unwrap()) {
                        (PlayPhase::Roll(..), ..) | (.., true) => Block::default(),
                        _ => Block::default().borders(Borders::ALL),
                    })
                    .style(match app.get_state().get_play_cursor_pos().unwrap() {
                        &PlayCursorPos::Dust(pos) => {
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
        match app.get_state().get_play_data() {
            Ok(play) => pid == play.get_player_id(),
            _ => false,
        }
    };
    let dislay_dice = |pid: usize| -> Option<Vec<u32>> {
        match app.get_state().get_play_data() {
            Ok(p) if is_playing(pid) => Some(HandOpError::unwrap_pips(p.get_hand().get_pips())),
            _ => None,
        }
    };

    let mut score_rows: Vec<_> = enum_iterator::all::<Boxes>()
        .map(|b| {
            Row::new(
                vec![Cell::from(format!("{:>1$}", b, BOXES_CELL_WIDTH))]
                    .into_iter()
                    .chain(
                        (0..app.get_game_data().unwrap().get_num_players()).map(|pid| {
                            let st = &app.get_game_data().unwrap().get_score_table(pid);
                            let is_playing = is_playing(pid);
                            let dice = dislay_dice(pid);

                            let text = if st.has_score_in(b) {
                                format!("{:>1$}", st.get_score(b).unwrap(), SCORE_CELL_WIDTH)
                            } else if let Some(d) = dice {
                                format!("{:>1$}", scoring(b, &d), SCORE_CELL_WIDTH)
                            } else {
                                String::new()
                            };

                            let style = if is_playing && !st.has_score_in(b) {
                                let pos = app.get_state().get_play_cursor_pos().unwrap();
                                let mut style = Style::default().fg(Color::Rgb(255, 215, 0));
                                if pos == &PlayCursorPos::Table(b) {
                                    style = style.fg(Color::Black).bg(Color::Rgb(255, 215, 0));
                                }
                                style
                            } else {
                                Style::default()
                            };

                            Cell::from(text).style(style)
                        }),
                    ),
            )
        })
        .collect();
    let bonus_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Bonus", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain(
                (0..app.get_game_data().unwrap().get_num_players()).map(|pid| {
                    let st = &app.get_game_data().unwrap().get_score_table(pid);
                    let is_playing = is_playing(pid);
                    let dice = dislay_dice(pid);

                    let default_bstext = format!("{:>2}", "");
                    let default_bsstyle = Style::default();
                    let (bstext, bsstyle) = if let Some(score) = st.calculate_bonus() {
                        (format!("{:>2}", score), default_bsstyle)
                    } else if let (true, Some(d)) = (is_playing, &dice) {
                        let pos = app.get_state().get_play_cursor_pos().unwrap();
                        let ifbs = if let &PlayCursorPos::Table(b) = pos {
                            st.calculate_bonus_if_filled_by(b, scoring(b, d))
                        } else {
                            None
                        };

                        if let Some(bs) = ifbs {
                            (
                                format!("{:>2}", bs),
                                default_bsstyle.fg(Color::Rgb(255, 215, 0)),
                            )
                        } else {
                            (default_bstext, default_bsstyle)
                        }
                    } else {
                        (default_bstext, default_bsstyle)
                    };

                    let us = st.get_total_upper_score();
                    let default_ustext = format!("{:>3}", us);
                    let default_usstyle = Style::default();
                    let (ustext, usstyle) = if let (true, Some(d)) = (is_playing, &dice) {
                        let pos = app.get_state().get_play_cursor_pos().unwrap();
                        let ifus = if let &PlayCursorPos::Table(b) = pos {
                            st.get_total_upper_score_if_filled_by(b, scoring(b, d))
                        } else {
                            us
                        };

                        if ifus > us {
                            (
                                format!("{:>3}", ifus),
                                default_usstyle.fg(Color::Rgb(255, 215, 0)),
                            )
                        } else {
                            (default_ustext, default_usstyle)
                        }
                    } else {
                        (default_ustext, default_usstyle)
                    };

                    Cell::from(Line::from(vec![
                        Span::styled(bstext, bsstyle),
                        Span::raw(" ("),
                        Span::styled(ustext, usstyle),
                        Span::raw(format!("/{:>2})", ScoreTable::BONUS_THRESHOLD)),
                    ]))
                }),
            ),
    );
    score_rows.push(bonus_cell);
    let total_cell = Row::new(
        vec![Cell::from(format!("{:>1$}", "Total", BOXES_CELL_WIDTH))]
            .into_iter()
            .chain(
                (0..app.get_game_data().unwrap().get_num_players()).map(|pid| {
                    let st = &app.get_game_data().unwrap().get_score_table(pid);
                    let is_playing = is_playing(pid);
                    let dice = dislay_dice(pid);
                    let total_score = st.get_total_score();

                    let default_text = format!("{:>1$}", total_score, SCORE_CELL_WIDTH);
                    let default_style = Style::default();
                    let (text, style) = if let (true, Some(d)) = (is_playing, &dice) {
                        let pos = app.get_state().get_play_cursor_pos().unwrap();
                        let if_total_score = if let &PlayCursorPos::Table(b) = pos {
                            st.get_total_score_if_filled_by(b, scoring(b, d))
                        } else {
                            total_score
                        };

                        if if_total_score > total_score {
                            (
                                format!("{:>1$}", if_total_score, SCORE_CELL_WIDTH),
                                default_style.fg(Color::Rgb(255, 215, 0)),
                            )
                        } else {
                            (default_text, default_style)
                        }
                    } else {
                        (default_text, default_style)
                    };

                    Cell::from(text).style(style)
                }),
            ),
    );
    score_rows.push(total_cell);
    let score_header = Row::new(vec![Cell::from(String::from(""))].into_iter().chain(
        (0..app.get_game_data().unwrap().get_num_players()).map(|pid| {
            let text = format!("{:^1$}", format!("Player{}", pid), SCORE_CELL_WIDTH);
            let style = if is_playing(pid) {
                Style::default().fg(Color::Black).bg(Color::LightYellow)
            } else {
                Style::default()
            };
            Cell::from(text).style(style)
        }),
    ));
    let score_table_width: Vec<_> = (0..(app.get_game_data().unwrap().get_num_players() + 1))
        .map(|x| {
            Constraint::Length(if x == 0 {
                BOXES_CELL_WIDTH as u16
            } else {
                SCORE_CELL_WIDTH as u16
            })
        })
        .collect();
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
    draw_score_table(f, app, chunks[1]);
}

fn draw_result(f: &mut Frame, app: &App, chunk: Rect) {
    match app.get_state() {
        AppState::Result => (),
        _ => panic!(),
    }

    let block = Block::default().borders(Borders::ALL);
    f.render_widget(block, chunk);

    let height = (app.get_game_data().unwrap().get_num_players() as u16) + 2;
    let width = chunk.width - 4;

    let text_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(create_centerd_rect(chunk, width + 2, height + 2));

    let mut results: Vec<_> = (0..app.get_game_data().unwrap().get_num_players())
        .map(|i| {
            (
                i,
                app.get_game_data()
                    .unwrap()
                    .get_score_table(i)
                    .get_total_score(),
            )
        })
        .collect();
    results.sort_by(|(.., left), (.., right)| left.cmp(right).reverse());
    let mut results: Vec<_> = results
        .iter()
        .map(|(pid, score)| {
            let rank = results.iter().position(|(.., s)| s == score).unwrap() + 1;
            Line::from(Span::styled(
                format!("{:^1$}", format!("{}. Player{}", rank, pid), width as usize),
                Style::default(),
            ))
        })
        .collect();
    results.extend([
        Line::from(Span::raw("")),
        Line::from(Span::styled(
            format!("{:^1$}", "Press ENTER to exit.", width as usize),
            Style::default(),
        )),
    ]);
    let text = Paragraph::new(results)
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(text, text_chunk[0]);
}

fn create_centerd_rect(base_rect: Rect, width: u16, height: u16) -> Rect {
    Rect::new(
        base_rect.x + (base_rect.width - width) / 2,
        base_rect.y + (base_rect.height - height) / 2,
        width,
        height,
    )
}
