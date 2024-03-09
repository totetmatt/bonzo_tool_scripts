mod bonzomatic;
use clap::Parser;
use core::sync::atomic::Ordering;
use crossterm::{
    event::{self, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use futures_util::StreamExt;
use log::debug;
use ratatui::prelude::Color;
use ratatui::prelude::Constraint;
use ratatui::prelude::Direction;
use ratatui::prelude::Layout;
use ratatui::prelude::Line;
use ratatui::prelude::Span;
use ratatui::prelude::Text;
use ratatui::prelude::{CrosstermBackend, Stylize, Terminal};
use ratatui::style::Style;
use ratatui::widgets::Block;
use ratatui::widgets::BorderType;
use ratatui::widgets::Borders;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Row;
use ratatui::widgets::Table;
use ratatui::widgets::TableState;
use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{stdout, Result};
use std::sync::Mutex;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::time::Duration;
use tokio::time::Instant;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
#[derive(Parser)]
#[clap(author, version, about)]
struct Activity {
    /// url for the room (ws://host.com:port/room/)
    url: String,
}
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Clone)]
enum Status {
    OFFLINE,
    ONLINE,
}
#[derive(Debug, PartialEq, PartialOrd, Clone)]
struct Participant {
    handle: String,
    timestamp: Instant,
    code: String,
    first_line: u32,
    anchor: u32,
    caret: u32,
    compile: bool,
    shader_time: f64,
}
impl Participant {
    fn get_duration_since_last(&self) -> Duration {
        let now = tokio::time::Instant::now();
        now.duration_since(self.timestamp)
    }
    fn is_offline(&self) -> bool {
        self.get_duration_since_last() > Duration::from_secs(1)
    }
}

async fn read_ws(
    participants: Arc<Mutex<HashMap<String, RefCell<Participant>>>>,
    close: Arc<AtomicBool>,
    connect_addr: &String,
) -> Result<()> {
    let (ws_stream, _) = connect_async(connect_addr)
        .await
        .expect("Failed to connect");

    //println!("WebSocket handshake has been successfully completed");
    let (_, mut read) = ws_stream.split();
    while let Some(message) = read.next().await {
        if close.load(Ordering::Relaxed) {
            break;
        }
        match message {
            Ok(data) => match data {
                Message::Ping(_) => debug!("Ping!"),
                Message::Text(_) => {
                    let payload: bonzomatic::Payload = bonzomatic::Payload::from_message(&data);

                    let p = &mut participants.lock().unwrap();

                    if p.contains_key(payload.get_nickname()) {
                        p.get(payload.get_nickname()).unwrap().replace(Participant {
                            handle: payload.get_nickname().to_string(),
                            timestamp: tokio::time::Instant::now(),
                            code: payload.get_code().to_string(),
                            first_line: payload.get_visible_line(),
                            anchor: payload.get_anchor(),
                            caret: payload.get_caret(),
                            compile: payload.get_compile(),
                            shader_time: payload.get_shader_time(),
                        });
                    } else {
                        p.insert(
                            payload.get_nickname().to_string(),
                            RefCell::new(Participant {
                                handle: payload.get_nickname().to_string(),
                                timestamp: tokio::time::Instant::now(),
                                code: payload.get_code().to_string(),
                                first_line: payload.get_visible_line(),
                                anchor: payload.get_anchor(),
                                caret: payload.get_caret(),
                                compile: payload.get_compile(),
                                shader_time: payload.get_shader_time(),
                            }),
                        );
                    }
                }
                _ => (),
            },
            _ => (),
        }
    }
    Ok(())
}

async fn interface(
    participants: Arc<Mutex<HashMap<String, RefCell<Participant>>>>,
    close: Arc<AtomicBool>,
) -> Result<()> {
    let mut selected_handle: Option<String> = None;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    loop {
        tokio::time::sleep(Duration::from_millis(16)).await;
        let participants_mutex = participants.lock().unwrap();
        let mut participants_list = participants_mutex
            .values()
            .map(|p| p.clone())
            .collect::<Vec<RefCell<Participant>>>();
        participants_list
            .sort_by_key(|a| (a.borrow().is_offline(), a.borrow().handle.to_lowercase().to_string()));
        let rows = participants_list
            .iter()
            .map(|p| {
                let p = p.borrow();

                if p.is_offline() {
                    Row::new(vec![format!("{:?}", Status::OFFLINE), p.handle.clone()])
                        .style(Style::new().red())
                } else {
                    Row::new(vec![
                        format!("{:?}", Status::ONLINE),
                        p.handle.clone(),
                        format!("{}", p.anchor),
                        format!("{}", p.caret),
                        format!("{}", p.shader_time),
                        if p.compile {
                            "🆒".to_owned()
                        } else {
                            "".to_owned()
                        },
                    ])
                    .style(Style::new().green())
                }
            })
            .collect::<Vec<Row>>();

        let selected_index = match selected_handle {
            Some(ref handle) => participants_list
                .iter()
                .position(|a| a.borrow().handle.eq(handle)),
            None => None,
        };
 
        let (code,line)= match selected_handle {
            Some(ref handle) => {
                let p = participants_mutex.get(handle).unwrap().borrow();
                let anchor = p.anchor as usize;
                let caret = p.caret as usize;
                let (first, last, diff) = if anchor > caret {
                    (caret, anchor, anchor - caret)
                } else {
                    (anchor,caret,  caret - anchor)
                };
                let mut final_string_vec: Vec<Line> = Vec::new();
                let mut total: usize = 0;
                let mut is_view = false;
                for (size, line) in p
                    .code
                    .split_terminator("\n")
                    .map(|line| (line.bytes().len(), line))
                {
                    let start_line = total;
                    let end_line = start_line + line.bytes().len();
                    let mut v: Vec<Span> = Vec::new();
                    if start_line <= first && first <= end_line {
                        let (left, right) = line.split_at(first - start_line);
                        v.push(Span::styled(left.to_string(), Style::default()));
                        v.push(Span::styled("▙".to_string(), Style::default().fg(Color::Red)));
                        v.push(Span::styled(right.to_string(), Style::default().fg(Color::Yellow)));
                        is_view=true;
                    } else {
                     
                        v.push(Span::styled(line.to_string(), if is_view {Style::default().fg(Color::Yellow)} else {Style::default()}));
                    };
                    if (start_line <= last && last <= end_line ) {
                        let split_idx = if v.len() != 1 {
                            diff
                        } else {
                            last - start_line
                        };
                        let tmp = v.pop().unwrap();
                        let (left, right) = tmp.content.split_at(split_idx);
                        v.push(Span::styled(left.to_string(), Style::default().fg(Color::Yellow)));
                        v.push(Span::styled("▜".to_string(), Style::default().fg(Color::Red)));
                        v.push(Span::styled(right.to_string(), Style::default()));
                        is_view=false;
                    }
                    final_string_vec.push(Line::from(v));
                    total += size + 1;
                }
             
                (final_string_vec,p.first_line)
            }
            None =>( Vec::new(),0),
        };
    
        let text = Text::from(code);
        terminal.draw(|frame| {
            let main_layout = Layout::new(
                Direction::Horizontal,
                [Constraint::Percentage(33), Constraint::Percentage(66)],
            )
            .split(frame.size());

            // Columns widths are constrained in the same way as Layout...
            let widths = [
                Constraint::Max(10),
                Constraint::Max(20),
                Constraint::Max(4),
                Constraint::Max(4),
                Constraint::Max(8),
                Constraint::Max(8),
            ];
            let mut table_state = TableState::default();
            let table = Table::new(rows, widths)
                .column_spacing(1)
                .style(Style::new().blue())
                .header(
                    Row::new(vec!["STATUS", "Handle"])
                        .style(Style::new().bold())
                        .bottom_margin(1),
                )
                //.footer(Row::new(vec!["", "Updated on Dec 28"]))
                .block(Block::default().title("Room Overview"))
                .highlight_style(Style::new().reversed())
                .highlight_symbol("► ");
            table_state = table_state.with_selected(selected_index);
            frame.render_stateful_widget(table, main_layout[0], &mut table_state);

            let block = Block::default()
                .title("Code")
                .title_alignment(ratatui::layout::Alignment::Center)
                .borders(Borders::LEFT | Borders::RIGHT | Borders::TOP | Borders::BOTTOM)
                .border_style(Style::default().fg(Color::White))
                .border_type(BorderType::Rounded)
                .style(Style::default().bg(Color::Black));

            let p = Paragraph::new(text)
                .block(block)
                .style(Style::new().white().on_black())       .scroll((line as u16, 0));;
     
            frame.render_widget(p, main_layout[1]);
        })?;

        if event::poll(std::time::Duration::from_millis(16))? {
            if let event::Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Release && key.code == KeyCode::Char('q') {
                    break;
                }

                if key.kind == KeyEventKind::Release && key.code == KeyCode::Up {
                    selected_handle = match &selected_handle {
                        None => participants_list.last().map(|p| p.borrow().handle.clone()),
                        Some(handle) => {
                            let pos = participants_list
                                .iter()
                                .position(|a| a.borrow().handle.eq(handle))
                                .unwrap();
                            if pos == 0 {
                                None
                            } else {
                                participants_list
                                    .get(pos - 1)
                                    .map(|p| p.borrow().handle.clone())
                            }
                        }
                    };
                }
                if key.kind == KeyEventKind::Release && key.code == KeyCode::Down {
                    selected_handle = match &selected_handle {
                        None => participants_list.first().map(|p| p.borrow().handle.clone()),

                        Some(handle) => {
                            let pos = participants_list
                                .iter()
                                .position(|a| a.borrow().handle.eq(handle))
                                .unwrap();
                            let next_pos = pos + 1;
                            if next_pos >= participants_list.len() {
                                None
                            } else {
                                participants_list
                                    .get(next_pos)
                                    .map(|p| p.borrow().handle.clone())
                            }
                        }
                    };
                }
            }
        }
    }
    terminal.clear()?;
    close.store(true, Ordering::Relaxed);
    Ok(())
}
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Activity::parse();
    let close = Arc::new(AtomicBool::new(false));
    let participants: Arc<Mutex<HashMap<String, RefCell<Participant>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _ = tokio::join!(
        read_ws(participants.clone(), close.clone(), &cli.url),
        interface(participants.clone(), close.clone())
    );

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}
