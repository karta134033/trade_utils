use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Timer {
    datetime: NaiveDateTime,
    fixed_update: FixedUpdate,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub enum FixedUpdate {
    Minute(i64),
    Hour(i64),
    Day(i64),
}

impl Timer {
    pub fn new(fixed_update: FixedUpdate) -> Self {
        let now = Utc::now();
        let datetime = NaiveDateTime::from_timestamp_millis(now.timestamp_millis()).unwrap();
        let mut timer = Timer {
            datetime,
            fixed_update: fixed_update.clone(),
        };
        match fixed_update {
            // Pad the datetime to 0 based
            FixedUpdate::Minute(_) => {
                let curr_ts = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                    NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap(),
                )
                .timestamp_millis();
                timer.datetime = NaiveDateTime::from_timestamp_millis(curr_ts).unwrap();
            }
            FixedUpdate::Hour(_) => {
                let curr_ts = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                    NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap(),
                )
                .timestamp_millis();
                timer.datetime = NaiveDateTime::from_timestamp_millis(curr_ts).unwrap();
            }
            FixedUpdate::Day(_) => {
                let curr_ts = NaiveDateTime::new(
                    NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                    NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                )
                .timestamp_millis();
                timer.datetime = NaiveDateTime::from_timestamp_millis(curr_ts).unwrap();
            }
        }
        info!("Start timer: {:?}", timer);
        timer
    }

    pub fn update(&mut self) -> bool {
        let now = Utc::now();
        let prev = self.datetime;
        let curr = now.naive_utc();
        let mut result = false;
        match self.fixed_update {
            FixedUpdate::Minute(minutes) => {
                let minute_duration = Duration::minutes(minutes);
                if prev.checked_add_signed(minute_duration).unwrap() <= curr {
                    self.datetime = NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                        NaiveTime::from_hms_opt(now.hour(), now.minute(), 0).unwrap(),
                    );
                    result = true;
                }
            }
            FixedUpdate::Hour(hours) => {
                let hour_duration = Duration::hours(hours);
                if prev.checked_add_signed(hour_duration).unwrap() <= curr {
                    self.datetime = NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                        NaiveTime::from_hms_opt(now.hour(), 0, 0).unwrap(),
                    );
                    result = true;
                }
            }
            FixedUpdate::Day(days) => {
                let day_duration = Duration::days(days);
                if prev.checked_add_signed(day_duration).unwrap() <= curr {
                    self.datetime = NaiveDateTime::new(
                        NaiveDate::from_ymd_opt(now.year(), now.month(), now.day()).unwrap(),
                        NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
                    );
                    result = true;
                }
            }
        }
        if result {
            info!("Update timer from {:?} to {:?}", prev, self.datetime);
        }
        result
    }

    pub fn get_ts_ms(&self) -> i64 {
        self.datetime.timestamp_millis()
    }
}
