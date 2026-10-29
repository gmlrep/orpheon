"""Extract settings catalog pref key→label pairs for i18n scaffolding."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
text = (ROOT / "src" / "settings" / "catalog.ts").read_text(encoding="utf-8")
pat = re.compile(r'\b[tsvpb]\(\s*"([^"]+)"\s*,\s*"((?:\\.|[^"\\])*)"')
seen: dict[str, str] = {}
for m in pat.finditer(text):
    key, label = m.group(1), m.group(2)
    seen.setdefault(key, label)

out = ROOT / "scripts" / "_pref_labels.txt"
out.write_text("\n".join(f"{k}\t{v}" for k, v in sorted(seen.items())) + "\n", encoding="utf-8")
print(f"{len(seen)} labels → {out}")
