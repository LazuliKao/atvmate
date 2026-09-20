use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_HISTORY_PER_DEVICE: usize = 100;
const SECONDS_PER_DAY: i64 = 86_400;

#[derive(Serialize, Deserialize, Object, Debug, Clone, PartialEq, Eq)]
pub struct OperationRecord {
    pub timestamp: String,
    pub operation: String,
    pub params: String,
    pub success: bool,
}

impl OperationRecord {
    pub fn new(operation: impl Into<String>, params: impl Into<String>, success: bool) -> Self {
        Self {
            timestamp: current_timestamp_iso8601(),
            operation: operation.into(),
            params: params.into(),
            success,
        }
    }
}

#[derive(Serialize, Deserialize, Object, Debug, Clone, PartialEq, Eq)]
pub struct HistoryResponse {
    pub operations: Vec<OperationRecord>,
}

#[derive(Serialize, Deserialize, Object, Debug, Clone, PartialEq, Eq)]
pub struct ReplayRequest {
    pub operations: Vec<OperationRecord>,
}

pub struct OperationHistory {
    records: Mutex<HashMap<String, VecDeque<OperationRecord>>>,
}

impl OperationHistory {
    pub fn new() -> Self {
        Self {
            records: Mutex::new(HashMap::new()),
        }
    }

    pub fn add_record(&self, device_id: &str, record: OperationRecord) {
        let mut records = self.records.lock().unwrap_or_else(|e| e.into_inner());
        let device_history = records.entry(device_id.to_string()).or_default();
        device_history.push_back(record);

        while device_history.len() > MAX_HISTORY_PER_DEVICE {
            device_history.pop_front();
        }
    }

    pub fn get_history(&self, device_id: &str) -> Vec<OperationRecord> {
        let records = self.records.lock().unwrap_or_else(|e| e.into_inner());
        records
            .get(device_id)
            .map(|history| history.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn clear_history(&self, device_id: &str) {
        let mut records = self.records.lock().unwrap_or_else(|e| e.into_inner());
        records.remove(device_id);
    }
}

impl Default for OperationHistory {
    fn default() -> Self {
        Self::new()
    }
}

pub fn current_timestamp_iso8601() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_seconds = duration.as_secs() as i64;
    let milliseconds = duration.subsec_millis();
    let days = total_seconds.div_euclid(SECONDS_PER_DAY);
    let seconds_of_day = total_seconds.rem_euclid(SECONDS_PER_DAY);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    let (year, month, day) = civil_from_days(days);

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{milliseconds:03}Z")
}

fn civil_from_days(days_since_epoch: i64) -> (i32, u32, u32) {
    let z = days_since_epoch + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = (yoe as i32) + (era as i32) * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };

    if month <= 2 {
        year += 1;
    }

    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    use super::{HistoryResponse, OperationHistory, OperationRecord, ReplayRequest};

    #[test]
    fn add_record_caps_history_at_one_hundred_entries() {
        let history = OperationHistory::new();

        for idx in 0..105 {
            history.add_record(
                "living-room",
                OperationRecord::new("key", format!(r#"{{"index":{idx}}}"#), idx % 2 == 0),
            );
        }

        let records = history.get_history("living-room");
        assert_eq!(records.len(), 100);
        assert_eq!(records.first().unwrap().params, r#"{"index":5}"#);
        assert_eq!(records.last().unwrap().params, r#"{"index":104}"#);
    }

    #[test]
    fn clear_history_removes_device_records() {
        let history = OperationHistory::new();
        history.add_record("bedroom", OperationRecord::new("key", "{}", true));

        history.clear_history("bedroom");

        assert!(history.get_history("bedroom").is_empty());
    }

    #[test]
    fn operation_record_timestamp_uses_iso8601_shape() {
        let record = OperationRecord::new("key", "{}", true);

        assert_eq!(record.timestamp.len(), 24);
        assert_eq!(&record.timestamp[4..5], "-");
        assert_eq!(&record.timestamp[7..8], "-");
        assert_eq!(&record.timestamp[10..11], "T");
        assert_eq!(&record.timestamp[13..14], ":");
        assert_eq!(&record.timestamp[16..17], ":");
        assert_eq!(&record.timestamp[19..20], ".");
        assert_eq!(&record.timestamp[23..24], "Z");
    }

    #[test]
    fn response_objects_wrap_operation_records() {
        let record = OperationRecord::new("combo", r#"{"keys":["home"]}"#, false);
        let history = HistoryResponse {
            operations: vec![record.clone()],
        };
        let replay = ReplayRequest {
            operations: vec![record.clone()],
        };

        assert_eq!(history.operations, vec![record.clone()]);
        assert_eq!(replay.operations, vec![record]);
    }
}
