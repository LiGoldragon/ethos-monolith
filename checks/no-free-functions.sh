# Every method call lives under a trait: no module-level free function in src,
# save the entry point Rust forces on the binary.
set -eu
if grep -R -n -E '^(pub(\([^)]*\))? )?fn ' "$src/src" | grep -v -E 'src/main\.rs:[0-9]+:fn main\(\)'; then
  echo "production Rust must not use module-level free functions" >&2
  exit 1
fi
touch "$out"
