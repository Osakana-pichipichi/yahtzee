use crate::app::{App, CursorPos};
use crate::scoring::{box_name, Boxes};
use enum_iterator::all;
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
    let play_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(20),
            Constraint::Percentage(50),
            Constraint::Percentage(30),
        ])
        .split(chunks[0]);

    /* Role block */
    let role_block = Block::default().title("Role").borders(Borders::ALL);
    f.render_widget(role_block, play_chunks[0]);

    let role_button_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(100)])
        .split(Rect {
            x: play_chunks[0].x + (play_chunks[0].width - 13) / 2,
            y: play_chunks[0].y + (play_chunks[0].height - 3) / 2,
            width: 13,
            height: 3,
        });
    let text = Paragraph::new(Spans::from(Span::styled("Role!", Style::default())))
        .block(Block::default().borders(Borders::ALL))
        .style(match app.cursor_pos {
            CursorPos::Role => Style::default().fg(Color::DarkGray).bg(Color::White),
            _ => Style::default(),
        })
        .alignment(Alignment::Center);
    f.render_widget(text, role_button_chunk[0]);

    /* Dice Block */
    let current_dice_block = Block::default().title("Dice").borders(Borders::ALL);
    f.render_widget(current_dice_block, play_chunks[1]);

    let dice_chunk = Layout::default()
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
        .split(Rect {
            x: play_chunks[1].x + (play_chunks[1].width - 39) / 2,
            y: play_chunks[1].y + (play_chunks[1].height - 5) / 2,
            width: 39,
            height: 5,
        });
    for i in 0..5 {
        let text = if app.current_play.is_held[i] {
            vec![
                Spans::from(Span::styled(DICE_STR[i][0], Style::default())),
                Spans::from(Span::styled(DICE_STR[i][1], Style::default())),
                Spans::from(Span::styled(DICE_STR[i][2], Style::default())),
            ]
        } else {
            vec![]
        };
        let text = Paragraph::new(text)
            .block(if app.current_play.is_held[i] {
                Block::default().borders(Borders::ALL)
            } else {
                Block::default()
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
        f.render_widget(text, dice_chunk[2 * i]);
    }

    /* Dust Block */
    let dust_block = Block::default().title("Dust").borders(Borders::ALL);
    f.render_widget(dust_block, play_chunks[2]);

    let dice_chunk = Layout::default()
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
        .split(Rect {
            x: play_chunks[2].x + (play_chunks[2].width - 39) / 2,
            y: play_chunks[2].y + (play_chunks[2].height - 5) / 2,
            width: 39,
            height: 5,
        });
    for i in 0..5 {
        let text = if !app.current_play.is_held[i] {
            vec![
                Spans::from(Span::styled(DICE_STR[i][0], Style::default())),
                Spans::from(Span::styled(DICE_STR[i][1], Style::default())),
                Spans::from(Span::styled(DICE_STR[i][2], Style::default())),
            ]
        } else {
            vec![]
        };
        let text = Paragraph::new(text)
            .block(if !app.current_play.is_held[i] {
                Block::default().borders(Borders::ALL)
            } else {
                Block::default()
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
        f.render_widget(text, dice_chunk[2 * i]);
    }

    /* Table block */
    let score_rows = all::<Boxes>()
        .enumerate()
        .map(|(bid, b)| {
            Row::new(
                vec![Cell::from(box_name(&b).to_string())]
                    .into_iter()
                    .chain(
                        (0..app.num_players)
                            .map(|pid| {
                                let st = &app.scores[pid];
                                let text = if st.is_filled(&b) || pid == app.current_play.player_id
                                {
                                    format!("{}", st.get_score(&b))
                                } else {
                                    format!("")
                                };
                                let cell = Cell::from(text);
                                match app.cursor_pos {
                                    CursorPos::Table(pos) => {
                                        if bid == pos && pid == app.current_play.player_id {
                                            cell.style(
                                                Style::default().fg(Color::Yellow).bg(Color::White),
                                            )
                                        } else {
                                            cell.style(Style::default().fg(Color::Yellow))
                                        }
                                    }

                                    _ => cell.style(Style::default().fg(Color::Yellow)),
                                }
                            })
                            .collect::<Vec<Cell>>()
                            .into_iter(),
                    ),
            )
        })
        .collect::<Vec<Row>>();
    let score_header = Row::new(
        vec![String::from("")].into_iter().chain(
            (0..app.num_players)
                .map(|x| format!("Player{}", x))
                .collect::<Vec<String>>()
                .into_iter(),
        ),
    );
    let score_table_width = (0..(app.num_players + 1))
        .map(|x| Constraint::Length(if x == 0 { 20 } else { 7 }))
        .collect::<Vec<Constraint>>();
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

    f.render_widget(score_block, chunks[1]);
}
