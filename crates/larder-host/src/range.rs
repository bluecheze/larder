/// A byte range to serve, end is inclusive like in the HTTP header.
#[derive(Debug, PartialEq, Eq)]
pub struct ByteRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum RangeRequest {
    Full,
    Partial(ByteRange),
    Unsatisfiable,
}

/// Handles the three forms you actually see: `bytes=10-20`, `bytes=10-`
/// and `bytes=-500`. Anything else, including multiple ranges, just gets
/// the whole file. That's allowed by the spec and nobody's WebDAV client
/// sends multi-range requests anyway.
pub fn parse_range(header: Option<&str>, size: u64) -> RangeRequest {
    let Some(spec) = header.and_then(|h| h.trim().strip_prefix("bytes=")) else {
        return RangeRequest::Full;
    };
    if spec.contains(',') {
        return RangeRequest::Full;
    }
    let Some((start, end)) = spec.split_once('-') else {
        return RangeRequest::Full;
    };
    let parsed = match (start.trim(), end.trim()) {
        ("", suffix) => suffix.parse::<u64>().ok().map(|n| (size.saturating_sub(n), size.saturating_sub(1))),
        (start, "") => start.parse().ok().map(|s| (s, size.saturating_sub(1))),
        (start, end) => match (start.parse(), end.parse::<u64>()) {
            (Ok(s), Ok(e)) => Some((s, e.min(size.saturating_sub(1)))),
            _ => None,
        },
    };
    match parsed {
        None => RangeRequest::Full,
        Some((start, end)) if size == 0 || start > end || start >= size => RangeRequest::Unsatisfiable,
        Some((start, end)) => RangeRequest::Partial(ByteRange { start, end }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn part(start: u64, end: u64) -> RangeRequest {
        RangeRequest::Partial(ByteRange { start, end })
    }

    #[test]
    fn common_forms() {
        assert_eq!(parse_range(Some("bytes=0-9"), 100), part(0, 9));
        assert_eq!(parse_range(Some("bytes=90-"), 100), part(90, 99));
        assert_eq!(parse_range(Some("bytes=-10"), 100), part(90, 99));
        assert_eq!(parse_range(Some("bytes=50-500"), 100), part(50, 99));
    }

    #[test]
    fn falls_back_to_full_file() {
        assert_eq!(parse_range(None, 100), RangeRequest::Full);
        assert_eq!(parse_range(Some("bytes=0-1,5-6"), 100), RangeRequest::Full);
        assert_eq!(parse_range(Some("items=0-1"), 100), RangeRequest::Full);
        assert_eq!(parse_range(Some("bytes=abc"), 100), RangeRequest::Full);
    }

    #[test]
    fn out_of_bounds() {
        assert_eq!(parse_range(Some("bytes=100-"), 100), RangeRequest::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=0-0"), 0), RangeRequest::Unsatisfiable);
        assert_eq!(parse_range(Some("bytes=9-3"), 100), RangeRequest::Unsatisfiable);
    }
}
