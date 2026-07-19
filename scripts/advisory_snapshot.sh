#!/usr/bin/env bash
# [#2403 Phase P] advisory 3종 스냅샷 생성 — public API 표면 / WASM JSON 계약 / CLI output.
# 사용법: ./scripts/advisory_snapshot.sh <출력 dir>
# 재현 조건: 같은 커밋 + release-test 빌드된 target/release-test/rhwp 존재.
# 대조: 리팩토링 각 단계 게이트에서 재생성 후 `diff -r` — advisory (무변동 기대).
set -euo pipefail
OUT="${1:?출력 디렉터리 필요}"
BIN=target/release-test/rhwp
[[ -x "$BIN" ]] || { echo "error: $BIN 없음 — cargo build --profile release-test 선행" >&2; exit 1; }
mkdir -p "$OUT"

# 1) public Rust API 표면 — pub 선언 시그니처의 결정적 목록 (파일 경로 + 정규화 시그니처)
#    도구 무의존(grep 기반) — 같은 방법으로 재생성해 비교하는 전제의 advisory 스냅샷.
#    줄번호는 제외한다 — 무관한 필드 추가로 전 항목이 밀리는 diff 노이즈 방지 (1단계 실측).
grep -rn --include="*.rs" -E "^[[:space:]]*pub (fn|struct|enum|trait|type|const|static|mod|use) " src \
  | sed -E 's/^([^:]+):[0-9]+:/\1: /; s/[[:space:]]+/ /g; s/ \{.*$//; s/;.*$//' \
  | sort > "$OUT/api_surface.txt"

# 2) WASM JSON 계약 — wasm_api 가 노출하는 JSON 반환 계약의 대표 표본.
#    네이티브에서 같은 document_core 질의를 쓰는 CLI 명령으로 고정한다.
# 3) CLI output — 대표 명령 × 대표 샘플 (HWP5/HWPX/HWP3 각 1 이상)
SAMPLES=(samples/biz_plan.hwp samples/hwp3-sample.hwp samples/issue_2148_degenerate_cell_vpos.hwpx)
: > "$OUT/cli_output.txt"
for s in "${SAMPLES[@]}"; do
  base=$(basename "$s")
  {
    echo "===== $base : info ====="
    "$BIN" info "$s" 2>&1
    echo "===== $base : dump-pages ====="
    "$BIN" dump-pages "$s" 2>&1 | head -40
  } >> "$OUT/cli_output.txt"
  # export-render-tree 는 -o 를 디렉터리로 취급해 render_tree_NNN.json 을 만든다
  rtdir="$OUT/rt_${base}.d"
  "$BIN" export-render-tree "$s" -p 0 -o "$rtdir" >/dev/null 2>&1 || \
    echo "render-tree 실패: $base" >> "$OUT/cli_output.txt"
done
# render tree JSON 은 크므로 sha256 만 표에 남기고 원본은 삭제 (계약 = 구조 해시)
: > "$OUT/render_tree_sha256.txt"
for d in "$OUT"/rt_*.d; do
  [[ -d "$d" ]] || continue
  for f in "$d"/*.json; do
    [[ -f "$f" ]] || continue
    echo "$(sha256sum "$f" | cut -d' ' -f1)  $(basename "$d")/$(basename "$f")" >> "$OUT/render_tree_sha256.txt"
  done
  rm -rf "$d"
done

echo "advisory snapshot → $OUT"
wc -l "$OUT"/api_surface.txt "$OUT"/cli_output.txt "$OUT"/render_tree_sha256.txt
