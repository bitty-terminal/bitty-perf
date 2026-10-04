//! Harness-pin PERF-01 contract (CTX-0686).
//!
//! CI half of the PB harness/corpora/machines pin record. Keeps the inventory
//! honest: the pin doc must name every committed artifact, and every named
//! artifact must exist with schema and provenance intact. No display,
//! network, clock, RNG, or host path participates; `forbid(unsafe)`.

#![forbid(unsafe_code)]

use bitty_perf::real_window::workspace_root;

/// Committed PB artifacts pinned by `baselines/harness-pin.md`.
const PINNED_ARTIFACTS: &[&str] = &[
    "crates/bitty-perf/baselines/pb-real-window.json",
    "crates/bitty-perf/baselines/pb-typical-session.json",
    "crates/bitty-perf/baselines/pb-latency.json",
    "crates/bitty-perf/baselines/pb-throughput-floor.json",
    "crates/bitty-perf/baselines/pb-idle.json",
    "crates/bitty-perf/baselines/parser-throughput.json",
    "crates/bitty-perf/baselines/pb-real-soak.json",
    "crates/bitty-perf/baselines/pb-dogfood-session.json",
];

#[test]
fn pin_record_names_every_budget_and_artifact() {
    let doc = std::fs::read_to_string(
        workspace_root().join("crates/bitty-perf/baselines/harness-pin.md"),
    )
    .expect("harness-pin.md must exist");
    for budget in ["PB-1", "PB-2", "PB-3", "PB-4", "PB-5", "PB-6", "PB-7"] {
        assert!(doc.contains(budget), "pin record must name {budget}");
    }
    for artifact in PINNED_ARTIFACTS {
        let file = artifact.rsplit('/').next().unwrap_or(artifact);
        assert!(
            doc.contains(file),
            "pin record must name pinned artifact {file}"
        );
    }
    assert!(
        doc.contains("OQ-100"),
        "pin record must state the OQ-100 gating block"
    );
    assert!(
        !doc.contains("/home/") && !doc.contains("/mnt/") && !doc.contains("C:\\Users"),
        "pin record must not embed a host path"
    );
}

#[test]
fn every_pinned_artifact_exists_with_schema_and_provenance() {
    for artifact in PINNED_ARTIFACTS {
        let path = workspace_root().join(artifact);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("pinned artifact {} must exist: {e}", path.display()));
        assert!(
            is_json_object(&text),
            "pinned artifact {artifact} must parse as a JSON object"
        );
        let version = json_number_field(&text, "schema_version").unwrap_or_else(|| {
            panic!("pinned artifact {artifact} must carry a numeric schema_version")
        });
        assert!(
            version >= 1.0,
            "pinned artifact {artifact} has a bad schema_version {version}"
        );
        for key in ["task", "captured_at", "revision"] {
            let value = json_string_field(&text, key).unwrap_or_else(|| {
                panic!("pinned artifact {artifact} must carry {key} as a JSON string")
            });
            assert!(
                !value.is_empty() && !value.contains("unspecified") && !value.contains("pending"),
                "pinned artifact {artifact} has placeholder provenance in {key}: {value}"
            );
        }
        assert!(
            !text.contains("/home/") && !text.contains("/mnt/") && !text.contains("C:\\Users"),
            "pinned artifact {artifact} must not embed a host path"
        );
    }
}

/// `true` when `text` is a brace-balanced JSON object (string-aware: braces
/// inside quoted strings and escaped quotes do not count).
fn is_json_object(text: &str) -> bool {
    let trimmed = text.trim();
    if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
        return false;
    }
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escaped = false;
    for ch in trimmed.chars() {
        if in_string {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '"' {
                in_string = false;
            }
        } else if ch == '"' {
            in_string = true;
        } else if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth -= 1;
            if depth < 0 {
                return false;
            }
        }
    }
    !in_string && depth == 0
}

/// Extract the decoded value of a top-level `"key": "string"` pair, handling
/// `\"` and `\\` escapes. Returns `None` when the key is absent, `null`, or
/// not a JSON string (so `null` provenance can never pass).
fn json_string_field(text: &str, key: &str) -> Option<String> {
    let value = json_raw_value(text, key)?;
    if !value.starts_with('"') {
        return None;
    }
    let mut out = String::new();
    let mut chars = value[1..].chars();
    loop {
        match chars.next()? {
            '\\' => out.push(chars.next()?),
            '"' => return Some(out),
            ch => out.push(ch),
        }
    }
}

/// Extract the value of a top-level `"key": <number>` pair.
fn json_number_field(text: &str, key: &str) -> Option<f64> {
    json_raw_value(text, key)?.parse::<f64>().ok()
}

/// Slice of the raw JSON value following a top-level `"key":` pair: a quoted
/// string (through its closing quote), a number/boolean/`null` literal (to
/// the next structural delimiter), or an object/array (balanced).
fn json_raw_value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    let needle = format!("\"{key}\"");
    let mut search = text;
    let mut offset = 0usize;
    loop {
        let found = search.find(&needle)?;
        let abs = offset + found;
        let mut rest = text[abs + needle.len()..].trim_start();
        if !rest.starts_with(':') {
            search = &search[found + needle.len()..];
            offset = abs + needle.len();
            continue;
        }
        rest = rest[1..].trim_start();
        if let Some(stripped) = rest.strip_prefix('"') {
            let mut escaped = false;
            for (i, ch) in stripped.char_indices() {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    return Some(&rest[..i + 2]);
                }
            }
            return None;
        }
        let end = rest.find([',', '}', ']', '\n']).unwrap_or(rest.len());
        return Some(rest[..end].trim_end());
    }
}

#[test]
fn field_extractors_reject_absent_null_and_non_string_values() {
    let text = "{\"task\": \"CTX-0003\", \"revision\": null, \"count\": 3}";
    assert_eq!(json_string_field(text, "task").as_deref(), Some("CTX-0003"));
    assert_eq!(json_string_field(text, "revision"), None);
    assert_eq!(json_string_field(text, "missing"), None);
    assert_eq!(json_string_field(text, "count"), None);
    assert_eq!(json_number_field(text, "count"), Some(3.0));
    assert_eq!(json_number_field(text, "task"), None);
    // Escaped quotes inside a value do not end the string early.
    let escaped = "{\"command\": \"say \\\"hi\\\"\"}";
    assert_eq!(
        json_string_field(escaped, "command").as_deref(),
        Some("say \"hi\"")
    );
    assert!(!is_json_object("{\"a\": "));
    assert!(is_json_object(text));
    // A missing value is not a string: the extractor rejects it.
    assert_eq!(json_string_field("{\"a\": }", "a"), None);
}
