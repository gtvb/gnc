use std::error;
use crate::nostr_client::response::Response;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

const MAX_ITEMS_ON_SCREEN: usize = 5;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Messages received from the relay
    pub feed: Vec<Response>,

    pub feed_capacity: usize,
    pub current_min_index: usize,
    pub current_max_index: usize,

    /// This is the place where the user may type in some data
    /// to send
    pub input_box: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            feed: Vec::new(),
            feed_capacity: 20,
            current_min_index: 0,
            current_max_index: 0,
            input_box: None,
        }
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn add_events(&mut self, events: &mut Vec<Response>) {
        // If we get more items than what we can store
        if self.feed.len() + events.len() > self.feed_capacity {
            // Remove the first `events.len()` items from the feed
            // to make room for the new ones
            self.feed.drain(..events.len());
        }

        // Otherwise, we can just append
        self.feed.append(events);
        self.feed_capacity += events.len();

        self.current_min_index = self.current_max_index;
        self.current_max_index += events.len() - 1;
    }

    pub fn scroll_up(&mut self) {
        self.current_min_index.checked_sub(MAX_ITEMS_ON_SCREEN).unwrap_or(0);
        self.current_max_index = self.current_min_index + MAX_ITEMS_ON_SCREEN;
    }

    pub fn scroll_down(&mut self) {
        let tmp = self.current_max_index;
        self.current_max_index.checked_add(MAX_ITEMS_ON_SCREEN).unwrap_or(self.feed.len() - 1);
        self.current_min_index = self.current_min_index + tmp;
    }
}
