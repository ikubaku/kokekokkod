use chrono::prelude::*;
use crate::context::ContextState::ChangeRead;
use chrono::Duration;

const DURATION_THRESHOLD_MIN: i64 = 10;

#[derive(PartialEq)]
pub enum StatusKind {
    NoData,
    Awake,
    Sleeping,
}

#[derive(PartialEq)]
enum ContextState {
    ChangeNotDetected,
    ChangeDetected,
    ChangeRead,
}

pub struct Context {
    last_status_change_datetime: Option<DateTime<Utc>>,
    last_update_datetime: Option<DateTime<Utc>>,
    status_kind: StatusKind,
    state: ContextState,
}

pub enum EventKind {
    WakeUp,
    StartSleeping,
}

pub struct Event {
    pub kind: EventKind,
    pub datetime: DateTime<Utc>,
}

impl Context {
    pub fn default() -> Self {
        Context {
            last_status_change_datetime: None,
            last_update_datetime: None,
            status_kind: StatusKind::NoData,
            state: ContextState::ChangeNotDetected,
        }
    }

    pub fn update_status(&mut self, datetime: DateTime<Utc>, status_kind: StatusKind) {
        if status_kind != self.status_kind {
            self.status_kind = status_kind;
            self.last_status_change_datetime = Some(datetime);
            self.last_update_datetime = Some(datetime);
            self.state = ContextState::ChangeNotDetected;
        } else {
            self.last_update_datetime = Some(datetime);
        }

        if self.state != ChangeRead {
            if let Some(d) = self.get_streak() {
                if d >= Duration::minutes(DURATION_THRESHOLD_MIN) {
                    self.state = ContextState::ChangeDetected;
                }
            }
        }
    }

    pub fn read_change(&mut self) -> Option<Event> {
        if self.state == ContextState::ChangeDetected {
            self.state = ContextState::ChangeRead;
            let kind = match self.status_kind {
                StatusKind::NoData => panic!("BUG: Invalid context state: ChangeDetected but status_kind is NoData"),
                StatusKind::Awake => EventKind::WakeUp,
                StatusKind::Sleeping => EventKind::StartSleeping,
            };
            Some(Event {
                kind,
                datetime: self.last_status_change_datetime
                    .expect("BUG: Invalid context state: ChangeDetected but no status change datetime")
                    .clone(),
            })
        } else {
            None
        }
    }

    fn get_streak(&self) -> Option<Duration> {
        let start = self.last_status_change_datetime?;
        let end = self.last_update_datetime?;

        Some(end - start)
    }
}
