#!/usr/bin/env sh
# wasm-pack passes extra arguments only to cargo build. Its earlier cargo
# metadata call can otherwise rewrite the workspace lockfile.
set -eu

# Studio의 Vite 개발 서버는 root `pkg/`를 alias로 읽고, public 경로를 직접 쓰는
# host/standalone 경로도 있다. 기본 web package를 성공적으로 만든 뒤 두 경로를
# 같은 glue·WASM으로 맞춘다. 다른 output directory는 검증·배포용일 수 있으므로 건드리지
# 않는다.
out_dir="pkg"
next_option=""
for arg in "$@"; do
  if [ "${next_option}" = "out_dir" ]; then
    out_dir="${arg}"
    next_option=""
    continue
  fi
  case "${arg}" in
    --out-dir)
      next_option="out_dir"
      ;;
    --out-dir=*)
      out_dir="${arg#--out-dir=}"
      ;;
  esac
done

# Opt-in WASM release experiment (#7473). Keep wasm-pack's packaging/wasm-opt
# pipeline and the default build unchanged. The mixed rlib+cdylib baseline does
# not run cross-crate LTO; cdylib alone would otherwise enable fat LTO.
case "${RHWP_WASM_CDYLIB_ONLY:-0}" in
  0|1) ;;
  *) echo "RHWP_WASM_CDYLIB_ONLY must be 0 or 1" >&2; exit 2 ;;
esac

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

if [ "${RHWP_WASM_CDYLIB_ONLY:-0}" = "1" ] && [ "${1:-}" = "build" ]; then
  shift
  release=0
  library=0
  wasm=0
  if [ "${CARGO_BUILD_TARGET:-}" = "wasm32-unknown-unknown" ]; then wasm=1; fi
  previous=""
  for arg in "$@"; do
    case "${arg}" in
      --release) release=1 ;;
      --lib) library=1 ;;
      --target=wasm32-unknown-unknown) wasm=1 ;;
      --target=*) wasm=0 ;;
      --profile|--profile=*|--config|--config=*|--package|--package=*|-p|-p?*|--workspace|--)
        echo "cdylib-only requires the standard release invocation without profile/config overrides" >&2
        exit 2 ;;
    esac
    if [ "${previous}" = "--target" ]; then
      wasm=0
      if [ "${arg}" = "wasm32-unknown-unknown" ]; then wasm=1; fi
    fi
    previous="${arg}"
  done
  if [ "${release}${library}${wasm}" != "111" ]; then
    echo "cdylib-only requires --release --lib --target wasm32-unknown-unknown" >&2
    exit 2
  fi
  exec "${RHWP_WASM_PACK_REAL_CARGO}" rustc "$@" \
    --package rhwp --crate-type cdylib --config profile.release.lto=false
fi

exec "${RHWP_WASM_PACK_REAL_CARGO}" "$@"
EOF
chmod +x "${shim_dir}/cargo"

PATH="${shim_dir}:${PATH}" \
  RHWP_WASM_PACK_REAL_CARGO="${real_cargo}" \
  wasm-pack build "$@" --locked

if [ "${out_dir}" = "pkg" ] || [ "${out_dir}" = "./pkg" ]; then
  for artifact in rhwp.js rhwp_bg.wasm; do
    if [ ! -f "${out_dir}/${artifact}" ]; then
      echo "wasm-pack output missing: ${out_dir}/${artifact}" >&2
      exit 1
    fi
    cp "${out_dir}/${artifact}" "rhwp-studio/public/${artifact}"
  done
fi
