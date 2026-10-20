#!/usr/bin/env bash
#
# Verify every deck in tests/decks/: `pptd check` -> `pptd build` ->
# validate_pptx.py. Set RENDER=1 to also rasterise each deck through headless
# LibreOffice and write build/preview/slide-NN.png plus a contact sheet.
#
# Usage:
#   ./verify_all.sh              # check + build + validate
#   RENDER=1 ./verify_all.sh     # ... plus rendered previews
#   PPTD=target/debug/pptd ./verify_all.sh
#
# Exit code 0 only when every deck passes every stage.

set -uo pipefail
cd "$(dirname "$0")" || exit 1

PPTD="${PPTD:-pptd}"
PYTHON="${PYTHON:-python3}"
VALIDATE="${VALIDATE:-../../skills/open-pptd/scripts/validate_pptx.py}"
RENDER="${RENDER:-0}"
ONLY="${ONLY:-}"

command -v "$PPTD" >/dev/null 2>&1 || { echo "pptd not found (set PPTD=...)"; exit 1; }
[ -f "$VALIDATE" ] || { echo "validator not found at $VALIDATE (set VALIDATE=...)"; exit 1; }

expected_pages() {
  "$PYTHON" - "$1" <<'PY'
import re, sys
text = open(sys.argv[1], encoding="utf-8").read()
inline = re.search(r"^pages:\s*\[(.*?)\]", text, re.M)
if inline:
    entries = [e.strip().strip("'\"") for e in inline.group(1).split(",") if e.strip()]
else:
    block = re.search(r"^pages:\s*\n((?:[ \t]*-[ \t]*.+\n)+)", text, re.M)
    entries = [l.split("-", 1)[1].strip().strip("'\"") for l in block.group(1).splitlines()] if block else []
print(len(entries))
PY
}

fail=0
printf '%-34s %-6s %-6s %-6s %s\n' DECK CHECK BUILD VALIDATE PAGES
printf '%s\n' "--------------------------------------------------------------------------"

for dir in */; do
  dir="${dir%/}"
  [ -n "$ONLY" ] && case "$dir" in *"$ONLY"*) ;; *) continue ;; esac
  [ -f "$dir/deck.pptd" ] || continue

  mkdir -p "$dir/build"
  out="$dir/build/deck.pptx"

  if "$PPTD" check "$dir/deck.pptd" >"$dir/build/check.log" 2>&1; then
    check=PASS
  else
    check="FAIL($?)"
  fi

  if "$PPTD" build "$dir/deck.pptd" -o "$out" --force >"$dir/build/build.log" 2>&1; then
    build=PASS
  else
    build="FAIL($?)"
  fi

  pages="$(expected_pages "$dir/deck.pptd")"
  if [ "$build" = PASS ]; then
    render_flag=""
    [ "$RENDER" = 1 ] && render_flag="--render-images $dir/build/preview"
    if "$PYTHON" "$VALIDATE" "$out" --expected-pages "$pages" --check-notes $render_flag \
        >"$dir/build/validate.log" 2>&1; then
      validate=PASS
    else
      validate="FAIL($?)"
    fi
  else
    validate=SKIP
  fi

  printf '%-34s %-6s %-6s %-6s %s\n' "$dir" "$check" "$build" "$validate" "$pages"
  for stage in "$check" "$build" "$validate"; do
    case "$stage" in FAIL*) fail=1 ;; esac
  done
done

printf '%s\n' "--------------------------------------------------------------------------"
if [ "$fail" = 0 ]; then
  echo "ALL DECKS PASS (logs in <deck>/build/*.log)"
else
  echo "FAILURES PRESENT — inspect <deck>/build/{check,build,validate}.log"
fi
exit "$fail"
