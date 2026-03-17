use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct Flash {
    started_at: Instant,
    duration: Duration,
}

impl Flash {
    pub fn new(duration: Duration) -> Self {
        Self {
            started_at: Instant::now(),
            duration,
        }
    }

    pub fn is_active(&self) -> bool {
        self.started_at.elapsed() < self.duration
    }
}

#[derive(Debug, Clone)]
pub struct Shake {
    started_at: Instant,
    duration: Duration,
}

impl Shake {
    pub fn new(duration: Duration) -> Self {
        Self {
            started_at: Instant::now(),
            duration,
        }
    }

    pub fn offset(&self) -> i16 {
        if !self.is_active() {
            return 0;
        }

        let elapsed_ms = self.started_at.elapsed().as_millis() as i16;
        match (elapsed_ms / 35) % 4 {
            0 => -1,
            1 => 1,
            2 => 0,
            _ => 1,
        }
    }

    pub fn is_active(&self) -> bool {
        self.started_at.elapsed() < self.duration
    }
}

#[derive(Debug, Clone, Default)]
pub struct Animations {
    pub focus_flash: Option<Flash>,
    pub task_flash: Option<Flash>,
    pub status_flash: Option<Flash>,
    pub group_flash: Option<Flash>,
    pub shake: Option<Shake>,
    pub cursor_anchor: Option<Instant>,
    pub cursor_last_activity: Option<Instant>,
}

impl Animations {
    pub fn new() -> Self {
        Self {
            focus_flash: None,
            task_flash: None,
            status_flash: None,
            group_flash: None,
            shake: None,
            cursor_anchor: Some(Instant::now()),
            cursor_last_activity: Some(Instant::now()),
        }
    }

    pub fn tick(&mut self) {
        if self.focus_flash.as_ref().is_some_and(|a| !a.is_active()) {
            self.focus_flash = None;
        }
        if self.task_flash.as_ref().is_some_and(|a| !a.is_active()) {
            self.task_flash = None;
        }
        if self.status_flash.as_ref().is_some_and(|a| !a.is_active()) {
            self.status_flash = None;
        }
        if self.group_flash.as_ref().is_some_and(|a| !a.is_active()) {
            self.group_flash = None;
        }
        if self.shake.as_ref().is_some_and(|a| !a.is_active()) {
            self.shake = None;
        }
    }

    pub fn pulse_focus(&mut self) {
        self.focus_flash = Some(Flash::new(Duration::from_millis(220)));
    }

    pub fn pulse_task(&mut self) {
        self.task_flash = Some(Flash::new(Duration::from_millis(420)));
    }

    pub fn pulse_status(&mut self) {
        self.status_flash = Some(Flash::new(Duration::from_millis(700)));
    }

    pub fn pulse_group(&mut self) {
        self.group_flash = Some(Flash::new(Duration::from_millis(380)));
    }

    pub fn shake(&mut self) {
        self.shake = Some(Shake::new(Duration::from_millis(240)));
    }

    pub fn note_cursor_activity(&mut self) {
        let now = Instant::now();
        self.cursor_anchor = Some(now);
        self.cursor_last_activity = Some(now);
    }

    pub fn cursor_visible(&self) -> bool {
        let Some(last_activity) = self.cursor_last_activity else {
            return true;
        };
        if last_activity.elapsed() < Duration::from_millis(700) {
            return true;
        }
        self.cursor_anchor
            .map(|anchor| ((anchor.elapsed().as_millis() / 530) % 2) == 0)
            .unwrap_or(true)
    }
}
