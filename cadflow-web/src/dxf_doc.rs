//! Serialization of CADFlow Markdown documents into DXF XRECORD & Extension Dictionary.
//!
//! Stores multiple Markdown documents packed into an AcDbXrecord under an extension
//! dictionary (AcDbDictionary) inside the drawing's OBJECTS section.

use cadcraft_dxf::Tag;
use serde::{Deserialize, Serialize};

pub const EXT_DICT_NAME: &str = "CADFLOW_DOCUMENTS_DICT";
pub const XRECORD_NAME: &str = "CADFLOW_DOCUMENTS";

thread_local! {
    static CURRENT_DOCS_PAYLOAD: std::cell::RefCell<Option<CadflowDocsPayload>> = const { std::cell::RefCell::new(None) };
}

pub fn set_current_docs(payload: CadflowDocsPayload) {
    CURRENT_DOCS_PAYLOAD.with(|cell| {
        *cell.borrow_mut() = Some(payload);
    });
}

pub fn get_current_docs() -> Option<CadflowDocsPayload> {
    CURRENT_DOCS_PAYLOAD.with(|cell| cell.borrow().clone())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CadflowDocsPayload {
    pub version: u32,
    pub files: Vec<DocFilePayload>,
    pub active_index: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocFilePayload {
    pub title: String,
    pub content: String,
}

impl Default for CadflowDocsPayload {
    fn default() -> Self {
        Self {
            version: 1,
            files: Vec::new(),
            active_index: 0,
        }
    }
}

/// JSON text split into XRECORD strings of at most 250 characters.
/// Backslashes are written as `\u005c` so no chunk contains a raw DXF `\U+` escape.
pub fn json_chunks(json: &str) -> Vec<String> {
    let mut safe = String::with_capacity(json.len());
    let mut it = json.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\\' {
            match it.next() {
                Some('\\') => safe.push_str("\\u005c"),
                Some(n) => {
                    safe.push('\\');
                    safe.push(n);
                }
                None => safe.push('\\'),
            }
        } else {
            safe.push(c);
        }
    }
    let chars: Vec<char> = safe.chars().collect();
    chars.chunks(250).map(|c| c.iter().collect()).collect()
}

/// Extract packed Markdown documents from a DXF file's bytes.
pub fn extract_cadflow_documents(bytes: &[u8]) -> Option<CadflowDocsPayload> {
    let tags = cadcraft_dxf::parse(bytes).ok()?;

    // Step 1: Collect dictionary mappings and XRECORD tags
    let mut dict_entries: std::collections::HashMap<String, Vec<(String, String)>> = std::collections::HashMap::new();
    let mut xrecords: std::collections::HashMap<String, Vec<Tag>> = std::collections::HashMap::new();

    let mut current_record: Option<(String, String, Vec<Tag>)> = None; // (kind, handle, tags)

    let mut it = tags.into_iter();
    while let Some(t) = it.next() {
        if t.code == 0 {
            if let Some((kind, handle, rtags)) = current_record.take() {
                let h_upper = handle.trim().to_ascii_uppercase();
                match kind.as_str() {
                    "DICTIONARY" | "ACDBDICTIONARYWDFLT" => {
                        let mut entries = Vec::new();
                        let mut entry_name: Option<String> = None;
                        for tag in &rtags {
                            match tag.code {
                                3 => entry_name = Some(tag.str()),
                                350 | 360 => {
                                    if let Some(name) = entry_name.take() {
                                        entries.push((name, tag.str().trim().to_ascii_uppercase()));
                                    }
                                }
                                _ => {}
                            }
                        }
                        dict_entries.insert(h_upper, entries);
                    }
                    "XRECORD" => {
                        xrecords.insert(h_upper, rtags);
                    }
                    _ => {}
                }
            }
            let kind = t.str();
            current_record = Some((kind, String::new(), Vec::new()));
        } else if let Some((_, handle, rtags)) = current_record.as_mut() {
            if t.code == 5 && handle.is_empty() {
                *handle = t.str();
            }
            rtags.push(t);
        }
    }

    if let Some((kind, handle, rtags)) = current_record {
        let h_upper = handle.trim().to_ascii_uppercase();
        match kind.as_str() {
            "DICTIONARY" | "ACDBDICTIONARYWDFLT" => {
                let mut entries = Vec::new();
                let mut entry_name: Option<String> = None;
                for tag in &rtags {
                    match tag.code {
                        3 => entry_name = Some(tag.str()),
                        350 | 360 => {
                            if let Some(name) = entry_name.take() {
                                entries.push((name, tag.str().trim().to_ascii_uppercase()));
                            }
                        }
                        _ => {}
                    }
                }
                dict_entries.insert(h_upper, entries);
            }
            "XRECORD" => {
                xrecords.insert(h_upper, rtags);
            }
            _ => {}
        }
    }

    // Step 2: Search for CADFLOW_DOCUMENTS handle in dictionaries
    let mut target_xrec_handle: Option<String> = None;

    // Check dictionaries for entry XRECORD_NAME
    for entries in dict_entries.values() {
        for (name, target_h) in entries {
            if name.eq_ignore_ascii_case(XRECORD_NAME) {
                target_xrec_handle = Some(target_h.clone());
                break;
            }
        }
        if target_xrec_handle.is_some() {
            break;
        }
    }

    // If target xrecord found by handle:
    if let Some(h) = target_xrec_handle {
        if let Some(tags) = xrecords.get(&h) {
            if let Some(payload) = parse_xrecord_payload(tags) {
                return Some(payload);
            }
        }
    }

    // Fallback: Check all XRECORDs for a valid CadflowDocsPayload
    for tags in xrecords.values() {
        if let Some(payload) = parse_xrecord_payload(tags) {
            return Some(payload);
        }
    }

    None
}

fn parse_xrecord_payload(tags: &[Tag]) -> Option<CadflowDocsPayload> {
    let mut json_str = String::new();
    for t in tags {
        if t.code == 1 || t.code == 3 {
            json_str.push_str(&t.str());
        }
    }
    if json_str.contains("\"version\"") && json_str.contains("\"files\"") {
        serde_json::from_str::<CadflowDocsPayload>(&json_str).ok()
    } else {
        None
    }
}

/// Inject the packed Markdown documents into a DXF file's bytes.
pub fn inject_cadflow_documents(bytes: &[u8], payload: &CadflowDocsPayload) -> Vec<u8> {
    let is_binary = cadcraft_dxf::is_binary(bytes);
    let mut tags = match cadcraft_dxf::parse(bytes) {
        Ok(t) => t,
        Err(_) => return bytes.to_vec(),
    };

    // 1. Scan maximum existing handle and handseed
    let mut max_handle = 0u64;
    let mut handseed_pos = None;

    for (i, t) in tags.iter().enumerate() {
        if t.code == 5 {
            if let Ok(h) = u64::from_str_radix(t.str().trim(), 16) {
                max_handle = max_handle.max(h);
            }
        } else if t.code == 9 && t.str().trim() == "$HANDSEED" {
            handseed_pos = Some(i + 1);
        }
    }

    if let Some(pos) = handseed_pos {
        if let Some(t) = tags.get(pos) {
            if t.code == 5 {
                if let Ok(seed) = u64::from_str_radix(t.str().trim(), 16) {
                    max_handle = max_handle.max(seed);
                }
            }
        }
    }

    let ext_dict_handle = max_handle.saturating_add(1);
    let xrec_handle = max_handle.saturating_add(2);
    let new_seed = max_handle.saturating_add(3);

    // Update $HANDSEED
    if let Some(pos) = handseed_pos {
        if let Some(t) = tags.get_mut(pos) {
            if t.code == 5 {
                *t = Tag::s(5, format!("{:X}", new_seed));
            }
        }
    }

    // 2. Find OBJECTS section and root dictionary
    let mut objects_sec_start = None;
    let mut objects_endsec = None;
    let mut root_dict_handle = "C".to_string(); // fallback default

    let mut in_objects = false;
    for (i, t) in tags.iter().enumerate() {
        if t.code == 0 && t.str() == "SECTION" {
            if let Some(next) = tags.get(i + 1) {
                if next.code == 2 && next.str() == "OBJECTS" {
                    in_objects = true;
                    objects_sec_start = Some(i);
                }
            }
        } else if in_objects && t.code == 0 && t.str() == "ENDSEC" {
            objects_endsec = Some(i);
            break;
        }
    }

    if let Some(start) = objects_sec_start {
        // Find first DICTIONARY record in OBJECTS section (Named Object Dictionary)
        let end = objects_endsec.unwrap_or(tags.len());
        let mut in_root_dict = false;
        let mut root_dict_end = start + 2;

        let mut i = start + 2;
        while i < end {
            if tags[i].code == 0 && tags[i].str() == "DICTIONARY" {
                in_root_dict = true;
                if let Some(h_tag) = tags.get(i + 1) {
                    if h_tag.code == 5 {
                        root_dict_handle = h_tag.str();
                    }
                }
                i += 2;
                while i < end && tags[i].code != 0 {
                    i += 1;
                }
                root_dict_end = i;
                break;
            }
            i += 1;
        }

        if in_root_dict {
            // Remove any existing EXT_DICT_NAME entry from root dictionary
            let mut remove_indices = Vec::new();
            let mut j = start + 2;
            while j < root_dict_end {
                if tags[j].code == 3 && tags[j].str().eq_ignore_ascii_case(EXT_DICT_NAME) {
                    remove_indices.push(j);
                    if j + 1 < root_dict_end && (tags[j + 1].code == 350 || tags[j + 1].code == 360) {
                        remove_indices.push(j + 1);
                    }
                }
                j += 1;
            }
            for idx in remove_indices.into_iter().rev() {
                tags.remove(idx);
                root_dict_end -= 1;
                if let Some(e) = objects_endsec.as_mut() {
                    *e -= 1;
                }
            }

            // Insert entry into root dictionary
            tags.insert(root_dict_end, Tag::s(3, EXT_DICT_NAME));
            tags.insert(root_dict_end + 1, Tag::s(350, format!("{:X}", ext_dict_handle)));
            if let Some(e) = objects_endsec.as_mut() {
                *e += 2;
            }
        }
    } else {
        // Create an OBJECTS section before EOF if not present
        let eof_idx = tags.iter().position(|t| t.code == 0 && t.str() == "EOF").unwrap_or(tags.len());
        let root_h = format!("{:X}", max_handle.saturating_add(4));
        root_dict_handle = root_h.clone();

        let new_objects = vec![
            Tag::s(0, "SECTION"),
            Tag::s(2, "OBJECTS"),
            Tag::s(0, "DICTIONARY"),
            Tag::s(5, root_h),
            Tag::s(330, "0"),
            Tag::s(100, "AcDbDictionary"),
            Tag::i(281, 1),
            Tag::s(3, EXT_DICT_NAME),
            Tag::s(350, format!("{:X}", ext_dict_handle)),
            Tag::s(0, "ENDSEC"),
        ];
        tags.splice(eof_idx..eof_idx, new_objects);
        objects_endsec = Some(eof_idx + 9);
    }

    // 3. Append Extension Dictionary and XRECORD before OBJECTS ENDSEC
    let endsec_pos = objects_endsec.unwrap_or_else(|| tags.len().saturating_sub(1));

    let mut ext_objects = vec![
        // Extension Dictionary (AcDbDictionary)
        Tag::s(0, "DICTIONARY"),
        Tag::s(5, format!("{:X}", ext_dict_handle)),
        Tag::s(102, "{ACAD_REACTORS"),
        Tag::s(330, root_dict_handle.clone()),
        Tag::s(102, "}"),
        Tag::s(330, root_dict_handle),
        Tag::s(100, "AcDbDictionary"),
        Tag::i(281, 1),
        Tag::s(3, XRECORD_NAME),
        Tag::s(350, format!("{:X}", xrec_handle)),
        // XRECORD (AcDbXrecord)
        Tag::s(0, "XRECORD"),
        Tag::s(5, format!("{:X}", xrec_handle)),
        Tag::s(102, "{ACAD_REACTORS"),
        Tag::s(330, format!("{:X}", ext_dict_handle)),
        Tag::s(102, "}"),
        Tag::s(330, format!("{:X}", ext_dict_handle)),
        Tag::s(100, "AcDbXrecord"),
        Tag::i(280, 1),
    ];

    let json = serde_json::to_string(payload).unwrap_or_default();
    let chunks = json_chunks(&json);
    for chunk in chunks {
        ext_objects.push(Tag::s(1, chunk));
    }

    tags.splice(endsec_pos..endsec_pos, ext_objects);

    if is_binary {
        cadcraft_dxf::write_binary(&tags, true)
    } else {
        cadcraft_dxf::write_ascii(&tags).into_bytes()
    }
}

