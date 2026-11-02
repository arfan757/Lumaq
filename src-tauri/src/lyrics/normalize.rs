/// Normalizes title, artist, and album for LRCLIB fallback search queries.
///
/// Strips common decorative suffixes:
/// - "- Remastered 2011", "- Remastered", "- 2011 Remaster"
/// - "(feat. X)", "[feat. X]", "(with X)", "(ft. X)"
/// - "- Live", "(Live)", "- Radio Edit", "- Single Version"
/// - Trailing "(Deluxe Edition)", "(Expanded Edition)", "[Bonus Track]" from albums
/// - Splits multi-artist strings (e.g. "Artist A, Artist B", "Artist A & Artist B") to first artist
pub fn normalize_title(title: &str) -> String {
    let mut s = title.trim().to_string();

    // Strip (feat. ...), [feat. ...], (ft. ...), (with ...)
    s = remove_parenthetical_pattern(&s, &["feat.", "feat ", "ft.", "ft ", "with "]);

    // Strip common dash suffixes: - Remastered..., - Live..., - Radio Edit...
    let dash_patterns = [
        "- remastered",
        "- remaster",
        "- 20", // e.g. - 2011 Remaster
        "- live",
        "- radio edit",
        "- single version",
        "- original mix",
        "- bonus track",
    ];

    for pattern in &dash_patterns {
        if let Some(idx) = s.to_lowercase().find(pattern) {
            s.truncate(idx);
            break;
        }
    }

    // Strip bracketed [Live], (Live), (Remastered)
    s = remove_parenthetical_pattern(&s, &["live", "remastered", "remaster", "deluxe"]);

    s.trim().to_string()
}

pub fn normalize_artist(artist: &str) -> String {
    let s = artist.trim();

    // Split on common multi-artist delimiters: comma, &, feat., ft.
    let delimiters = [",", " & ", " feat. ", " ft. ", " and ", " / "];
    let mut first_artist = s;

    for delim in &delimiters {
        if let Some((first, _)) = first_artist.split_once(delim) {
            first_artist = first.trim();
        }
    }

    first_artist.trim().to_string()
}

pub fn normalize_album(album: &str) -> String {
    let mut s = album.trim().to_string();

    let album_patterns = [
        "(deluxe edition)",
        "(deluxe version)",
        "(deluxe)",
        "[deluxe edition]",
        "(expanded edition)",
        "(remastered)",
        "- remastered",
        "(special edition)",
        "(bonus track version)",
    ];

    for pattern in &album_patterns {
        if let Some(idx) = s.to_lowercase().find(pattern) {
            s.truncate(idx);
            break;
        }
    }

    s.trim().to_string()
}

fn remove_parenthetical_pattern(input: &str, keywords: &[&str]) -> String {
    let mut output = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '(' || c == '[' {
            let close_char = if c == '(' { ')' } else { ']' };
            let mut inner = String::new();
            let mut closed = false;

            for next_c in chars.by_ref() {
                if next_c == close_char {
                    closed = true;
                    break;
                }
                inner.push(next_c);
            }

            let inner_lower = inner.to_lowercase();
            let contains_keyword = keywords.iter().any(|k| inner_lower.contains(k));

            if !closed || !contains_keyword {
                output.push(c);
                output.push_str(&inner);
                if closed {
                    output.push(close_char);
                }
            }
        } else {
            output.push(c);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_title() {
        assert_eq!(
            normalize_title("Bohemian Rhapsody - Remastered 2011"),
            "Bohemian Rhapsody"
        );
        assert_eq!(normalize_title("Starboy (feat. Daft Punk)"), "Starboy");
        assert_eq!(
            normalize_title("Hotel California - Live"),
            "Hotel California"
        );
        assert_eq!(
            normalize_title("Clint Eastwood - Radio Edit"),
            "Clint Eastwood"
        );
        assert_eq!(
            normalize_title("Comfortably Numb (Live at Pompeii)"),
            "Comfortably Numb"
        );
    }

    #[test]
    fn test_normalize_artist() {
        assert_eq!(normalize_artist("Queen"), "Queen");
        assert_eq!(normalize_artist("The Weeknd, Daft Punk"), "The Weeknd");
        assert_eq!(normalize_artist("Coldplay & BTS"), "Coldplay");
        assert_eq!(normalize_artist("Drake feat. Rihanna"), "Drake");
    }

    #[test]
    fn test_normalize_album() {
        assert_eq!(
            normalize_album("A Night At The Opera (Deluxe Edition)"),
            "A Night At The Opera"
        );
        assert_eq!(normalize_album("Abbey Road (Remastered)"), "Abbey Road");
    }
}
