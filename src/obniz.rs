use chrono::prelude::*;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ObnizResponse {
    weight: f64,
    datetime: f64,
    status: String,
}

pub enum Status {
    Ok,
    Ng,
    NoData,
}

impl ObnizResponse {
    pub fn status(&self) -> Status {
        if self.status == "ok" {
            Status::Ok
        } else if self.status == "ng" {
            Status::Ng
        } else if self.status == "noData" {
            Status::NoData
        } else {
            panic!("BUG: Unknown status value");
        }
    }

    pub fn get_datetime(&self) -> DateTime<Utc> {
        let unix_time = NaiveDateTime::from_timestamp((self.datetime / 1000.0) as i64, 0);
        DateTime::from_utc(unix_time, Utc)
    }

    pub fn is_heavier_than(&self, threshold: f64) -> bool {
        self.weight > threshold
    }
}
