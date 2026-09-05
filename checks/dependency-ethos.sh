# Every dependency's ethos declaration reads and generates with the built tool.
set -eu
out_dir="$TMPDIR/generated"
for declaration in $declarations; do
  reply="$("$package/bin/ethos-zero" "Generate.{ $declaration $out_dir }")"
  case "$reply" in
    Generated.*) echo "$reply" ;;
    *) echo "$declaration: $reply" >&2; exit 1 ;;
  esac
done
touch "$out"
