# #8151 검증 명령과 실제 결과

## 실행 신원

- base: `2e087a5b4bc1515ad877e8020b30114639a59392`
- 후보 runtime: `d81b8b4410779017d50a8891caa6a83ebec994d6`
- 회귀·lint를 검증한 code/test head: `1c8294dd523fab69b303ccb4cb45bb55de003a72`
- Linux `ubuntu-ted3`, Cargo 공용 target `/home/tsjang/rhwp/target/pr-review`, 빌드 잡 8개.
- 아래 Cargo 계열 작업은 같은 target에서 순차 실행했다. 실행 로그는 커밋하지 않고
  `output/pr-review/handbook-hwpx-p144-20261022/`에 보존했다.

## Rust 사전 검증

공통 경로를 아래처럼 고정한다.

```bash
export CARGO_BUILD_JOBS=8
rhwp_review_target_dir=/home/tsjang/rhwp/target/pr-review
rhwp_review_base_sha=2e087a5b4bc1515ad877e8020b30114639a59392
```

| 검사 | 실제 결과 | 로컬 로그 |
| --- | --- | --- |
| `cargo fmt --all -- --check` | exit 0 | `format-check.log` |
| `cargo clippy --locked --target-dir "$rhwp_review_target_dir" -- -D warnings` | exit 0 | `native-clippy.log` |
| `cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir "$rhwp_review_target_dir" -- -D warnings` | exit 0 | `wasm-clippy.log` |
| `cargo build --locked --workspace --target-dir "$rhwp_review_target_dir"` | exit 0 | `workspace-build.log` |
| `cargo clippy --locked --workspace --all-targets --target-dir "$rhwp_review_target_dir" -- -D warnings` | exit 0 | `workspace-clippy.log` |
| `node scripts/rust-test-suite-manifest.mjs --prepare` | exit 0 | `manifest-prepare.log` |
| `node scripts/rust-test-suite-manifest.mjs --check --base-ref "$rhwp_review_base_sha"` | exit 0 | `manifest-check.log` |

`src/**`의 `#[cfg(test)]` 변경은 없으므로 unit tier 정책 검사는 비해당이다.
생성 suite와 manifest는 제출 변경에 포함하지 않는다.

## 회귀와 Native Skia

```bash
node scripts/run-rust-test.mjs issue_3930_hwpx_hwp_save_layout -- \
  --cargo-profile release-test --features native-skia \
  --target-dir "$rhwp_review_target_dir"
node scripts/run-rust-test.mjs issue_8151_nested_overlay_outer_margin -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target_dir" \
  --features native-skia
cargo test --locked --profile release-test --target-dir "$rhwp_review_target_dir" \
  --features native-skia --lib -- --test-threads 8
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target_dir" \
  --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- \
  --cargo-profile release-test --target-dir "$rhwp_review_target_dir" \
  --features native-skia
cargo nextest run --locked --cargo-profile release-test \
  --target-dir "$rhwp_review_target_dir" --tests --test-threads 12 --no-fail-fast
```

| 실행 | 결과 |
| --- | --- |
| 기존 #3930 suite 경로 | 2 PASS, 252 SKIP |
| 신규 #8151 suite 경로 | 1 PASS, 274 SKIP |
| Native Skia lib | rhwp 3,859 PASS / 12 IGNORE, workspace lib 추가 15 + 165 + 2 PASS |
| Native Skia placeholder | 2 PASS, 253 SKIP |
| Native Skia 직접 PDF export | 14 PASS, 251 SKIP |
| 전체 nextest | **11,359 PASS / 0 FAIL / 45 SKIP**, 실제 테스트 1,476.471초 |

신규 동일 검사 소스를 frozen base/head rlib에 직접 링크한 대조에서는 base가
바깥여백 이동량 0 대 원본 3.773333px로 FAIL(exit 101), 후보가 PASS(exit 0)였다.
이는 위 정식 suite 실행과 구분한 수정 전후 검출 증거다.

## fresh WASM과 시각 검증 재현

호스트의 locked wrapper로 최적화까지 빌드했다. 소요 시간 24분 49초,
`wasm-opt` 포함 정상 종료했다. `pkg`와 Studio public의 JS/WASM 해시 일치를 확인했고,
그 사본을 고정한 `head-pkg`로 실제 브라우저 Canvas 출력을 실행했다.

```bash
CARGO_TARGET_DIR="$rhwp_review_target_dir" \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
```

실제 producer 해시는 [producer-integrity.tsv](producer-integrity.tsv)에 있다.
원본과 독립 PDF 목록은 [inputs.tsv](inputs.tsv)에 있다. 각 입력의 compat를 적용한다.
Native는 `export-png`의 실제 Skia 출력을 사용한다.

```bash
"$rhwp_producer" export-png "$rhwp_input" --profile print \
  --compat "$rhwp_compat" --dpi 96 --scale 1.0 --json \
  -o "$rhwp_png_output" --render-tree-output "$rhwp_tree_output" \
  --font-path /usr/local/share/fonts/hancom-office-2020 \
  --font-path /usr/local/share/fonts/hwp-convert-mcp-kopub
python3 scripts/visual_sweep_pdf.py --pdf "$rhwp_print_pdf" --dpi 96 \
  --prefix "$rhwp_pdf_png_output/pdf"
python3 scripts/visual_sweep.py --silhouette-only \
  --png-pair "$rhwp_png_output" "$rhwp_pdf_png_output" \
  --key "$rhwp_key" --page-workers 2 --out "$rhwp_score_output"
```

`rhwp_producer`와 WASM package를 frozen base/후보로 각각 지정한다. WASM은 같은 설치
글꼴 디렉터리를 명시 공급해 실제 Canvas에서 출력한다.

```bash
RHWP_FONT_PATH=/usr/local/share/fonts/hancom-office-2020:/usr/local/share/fonts/hwp-convert-mcp-kopub \
  python3 scripts/visual_sweep.py --silhouette-only --compat "$rhwp_compat" \
  --dpi 96 --page-workers 1 --rhwp-bin "$rhwp_producer" \
  --wasm-pkg "$rhwp_wasm_pkg" --out "$rhwp_wasm_output" \
  --file-target "$rhwp_key" "$rhwp_input" "$rhwp_print_pdf"
```

base WASM은 page-workers 4, 후보는 1로 실행했다. 입력·글꼴·DPI·조판 옵션과
비교 지표는 같으며, 실행 병렬도 차이는 생산 신원에서 구분한다. 최초 명시 글꼴 공급이 빠진
WASM 진단 결과는 최종 제출 TSV·PNG로 사용하지 않는다.

전체 시각 검증 결과와 직접 판독은 [README](README.md),
변경 페이지와 작은 raw 하락의 원인은 [페이지 변화 분석](page-change-analysis.md)에 연결한다.
Studio 소스·편집 command·npm API 변경은 없으므로 Studio unit/E2E·Undo 계약 검사는 비해당이다.
공통 레이아웃의 실제 소비 경로는 Native Skia와 fresh WASM Canvas 출력으로 검증했다.
