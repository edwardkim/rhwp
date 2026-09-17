---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-17
---

# PR #7243 검토

**판정: 부분 개선 확인, 이슈 #7234 전체 해결로는 머지 보류.** 기존 입력의 조판 결함이 남으므로 `Closes #7234`를 그대로 승계하지 않는다. 새 scaffold 산출물 개선으로 범위를 한정하는 경우와 기존 엔진 결함 해결을 구분한다.

## 접수·체리픽

- [원 PR #7243](https://github.com/edwardkim/rhwp/pull/7243), planet6897, devel 대상 non-draft. 기존 기여자, reviewer `jangster77` 지정.
- source `d465c0a47916b973e8be99225b88c1ee59205915` → 통합 `75a48488676d79a0357ba1cae6c863ac2120b668`, `cherry-pick -x`, 충돌 없음.
- 브랜치 `codex/pr7239-7240-review-20260917`, 기준 `236a601da803b53429e9090eef652c661dd3bfe2`. #7239·#7240·#7242를 함께 포함한다.

## 발견 사항

### P1 — 원 이슈의 기존 재현 파일은 계속 넘치므로 종료 범위가 과도함

`src/scaffold/builder.rs`는 새 셀의 선언 높이를 282→1282 HU로 바꾼다. 정상적인 최소 높이 생성 개선이지만 이미 저장된 `samples/issue7216/tall_table_after.hwpx`의 셀은 여전히 282 HU다. 원 이슈는 그 파일을 열 때 행 컷과 실제 렌더 높이가 달라지는 엔진 결함을 포함한다. PR 설명도 기존 overflow baseline과 엔진 불일치가 남는다고 명시한다.

한컴이 기존 282 HU 셀을 보존하며 정상 조판한다는 작성자 근거와도 일치한다. 새 fixture로 입력을 바꾼 통과를 원 이슈의 해소로 세지 않는다. 통합 PR에서는 `Refs #7234`로 범위를 한정하고 이슈를 열어 두어야 한다. 원 이슈 전체를 종료하려면 기존 입력의 컷·측정·실제 배치를 공통 계약으로 해결하고 원본·대조군을 검증해야 한다.

### P2 — 시각 검증 비해당 주장은 부적절함

renderer 파일이 바뀌지 않아도 생성한 선언 높이가 행 분할과 최종 배치를 바꾼다. 저장소 조판 지침에 따라 Native/fresh WASM Visual Sweep을 추가한다. 제출 PDF가 종전 PDF와 같다는 사실은 rhwp 출력의 일치를 대신하지 않는다.

## 독립 검증

- 새 테스트를 builder 변경 전 코드에 이식: **0 passed / 2 failed**. 셀 선언 높이 assertion과 실제 본문 바닥 넘침(칸 바닥 1100.0, 본문 바닥 1009.1)으로 실패했다.
- builder 변경 적용 후: **2 passed**. 첫 조각의 본문 내 수용과 1쪽 행 46/2쪽 행 47 계약을 확인했다. 테스트 허용치·baseline 변경 없음.
- 기존 Git 입력·PDF를 재사용: [긴 표](../../../samples/issue7234/tall_table_cell_row_height.hwpx), [긴 표 PDF](../../../samples/issue7234/tall_table_cell_row_height-2020.pdf), [짧은 표](../../../samples/issue7234/short_table_cell_row_height.hwpx), [짧은 표 PDF](../../../samples/issue7234/short_table_cell_row_height-2020.pdf).
- Native Visual Sweep 직접 판독: 긴 표 2쪽, 짧은 표 1쪽. 긴 표는 1쪽 마지막 46행·2쪽 첫 47행으로 PDF와 일치하고 본문 바닥 넘침이 해소됐다. 표·후속 문단의 소폭 위치와 글꼴·선 굵기 차이는 남아 전체 시각 일치로 보고하지 않는다.
- fresh WASM·최종 공통 lint 결과는 아래 최종 기록에 추가한다. 원 작성자 CI 9993 passed를 통합 head 결과로 재사용하지 않는다.

## Merge 후 contributor PR comment 계획

실제 통합 merge·최종 CI가 완료된 경우 기여 감사와 적용 SHA, 새 scaffold 산출물에 한정된 개선 범위, 기존 엔진 결함 및 #7234 OPEN 유지를 한국어로 설명한다. [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 긴 표 1·2쪽과 짧은 표 1쪽의 Native/fresh WASM compare·overlay·review PNG를 merge SHA raw URL로 본문에 포함한다. UTF-8 파일과 `--body-file`로 게시 후 한국어·이미지 URL·실제 head를 검증한다. contributor fork branch를 삭제하지 않는다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

### 최종 후보 직접 확인

`layout-anomaly samples/issue7216/tall_table_after.hwpx --json`: 기존 입력 **overflowCount=1 / overBottom=91.08px**. 새 `samples/issue7234/tall_table_cell_row_height.hwpx`: **overflowCount=0**, 둘 다 2쪽이다. 따라서 `Closes #7234` 보류 사유는 코드 추정이 아니라 실제 실행 결과다. 새 입력의 Native/WASM raster는 3쪽 모두 서로 동일하고, 긴 표 46/47 행 분할과 짧은 표 1쪽을 PDF와 직접 확인했다. #7243 원 CI [35220748013](https://github.com/edwardkim/rhwp/actions/runs/35220748013)는 조회 당시 진행 중이며 그 결과를 로컬 검증으로 세지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| tall p1 | [compare](../assets/pr7243_review/native_tall_compare_001.png) · [overlay](../assets/pr7243_review/native_tall_overlay_001.png) · [review](../assets/pr7243_review/native_tall_review_001.png) | [compare](../assets/pr7243_review/wasm_tall_compare_001.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_001.png) · [review](../assets/pr7243_review/wasm_tall_review_001.png) |
| tall p2 | [compare](../assets/pr7243_review/native_tall_compare_002.png) · [overlay](../assets/pr7243_review/native_tall_overlay_002.png) · [review](../assets/pr7243_review/native_tall_review_002.png) | [compare](../assets/pr7243_review/wasm_tall_compare_002.png) · [overlay](../assets/pr7243_review/wasm_tall_overlay_002.png) · [review](../assets/pr7243_review/wasm_tall_review_002.png) |
| short p1 | [compare](../assets/pr7243_review/native_short_compare_001.png) · [overlay](../assets/pr7243_review/native_short_overlay_001.png) · [review](../assets/pr7243_review/native_short_review_001.png) | [compare](../assets/pr7243_review/wasm_short_compare_001.png) · [overlay](../assets/pr7243_review/wasm_short_overlay_001.png) · [review](../assets/pr7243_review/wasm_short_review_001.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.
