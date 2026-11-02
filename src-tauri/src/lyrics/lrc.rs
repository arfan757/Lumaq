use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    pub time_ms: u64,
    pub text: String,
}

/// Parses standard and extended LRC format into sorted LyricLine entries.
///
/// Features supported:
/// - Standard timestamps: `[mm:ss.xx]` (hundredths of second)
/// - Millisecond timestamps: `[mm:ss.xxx]`
/// - Multiple timestamps per line: `[00:12.00][00:24.00]Chorus line`
/// - Global offset tag: `[offset:+/-ms]`
/// - Metadata tag skipping: `[ti:...]`, `[ar:...]`, `[al:...]`, `[by:...]`, etc.
/// - Intentional instrumental gaps preserved (empty text lines).
/// - Sorted in ascending order of `time_ms`.
pub fn parse_lrc(lrc_content: &str) -> Vec<LyricLine> {
    let mut offset_ms: i64 = 0;
    let mut raw_lines: Vec<(i64, String)> = Vec::new();

    for line in lrc_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Check for global [offset:+/-ms] tag
        if trimmed.starts_with("[offset:") && trimmed.ends_with(']') {
            let val_str = &trimmed[8..trimmed.len() - 1].trim();
            if let Ok(parsed_offset) = val_str.parse::<i64>() {
                offset_ms = parsed_offset;
            }
            continue;
        }

        // Parse one or more timestamp tags at the beginning of the line
        let mut rest = trimmed;
        let mut line_timestamps: Vec<u64> = Vec::new();

        while rest.starts_with('[') {
            if let Some(close_bracket) = rest.find(']') {
                let tag = &rest[1..close_bracket];
                if let Some(time) = parse_timestamp(tag) {
                    line_timestamps.push(time);
                    rest = rest[close_bracket + 1..].trim_start();
                } else {
                    // Could be metadata tag like [ti:Title] or unknown tag
                    // If we haven't found any timestamps yet and this is not a timestamp,
                    // ignore this metadata tag or whole line
                    if line_timestamps.is_empty() {
                        break;
                    }
                    // Otherwise break out of timestamp parsing
                    break;
                }
            } else {
                break;
            }
        }

        if !line_timestamps.is_empty() {
            let text = rest.to_string();
            for time in line_timestamps {
                raw_lines.push((time as i64, text.clone()));
            }
        }
    }

    // Apply offset and filter out negative timestamps, sort by time
    let mut result: Vec<LyricLine> = raw_lines
        .into_iter()
        .map(|(t, text)| {
            let final_time = (t + offset_ms).max(0) as u64;
            LyricLine {
                time_ms: final_time,
                text,
            }
        })
        .collect();

    result.sort_by_key(|l| l.time_ms);
    result
}

/// Parses `mm:ss.xx` or `mm:ss.xxx` or `mm:ss` into milliseconds.
fn parse_timestamp(tag: &str) -> Option<u64> {
    let parts: Vec<&str> = tag.split(':').collect();
    if parts.len() != 2 {
        return None;
    }

    let minutes: u64 = parts[0].trim().parse().ok()?;
    let sec_parts: Vec<&str> = parts[1].split('.').collect();

    if sec_parts.len() == 1 {
        let seconds: u64 = sec_parts[0].trim().parse().ok()?;
        return Some(minutes * 60_000 + seconds * 1_000);
    } else if sec_parts.len() == 2 {
        let seconds: u64 = sec_parts[0].trim().parse().ok()?;
        let frac_str = sec_parts[1].trim();

        let ms: u64 = match frac_str.len() {
            1 => frac_str.parse::<u64>().ok()? * 100,
            2 => frac_str.parse::<u64>().ok()? * 10,
            3 => frac_str.parse::<u64>().ok()?,
            len if len > 3 => frac_str[..3].parse::<u64>().ok()?,
            _ => 0,
        };

        return Some(minutes * 60_000 + seconds * 1_000 + ms);
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_lrc() {
        let input = r#"
[ti:Test Title]
[ar:Test Artist]
[00:01.50]First sample line
[00:04.200]Second sample line with 3 digits
[00:08.00]
[00:10.00]Line after intentional gap
"#;
        let lines = parse_lrc(input);
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[0].time_ms, 1500);
        assert_eq!(lines[0].text, "First sample line");
        assert_eq!(lines[1].time_ms, 4200);
        assert_eq!(lines[1].text, "Second sample line with 3 digits");
        assert_eq!(lines[2].time_ms, 8000);
        assert_eq!(lines[2].text, "");
        assert_eq!(lines[3].time_ms, 10000);
        assert_eq!(lines[3].text, "Line after intentional gap");
    }

    #[test]
    fn test_multiple_timestamps_and_offset() {
        let input = r#"
[offset:+500]
[00:02.00][00:06.00]Repeated sample chorus
"#;
        let lines = parse_lrc(input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].time_ms, 2500);
        assert_eq!(lines[0].text, "Repeated sample chorus");
        assert_eq!(lines[1].time_ms, 6500);
        assert_eq!(lines[1].text, "Repeated sample chorus");
    }

    #[test]
    fn test_negative_offset_clamping() {
        let input = r#"
[offset:-3000]
[00:01.00]Early line
[00:05.00]Later line
"#;
        let lines = parse_lrc(input);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].time_ms, 0); // clamped to 0
        assert_eq!(lines[1].time_ms, 2000);
    }

    #[test]
    fn test_empty_and_malformed() {
        let input = r#"
[unclosed tag
[00:01.xx]Bad digits
[00:02.50]Valid line
"#;
        let lines = parse_lrc(input);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].time_ms, 2500);
        assert_eq!(lines[0].text, "Valid line");
    }
}

