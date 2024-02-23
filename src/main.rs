use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log2::*;
use ratatui::{
    backend::{Backend, CrosstermBackend},
    prelude::*,
    style::{Color, Modifier, Style},
    widgets::*,
    Terminal,
};
use serde_json::Value;
use std::{env, error::Error, fmt, hash::Hash, io, io::Read, process::exit};
use tui_tree_widget::{Tree, TreeItem, TreeState};

#[derive(Default, Clone, PartialEq, Eq, Hash, Debug)]
pub enum JsonPointer {
    ObjectKey(String),
    ArrayIdx(usize),
    #[default]
    None,
}

impl ToString for JsonPointer {
    fn to_string(&self) -> String {
        match self {
            Self::ObjectKey(key) => key.clone(),
            Self::ArrayIdx(index) => index.to_string(),
            Self::None => String::new(),
        }
    }
}

// TODO: https://github.com/aweinstock314/rust-clipboard

struct Content {
    key: Vec<JsonPointer>,
    value: String,
}

impl fmt::Debug for Content {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("")
            .field(&self.key)
            .field(&self.value)
            .finish()
    }
}

struct App<'a> {
    state: TreeState<JsonPointer>,
    items: Vec<TreeItem<'a, JsonPointer>>,
    show_cmd_popup: bool,
}

impl<'a> App<'a> {
    fn new(items: Vec<TreeItem<'a, JsonPointer>>) -> Self {
        Self {
            state: TreeState::default(),
            items,
            show_cmd_popup: false,
        }
    }
}

pub fn root_tree_items<'a>(root: &Value) -> Vec<TreeItem<'_, JsonPointer>> {
    match root {
        Value::Object(object) => tree_items_obj(object),
        Value::Array(array) => tree_items_arr(array),
        _ => vec![TreeItem::new_leaf(JsonPointer::None, root.to_string())],
    }
}

fn tree_items(key: JsonPointer, value: &Value) -> TreeItem<JsonPointer> {
    match value {
        Value::Object(object) => {
            let text = key.to_string();
            TreeItem::new(key, text, tree_items_obj(object)).unwrap()
        }
        Value::Array(array) => {
            let text = key.to_string();
            TreeItem::new(key, text, tree_items_arr(array)).unwrap()
        }
        _ => {
            let text = format!("{}: {value}", key.to_string());
            TreeItem::new_leaf(key, text)
        }
    }
}

fn tree_items_obj<'a>(object: &serde_json::Map<String, Value>) -> Vec<TreeItem<'_, JsonPointer>> {
    object
        .iter()
        .map(|(key, value)| tree_items(JsonPointer::ObjectKey(key.clone()), value))
        .collect()
}

fn tree_items_arr<'a>(array: &[Value]) -> Vec<TreeItem<'_, JsonPointer>> {
    array
        .iter()
        .enumerate()
        .map(|(index, value)| tree_items(JsonPointer::ArrayIdx(index), value))
        .collect()
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| {
            let area = f.size();

            let items = Tree::new(app.items.clone())
                .expect("unique item identifiers")
                .block(Block::bordered().title(format!(
                    "{} {:?}",
                    env!("CARGO_PKG_DESCRIPTION"),
                    app.state
                )))
                .highlight_style(
                    Style::new()
                        .fg(Color::Black)
                        .bg(Color::Gray)
                        .add_modifier(Modifier::BOLD),
                );

            // let vertical =
            //     Layout::vertical([Constraint::Percentage(20), Constraint::Percentage(80)]);
            // let [instructions, _content] = vertical.areas(area);
            // let text = if app.show_cmd_popup {
            //     "Press c to close the Command popup"
            // } else {
            //     "Press c to show the Command popup"
            // };
            // let paragraph = Paragraph::new(text.blue())
            //     .centered()
            //     .wrap(Wrap { trim: true });
            // f.render_widget(paragraph, instructions);
            if app.show_cmd_popup {
                let block = Block::default()
                    .title("Available commands")
                    .borders(Borders::ALL);
                let area = centered_rect(60, 30, area);
                f.render_widget(Clear, area);
                f.render_widget(block, area);
            }
            f.render_stateful_widget(items, area, &mut app.state);
        })?;

        // // main: selected: [ObjectKey("ticket"), ObjectKey("state"), ObjectKey("list"), ArrayIdx(0), ObjectKey("customer_id")]
        // // TODO: https://doc.rust-lang.org/nightly/core/fmt/trait.Debug.html#examples-1
        // debug!("selected: {:?}", app.state.selected());

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Enter | KeyCode::Char(' ') => app.state.toggle_selected(),
                    KeyCode::Left => app.state.key_left(),
                    KeyCode::Right => app.state.key_right(),
                    KeyCode::Down => app.state.key_down(&app.items),
                    KeyCode::Up => app.state.key_up(&app.items),
                    KeyCode::Home => {
                        app.state.select_first(&app.items);
                    }
                    KeyCode::End => {
                        app.state.select_last(&app.items);
                    }
                    KeyCode::PageDown => app.state.scroll_down(3),
                    KeyCode::PageUp => app.state.scroll_up(3),
                    KeyCode::Char('c') => app.show_cmd_popup = !app.show_cmd_popup,
                    // KeyCode::F(1) => {
                    //     let t = Content {
                    //         key: app.state.selected(),
                    //         value: "".to_string(),
                    //     };
                    //     debug!("selected: {:?}", t);
                    // }
                    _ => {}
                },
                Event::Mouse(mouse) => match mouse.kind {
                    event::MouseEventKind::ScrollDown => app.state.scroll_down(1),
                    event::MouseEventKind::ScrollUp => app.state.scroll_up(1),
                    _ => {}
                },
                _ => {}
            }
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(r);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}

fn main() -> Result<(), Box<dyn Error>> {
    // let _log2 = log2::open(&format!("{}.log", env!("CARGO_PKG_NAME"))).start();

    let mut stdin = io::stdin();
    let mut buff = String::new();
    stdin.read_to_string(&mut buff)?;
    let json_input: Value = serde_json::from_str(&buff)?;
    // debug!("json_input: {json_input:?}");

    let items = root_tree_items(&json_input);
    // debug!("items: {items:?}");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let app = App::new(items);
    let res = run_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
        exit(1);
    }
    Ok(())
}
