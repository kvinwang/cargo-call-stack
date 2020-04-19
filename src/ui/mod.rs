use event::{Event, Events};
use std::io;
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    style::{Color, Style},
    widgets::{Block, Borders, List, Text},
    Terminal,
};
use petgraph::graph::DiGraph;
use crate::Node;
use petgraph::stable_graph::NodeIndex;

struct App<'a> {
    history: Vec<NodeIndex>,
    graph: DiGraph::<Node<'a>, ()>,
    items: Vec<(NodeIndex, u64, String)>,
    info_style: Style,
    active_style: Style,
    cur: usize,
    node: NodeIndex,
}

impl<'a> App<'a> {
    fn new(start: NodeIndex, graph: DiGraph::<Node<'a>, ()>) -> Self {
        let mut app = App {
            history: vec![],
            graph,
            items: vec![],
            info_style: Style::default().fg(Color::Gray),
            active_style: Style::default().fg(Color::Red),
            cur: 0,
            node: start,
        };
        app.load_items(start, false);
        app
    }
}

impl<'a> App<'a> {
    fn load_items(&mut self, node_ind: NodeIndex, push_history: bool) {
        fn node_desc(graph: &DiGraph::<Node, ()>, index: NodeIndex) -> String {
            graph.node_weight(index).map(|node| {
                format!("max {:10} local = {:<5} {}", node.max.map_or("?".to_owned(), |m|m.to_string()), node.local.to_string(), node.name)
            }).unwrap_or("node not found!".to_owned())
        }
        if self.graph.node_weight(node_ind).is_some() {
            if push_history {
                self.history.push(self.node);
            }
            self.node = node_ind;
            self.items.clear();
            self.items.push((node_ind, 0, format!("{} {}", node_ind.index(), node_desc(&self.graph, node_ind))));
            let mut callees = self.graph.neighbors(self.node).detach();

            while let Some((_, callee)) = callees.next(&self.graph) {
                let label = node_desc(&self.graph, callee);
                let max = self.graph.node_weight(callee).map_or(0, |m| {
                    m.max.map_or(0, |m| {
                        m.max_value()
                    })
                });
                self.items.push((callee, max, format!(" {:<5} {}", callee.index(), label)));
            }
            if self.items.len() > 3 {
                self.items[1..].sort_by(|a, b| b.1.cmp(&a.1))
            }
            self.cur = 0;
        }
    }

    fn back(&mut self) {
        if let Some(ind) = self.history.pop() {
            self.load_items(ind, false);
        }
    }
}

pub(crate) fn run(g: DiGraph::<Node, ()>) -> Result<(), failure::Error> {
    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    let mut events = Events::new();
    events.disable_exit_key();

    // App
    let mut app = App::new(NodeIndex::new(0), g);

    loop {
        terminal.draw(|mut f| {
            let items = app.items.iter()
                .enumerate()
                .map(|(ind, &(_node, _max, ref label))| {
                    Text::styled(
                        label.clone(),
                        if app.cur == ind { app.active_style } else { app.info_style }
                    )
                });
            let events_list = List::new(items)
                .block(Block::default()
                    .borders(Borders::ALL).title("List"));
            f.render_widget(events_list, f.size());
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Char('q') => {
                    break;
                }
                Key::Char('j') => {
                    app.cur += 1;
                    if app.cur >= app.items.len() {
                        app.cur = 0;
                    }
                }
                Key::Char('k') => {
                    if app.cur == 0 {
                        app.cur = app.items.len() - 1;
                    } else {
                        app.cur -= 1;
                    }
                }
                Key::Char('l') => {
                    if app.cur != 0 {
                        let ind = app.items[app.cur].0;
                        app.load_items(ind, true);
                    }
                }
                Key::Char('h') => {
                    app.back();
                }
                _ => {}
            },
        }
    }
    Ok(())
}

mod event;
