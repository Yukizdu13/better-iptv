use crate::http::get_http_client;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use flate2::read::GzDecoder;
use log::{info, warn};
use quick_xml::events::Event;
use quick_xml::Reader;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpgProgram {
    pub channel_id: String,
    pub title: String,
    pub description: Option<String>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub category: Option<String>,
}

/// Fetch and parse XMLTV EPG data from a URL (async part)
pub async fn fetch_and_parse_epg(url: &str) -> Result<Vec<EpgProgram>> {
    info!("Fetching EPG from: {}", url);

    // Download EPG file using shared HTTP client
    let response = get_http_client()
        .get(url)
        .send()
        .await
        .context("Failed to download EPG file")?;

    let bytes = response.bytes().await.context("Failed to read EPG response")?;

    // Check if it's gzipped based on URL or magic bytes
    let xml_content = if url.ends_with(".gz") || is_gzipped(&bytes) {
        decompress_gzip(&bytes)?
    } else {
        String::from_utf8(bytes.to_vec()).context("Invalid UTF-8 in EPG file")?
    };

    // Parse XMLTV
    let programs = parse_xmltv(&xml_content)?;

    info!("Parsed {} EPG programs from XMLTV", programs.len());
    Ok(programs)
}

/// Store EPG programs in database (sync part)
pub fn store_epg_programs(conn: &Connection, programs: &[EpgProgram]) -> Result<usize> {
    let count = store_programs(conn, programs)?;
    info!("Stored {} EPG programs in database", count);
    Ok(count)
}

/// Check if bytes are gzipped (magic bytes: 1f 8b)
fn is_gzipped(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0] == 0x1f && bytes[1] == 0x8b
}

/// Decompress gzip data
fn decompress_gzip(bytes: &[u8]) -> Result<String> {
    let mut decoder = GzDecoder::new(bytes);
    let mut decompressed = String::new();
    decoder
        .read_to_string(&mut decompressed)
        .context("Failed to decompress gzipped EPG")?;
    Ok(decompressed)
}

/// Parse XMLTV format
fn parse_xmltv(xml: &str) -> Result<Vec<EpgProgram>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    // Pre-allocate with estimated capacity (typical EPG has 5k-50k programs)
    let mut programs = Vec::with_capacity(10_000);
    let mut buf = Vec::with_capacity(1024);

    let mut current_program: Option<EpgProgramBuilder> = None;
    let mut in_title = false;
    let mut in_desc = false;
    let mut in_category = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                match e.name().as_ref() {
                    b"programme" => {
                        let mut channel_id = String::new();
                        let mut start_time = None;
                        let mut end_time = None;

                        for attr in e.attributes().flatten() {
                            match attr.key.as_ref() {
                                b"channel" => {
                                    channel_id = String::from_utf8_lossy(&attr.value).to_string();
                                }
                                b"start" => {
                                    start_time = parse_xmltv_time(&attr.value);
                                }
                                b"stop" => {
                                    end_time = parse_xmltv_time(&attr.value);
                                }
                                _ => {}
                            }
                        }

                        if let (Some(start), Some(end)) = (start_time, end_time) {
                            current_program = Some(EpgProgramBuilder {
                                channel_id,
                                title: String::new(),
                                description: None,
                                start_time: start,
                                end_time: end,
                                category: None,
                            });
                        }
                    }
                    b"title" => in_title = true,
                    b"desc" => in_desc = true,
                    b"category" => in_category = true,
                    _ => {}
                }
            }
            Ok(Event::Text(e)) => {
                if let Some(ref mut prog) = current_program {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if in_title {
                        prog.title = text;
                    } else if in_desc {
                        prog.description = Some(text);
                    } else if in_category {
                        prog.category = Some(text);
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                match e.name().as_ref() {
                    b"programme" => {
                        if let Some(prog) = current_program.take() {
                            if !prog.title.is_empty() {
                                programs.push(prog.build());
                            }
                        }
                    }
                    b"title" => in_title = false,
                    b"desc" => in_desc = false,
                    b"category" => in_category = false,
                    _ => {}
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                warn!("XML parsing error: {}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(programs)
}

/// Parse XMLTV timestamp format (YYYYMMDDHHmmss +ZZZZ)
fn parse_xmltv_time(value: &[u8]) -> Option<DateTime<Utc>> {
    let time_str = String::from_utf8_lossy(value);

    // Format: "20231215193000 +0100"
    if time_str.len() < 14 {
        return None;
    }

    let year: i32 = time_str[0..4].parse().ok()?;
    let month: u32 = time_str[4..6].parse().ok()?;
    let day: u32 = time_str[6..8].parse().ok()?;
    let hour: u32 = time_str[8..10].parse().ok()?;
    let minute: u32 = time_str[10..12].parse().ok()?;
    let second: u32 = time_str[12..14].parse().ok()?;

    use chrono::{NaiveDate, NaiveDateTime, NaiveTime, TimeZone};

    let date = NaiveDate::from_ymd_opt(year, month, day)?;
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    let naive_dt = NaiveDateTime::new(date, time);

    // Parse timezone offset if present
    if time_str.len() >= 20 {
        let tz_part = time_str[15..20].trim();
        if let Ok(offset_minutes) = parse_tz_offset(tz_part) {
            use chrono::FixedOffset;
            if let Some(offset) = FixedOffset::east_opt(offset_minutes * 60) {
                if let Some(dt_with_tz) = offset.from_local_datetime(&naive_dt).single() {
                    return Some(dt_with_tz.with_timezone(&Utc));
                }
            }
        }
    }

    // Fallback: assume UTC
    Some(Utc.from_utc_datetime(&naive_dt))
}

/// Parse timezone offset (+0100, -0500, etc.)
fn parse_tz_offset(tz_str: &str) -> Result<i32, ()> {
    if tz_str.len() != 5 {
        return Err(());
    }

    let sign = if tz_str.starts_with('+') { 1 } else if tz_str.starts_with('-') { -1 } else { return Err(()); };

    let hours: i32 = tz_str[1..3].parse().map_err(|_| ())?;
    let minutes: i32 = tz_str[3..5].parse().map_err(|_| ())?;

    Ok(sign * (hours * 60 + minutes))
}

/// Store programs in database
fn store_programs(conn: &Connection, programs: &[EpgProgram]) -> Result<usize> {
    // Clear old programs (older than 24 hours ago)
    let cutoff = Utc::now() - chrono::Duration::hours(24);
    conn.execute(
        "DELETE FROM epg_programs WHERE end_time < ?1",
        rusqlite::params![cutoff.to_rfc3339()],
    )?;

    // Insert new programs
    let mut stmt = conn.prepare(
        "INSERT OR REPLACE INTO epg_programs
         (channel_epg_id, title, description, start_time, end_time, category)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )?;

    let mut count = 0;
    for program in programs {
        stmt.execute(rusqlite::params![
            program.channel_id,
            program.title,
            program.description,
            program.start_time.to_rfc3339(),
            program.end_time.to_rfc3339(),
            program.category,
        ])?;
        count += 1;
    }

    Ok(count)
}

/// Get current program for a channel
pub fn get_current_program(conn: &Connection, channel_epg_id: &str) -> Result<Option<String>> {
    let now = Utc::now();

    let program: Option<String> = conn
        .query_row(
            "SELECT title FROM epg_programs
             WHERE channel_epg_id = ?1
             AND start_time <= ?2
             AND end_time > ?2
             ORDER BY start_time DESC
             LIMIT 1",
            rusqlite::params![channel_epg_id, now.to_rfc3339()],
            |row| row.get(0),
        )
        .optional()?;

    Ok(program)
}

/// Get next program for a channel
pub fn get_next_program(conn: &Connection, channel_epg_id: &str) -> Result<Option<String>> {
    let now = Utc::now();

    let program: Option<String> = conn
        .query_row(
            "SELECT title FROM epg_programs
             WHERE channel_epg_id = ?1
             AND start_time > ?2
             ORDER BY start_time ASC
             LIMIT 1",
            rusqlite::params![channel_epg_id, now.to_rfc3339()],
            |row| row.get(0),
        )
        .optional()?;

    Ok(program)
}

// Builder struct for constructing programs
struct EpgProgramBuilder {
    channel_id: String,
    title: String,
    description: Option<String>,
    start_time: DateTime<Utc>,
    end_time: DateTime<Utc>,
    category: Option<String>,
}

impl EpgProgramBuilder {
    fn build(self) -> EpgProgram {
        EpgProgram {
            channel_id: self.channel_id,
            title: self.title,
            description: self.description,
            start_time: self.start_time,
            end_time: self.end_time,
            category: self.category,
        }
    }
}
