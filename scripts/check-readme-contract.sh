#!/usr/bin/env bash
# Verify public onboarding promises, not README layout or prose styling.
set -euo pipefail

readme="README.md"
required=(
  'cargo install hevy-rs --locked'
  'npx skills add diegaccio/hevy-rs --skill hevy-rs'
  "HEVY_API_KEY='your-api-key'"
  'hevy-rs --format json user get'
  'hevy-rs workouts --help'
  'fresh, explicit approval'
)

for promise in "${required[@]}"; do
  if ! grep --fixed-strings --quiet -- "$promise" "$readme"; then
    printf 'README onboarding promise is missing: %s\n' "$promise" >&2
    exit 1
  fi
done

python3 - "$readme" <<'PY'
import re
import sys
from pathlib import Path
from urllib.parse import unquote, urlparse

readme = Path(sys.argv[1])
for target in re.findall(r"\]\(([^ )]+)(?:\s+[^)]*)?\)", readme.read_text(encoding="utf-8")):
    parsed = urlparse(target)
    if parsed.scheme or parsed.netloc or target.startswith("#"):
        continue
    path = readme.parent / unquote(parsed.path)
    if not path.is_file():
        raise SystemExit(f"README local documentation target is missing: {target}")
PY
