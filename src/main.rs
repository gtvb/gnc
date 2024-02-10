mod nostr_client;
mod tui;

// use nostr_client::{connection::Connection, event::Event as NostrEvent};
// use secp256k1::Secp256k1;

use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io;
use tui::app::{App, AppResult};
use tui::event::{Event, EventHandler};
use tui::handler::handle_key_events;
use tui::tui::Tui;

// const SECRET_KEY: &'static str = "";
// const PUB_KEY: &'static str = "";

#[tokio::main]
async fn main() -> AppResult<()> {
    // Create an application.
    let mut app = App::new();

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
