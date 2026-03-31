use alloc::string::String;

/// Container for custom HTTP response headers.
pub struct RespHeaders {
    /// The custom installer protocol version.
    ///
    /// Bumped on breaking changes, to ensure that the server and the client are compatible.
    pub protocol: u8,
    pub author_id: String,
    pub app_id: String,
    pub today: (u16, u8, u8),
    pub expected_size: u32,
}

impl RespHeaders {
    pub fn parse(raw: &[u8]) -> Result<Self, &'static str> {
        let mut res = Self {
            protocol: 0,
            author_id: String::new(),
            app_id: String::new(),
            today: (0, 0, 0),
            expected_size: 0,
        };
        // TODO: validate status code.
        for line in raw.split(|c| *c == b'\n') {
            // Parse the HTTP header.
            let line = line.trim_ascii();
            let Some(line) = line.strip_prefix(b"X-F0-") else {
                continue;
            };
            let Some(idx) = line.iter().position(|&c| c == b':') else {
                return Err("invalid header received");
            };
            let (key, value) = line.split_at(idx);
            let key = key.trim_ascii();
            let key = key.to_ascii_lowercase();
            let key = unsafe { str::from_utf8_unchecked(&key) };
            let Ok(value) = str::from_utf8(value.trim_ascii()) else {
                return Err("invalid header encoding");
            };

            match key {
                "protocol" => {
                    if value != "1" {
                        return Err("unsupported protocol");
                    }
                    res.protocol = 1;
                }
                "author-id" => res.author_id = String::from(value),
                "app-id" => res.app_id = String::from(value),
                "today" => res.today = parse_date(value)?,
                "expected-size" => {}
                _ => return Err("unsupported header received"),
            }
        }

        // Validate that all expected headers are set.
        if res.protocol == 0 {
            return Err("protocol not specified");
        }
        if res.today.0 == 0 {
            return Err("current date not specified");
        }
        if res.author_id.is_empty() {
            return Err("author ID not specified");
        }
        if res.app_id.is_empty() {
            return Err("author ID not specified");
        }

        Ok(res)
    }
}

fn parse_date(raw: &str) -> Result<(u16, u8, u8), &'static str> {
    // Split the date parts.
    let Some((year, rest)) = raw.split_once('-') else {
        return Err("invalid date format");
    };
    let Some((month, day)) = rest.split_once('-') else {
        return Err("invalid date format");
    };

    // Parse parts to integers.
    let Ok(year) = year.parse::<u16>() else {
        return Err("invalid year");
    };
    let Ok(month) = month.parse::<u8>() else {
        return Err("invalid month");
    };
    let Ok(day) = day.parse::<u8>() else {
        return Err("invalid day");
    };

    // Validate ranges.
    if !(2024..=3000).contains(&year) {
        return Err("invalid year");
    }
    if month == 0 || month > 12 {
        return Err("invalid month");
    }
    if day == 0 || day > 31 {
        return Err("invalid day");
    }
    Ok((year, month, day))
}
