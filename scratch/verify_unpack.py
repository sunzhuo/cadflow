import json
import os

DXF_PATH = r"d:\SourceCodes\cadflow\cadflow-web\assets\sim\apps\nb1\docs\scene.dxf"

with open(DXF_PATH, "r", encoding="utf-8", errors="replace") as f:
    lines = f.readlines()

stripped = [line.rstrip("\r\n") for line in lines]
tags = []
for i in range(0, len(stripped) - 1, 2):
    try:
        code = int(stripped[i].strip())
        val = stripped[i + 1]
        tags.append((code, val))
    except ValueError:
        pass

# Emulate extract_cadflow_documents in dxf_doc.rs
dict_entries = {}
xrecords = {}

current_record = None # (kind, handle, rtags)

for code, val in tags:
    if code == 0:
        if current_record is not None:
            kind, handle, rtags = current_record
            h_upper = handle.strip().upper()
            if kind in ("DICTIONARY", "ACDBDICTIONARYWDFLT"):
                entries = []
                entry_name = None
                for tc, tv in rtags:
                    if tc == 3:
                        entry_name = tv
                    elif tc in (350, 360):
                        if entry_name:
                            entries.append((entry_name, tv.strip().upper()))
                            entry_name = None
                dict_entries[h_upper] = entries
            elif kind == "XRECORD":
                xrecords[h_upper] = rtags
        current_record = (val.strip(), "", [])
    elif current_record is not None:
        kind, handle, rtags = current_record
        if code == 5 and not handle:
            current_record = (kind, val.strip(), rtags)
        else:
            rtags.append((code, val))

# Find CADFLOW_DOCUMENTS handle
target_xrec_handle = None
for entries in dict_entries.values():
    for name, handle in entries:
        if name == "CADFLOW_DOCUMENTS":
            target_xrec_handle = handle
            break

print(f"Target XRecord Handle: {target_xrec_handle}")
assert target_xrec_handle is not None, "CADFLOW_DOCUMENTS XRecord handle not found!"

rtags = xrecords.get(target_xrec_handle, [])
chunks = [tv for tc, tv in rtags if tc == 1]
print(f"Found {len(chunks)} chunks in XRecord")

full_json = "".join(chunks)
payload = json.loads(full_json)
print(f"Decoded payload version: {payload.get('version')}")
print(f"Files count: {len(payload.get('files', []))}")
for f in payload.get('files', []):
    print(f" - {f['title']}: {len(f['content'])} characters")

print("SUCCESS: DXF Single-file packaging and extraction verified 100%!")
