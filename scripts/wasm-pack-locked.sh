#!/usr/bin/env sh
# wasm-pack passes extra arguments only to cargo build. Its earlier cargo
# metadata call can otherwise rewrite the workspace lockfile.
set -eu

real_cargo="${CARGO:-}"
if [ -z "${real_cargo}" ]; then
  real_cargo="$(command -v cargo)"
elif [ "${real_cargo#*/}" = "${real_cargo}" ]; then
  real_cargo="$(command -v "${real_cargo}")"
fi

shim_dir="$(mktemp -d)"
cleanup() {
  rm -rf "${shim_dir}"
}
trap cleanup EXIT HUP INT TERM

cat > "${shim_dir}/cargo" <<'EOF'
#!/usr/bin/env sh
set -eu

if [ "${1:-}" = "metadata" ]; then
  for arg in "$@"; do
    if [ "${arg}" = "--locked" ]; then
      exec "${RHWP_WASM_PACK_REAL_CARGO}" "$@"
    fi
  done
  exec "${RHWP_WASM_PACK_REAL_CARGO}" "$@" --locked
fi

exec "${RHWP_WASM_PACK_REAL_CARGO}" "$@"
EOF
chmod +x "${shim_dir}/cargo"

PATH="${shim_dir}:${PATH}" \
  RHWP_WASM_PACK_REAL_CARGO="${real_cargo}" \
  wasm-pack build "$@" --locked
