use chrono::prelude::*;
use chrono::Duration;

const DURATION_THRESHOLD_MIN: i64 = 10;

#[derive(PartialEq, Clone)]
pub enum SensorStatusKind {
    NoData,
    Awake,
    Sleeping,
}

#[derive(PartialEq)]
enum ContextState {
    ChangeNotDetected,
    ChangeDetected,
}

pub struct Context {
    last_status_change_datetime: Option<DateTime<Utc>>,
    last_update_datetime: Option<DateTime<Utc>>,
    last_event_datetime: Option<DateTime<Utc>>,
    last_sensor_event_kind: SensorStatusKind,
    sensor_status_kind: SensorStatusKind,
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
            last_event_datetime: None,
            last_sensor_event_kind: SensorStatusKind::NoData,
            sensor_status_kind: SensorStatusKind::NoData,
            state: ContextState::ChangeNotDetected,
        }
    }

    pub fn update_status(&mut self, datetime: DateTime<Utc>, status_kind: SensorStatusKind) {
        if status_kind != self.sensor_status_kind {
            self.sensor_status_kind = status_kind;
            self.last_status_change_datetime = Some(datetime);
            self.last_update_datetime = Some(datetime);
        } else {
            self.last_update_datetime = Some(datetime);
        }

        if let Some(d) = self.get_duration() {
            if self.last_event_datetime != self.last_status_change_datetime
                && d >= Duration::minutes(DURATION_THRESHOLD_MIN) {
                self.commit_new_event();
            }
        }
    }

    pub fn read_change(&mut self) -> Option<Event> {
        if self.state == ContextState::ChangeDetected {
            self.state = ContextState::ChangeNotDetected;
            let kind = match self.last_sensor_event_kind {
                SensorStatusKind::NoData => panic!("BUG: Invalid context state: ChangeDetected but the sensor status is NoData"),
                SensorStatusKind::Awake => EventKind::WakeUp,
                SensorStatusKind::Sleeping => EventKind::StartSleeping,
            };
            Some(Event {
                kind,
                datetime: self.last_event_datetime
                    .expect("BUG: Invalid context state: ChangeDetected but no event datetime")
                    .clone(),
            })
        } else {
            None
        }
    }

    fn get_duration(&self) -> Option<Duration> {
        let start = self.last_status_change_datetime?;
        let end = self.last_update_datetime?;

        Some(end - start)
    }

    fn commit_new_event(&mut self) {
        self.last_event_datetime = Some(self.last_status_change_datetime.expect("BUG: Tried to commit new event without no last status change"));
        self.last_sensor_event_kind = self.sensor_status_kind.clone();
        self.state = ContextState::ChangeDetected;
    }
}
