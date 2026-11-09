import json
import os

DOCS_DIR = r"d:\SourceCodes\cadflow\cadflow-web\assets\sim\apps\nb1\docs"
CODE_DIR = r"d:\SourceCodes\cadflow\cadflow-web\assets\sim\apps\nb1\code"
DXF_PATH = os.path.join(DOCS_DIR, "scene.dxf")

# 1. Read files
with open(os.path.join(DOCS_DIR, "scene.md"), "r", encoding="utf-8") as f:
    scene_md = f.read()

with open(os.path.join(CODE_DIR, "scene.js"), "r", encoding="utf-8") as f:
    scene_js = f.read()

with open(os.path.join(CODE_DIR, "settings.js"), "r", encoding="utf-8") as f:
    settings_js = f.read()

payload = {
    "version": 1,
    "files": [
        {"title": "scene.md", "content": scene_md},
        {"title": "scene.js", "content": scene_js},
        {"title": "settings.js", "content": settings_js},
    ],
    "active_index": 0,
}

json_str = json.dumps(payload, ensure_ascii=False)

def json_chunks(s):
    safe = ""
    i = 0
    while i < len(s):
        c = s[i]
        if c == "\\":
            if i + 1 < len(s) and s[i + 1] == "\\":
                safe += "\\u005c"
                i += 2
            elif i + 1 < len(s):
                safe += "\\" + s[i + 1]
                i += 2
            else:
                safe += "\\"
                i += 1
        else:
            safe += c
            i += 1
    return [safe[j : j + 250] for j in range(0, len(safe), 250)]

chunks = json_chunks(json_str)
print(f"Payload JSON size: {len(json_str)} chars, chunks: {len(chunks)}")

# Read DXF
with open(DXF_PATH, "r", encoding="utf-8", errors="replace") as f:
    lines = f.readlines()

# Strip newlines
stripped = [line.rstrip("\r\n") for line in lines]

# Parse tag pairs (code, value)
tags = []
for i in range(0, len(stripped) - 1, 2):
    code_str = stripped[i].strip()
    try:
        code = int(code_str)
        val = stripped[i + 1]
        tags.append((code, val))
    except ValueError:
        pass

# Scan max handle and $HANDSEED
max_handle = 0
handseed_idx = None

for idx, (code, val) in enumerate(tags):
    if code == 5:
        try:
            h = int(val.strip(), 16)
            if h > max_handle:
                max_handle = h
        except ValueError:
            pass
    elif code == 9 and val.strip() == "$HANDSEED":
        handseed_idx = idx + 1

if handseed_idx is not None and handseed_idx < len(tags):
    code, val = tags[handseed_idx]
    if code == 5:
        try:
            h = int(val.strip(), 16)
            if h > max_handle:
                max_handle = h
        except ValueError:
            pass

ext_dict_handle = max_handle + 1
xrec_handle = max_handle + 2
new_seed = max_handle + 3

print(f"Max handle: {hex(max_handle)}, ExtDict: {hex(ext_dict_handle)}, XRec: {hex(xrec_handle)}")

# Update HANDSEED
if handseed_idx is not None:
    tags[handseed_idx] = (5, f"{new_seed:X}")

# Find OBJECTS section and root dictionary
objects_sec_start = None
objects_endsec = None

in_objects = False
for idx, (code, val) in enumerate(tags):
    if code == 0 and val.strip() == "SECTION":
        if idx + 1 < len(tags) and tags[idx + 1][0] == 2 and tags[idx + 1][1].strip() == "OBJECTS":
            in_objects = True
            objects_sec_start = idx
    elif in_objects and code == 0 and val.strip() == "ENDSEC":
        objects_endsec = idx
        break

root_dict_handle = "C"
if objects_sec_start is not None:
    curr_handle = ""
    curr_kind = ""
    for idx in range(objects_sec_start, objects_endsec):
        code, val = tags[idx]
        if code == 0:
            curr_kind = val.strip()
            curr_handle = ""
        elif code == 5 and not curr_handle:
            curr_handle = val.strip()
            if curr_kind in ("DICTIONARY", "ACDBDICTIONARYWDFLT"):
                root_dict_handle = curr_handle
                break

# Build root dict entry tags
root_entry = [
    (3, "CADFLOW_DOCUMENTS_DICT"),
    (350, f"{ext_dict_handle:X}"),
]

# Insert root_entry inside root dictionary right before next 0 group
inserted_in_root = False
curr_handle = ""
for idx in range(objects_sec_start, objects_endsec):
    code, val = tags[idx]
    if code == 0:
        if curr_handle.upper() == root_dict_handle.upper():
            # insert here
            for r_tag in reversed(root_entry):
                tags.insert(idx, r_tag)
            inserted_in_root = True
            objects_endsec += len(root_entry)
            break
        curr_handle = ""
    elif code == 5 and not curr_handle:
        curr_handle = val.strip()

# Build extension dictionary and XRECORD tags
ext_objects = [
    # Ext Dict
    (0, "DICTIONARY"),
    (5, f"{ext_dict_handle:X}"),
    (102, "{ACAD_REACTORS"),
    (330, root_dict_handle.upper()),
    (102, "}"),
    (330, root_dict_handle.upper()),
    (100, "AcDbDictionary"),
    (281, "1"),
    (3, "CADFLOW_DOCUMENTS"),
    (350, f"{xrec_handle:X}"),
    # XRECORD
    (0, "XRECORD"),
    (5, f"{xrec_handle:X}"),
    (102, "{ACAD_REACTORS"),
    (330, f"{ext_dict_handle:X}"),
    (102, "}"),
    (330, f"{ext_dict_handle:X}"),
    (100, "AcDbXrecord"),
    (280, "1"),
]

for ch in chunks:
    ext_objects.append((1, ch))

# Splice before objects_endsec
tags[objects_endsec:objects_endsec] = ext_objects

# Write back
OUT_PATH = os.path.join(DOCS_DIR, "scene.dxf")
with open(OUT_PATH, "w", encoding="utf-8") as f:
    for code, val in tags:
        f.write(f"{code:>3}\n{val}\n")

print(f"Successfully packaged scene.dxf with {len(chunks)} chunks!")
