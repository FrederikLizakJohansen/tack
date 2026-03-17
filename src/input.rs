use std::time::Duration;

use crossterm::event::{self, Event, KeyEvent};

pub fn next_event(timeout: Duration) -> std::io::Result<Option<Event>> {
    if event::poll(timeout)? {
        event::read().map(Some)
    } else {
        Ok(None)
    }
}

pub fn as_key(event: Event) -> Option<KeyEvent> {
    match event {
        Event::Key(key) => Some(key),
        _ => None,
    }
}

