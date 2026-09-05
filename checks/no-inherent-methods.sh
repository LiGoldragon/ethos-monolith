# Behavior is homed in kinds: no inherent impl block in src.
set -eu
if grep -R -n -E '^[[:space:]]*impl(<[^>]*>)?[[:space:]]' "$src/src" | grep -v ' for ' | grep -E '\{[[:space:]]*$'; then
  echo "production Rust must home behavior in traits" >&2
  exit 1
fi
touch "$out"
