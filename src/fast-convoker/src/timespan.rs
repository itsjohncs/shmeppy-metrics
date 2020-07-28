use chrono::{DateTime, Utc};

use spet::span::{SimpleSpan};


pub type TimeSpan = SimpleSpan<DateTime<Utc>>;
