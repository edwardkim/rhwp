---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-17
---

# PR #7242 검토

**머지 보류 — 공개 입력의 PDF 시각 증거 미충족.** #7239·#7240 통합 후보에 누적 적용했다.

## 접수·적용 범위

- 원 PR: [#7242](https://github.com/edwardkim/rhwp/pull/7242), LJYeon12. 기존 #7107·#7109·#7165·#7168·#7200 기여 이력이 있어 첫 기여자 절차는 비해당이다.
- source head `03ba57cb804610e899a3e6e1ce2cb3cfcb21df6e`, devel 대상 non-draft, reviewer `jangster77` 지정.
- `codex/pr7239-7240-review-20260917`, 기준 `236a601da803b53429e9090eef652c661dd3bfe2`.
- source 7개 커밋을 `cherry-pick -x`로 적용. `bf8070c27` → `fc668664a` → `0e3513b66` → `bb45be16b` → `1e3aed178` → `403c4ef7f` → `baa29e96e`. 텍스트 충돌 없음.
- 최종 source 범위: `src/renderer/layout.rs`, `tests/cases/stored_table_text_tail.rs`, `samples/stored-table-text-tail/README.md`, `native-8-0.hwpx`.
- 원 작성자 CI/사적 문서 검증은 제출자의 증거다. 이 통합 head의 검증과 구분한다. 비공개 원문·PDF를 공개하거나 확보했다고 간주하지 않는다.

## 구현·증거 대조

`layout_column_content`의 PageItem 순회 → 표 배치의 실제 `y_offset` → `PartialParagraph`의 저장 host 줄 끝과 다음 줄 시작의 gap → 실제 문단 layout → text/table/shape 점유 합집합 → 마지막 소유 item에서 문단 뒤 간격을 적용한다.

- 저장 줄이 같은 줄이면 원 앵커를 유지한다. 표 소속 줄 바로 다음 저장 줄만 흐름 끝과 gap을 사용한다.
- 문단 뒤 간격은 visible text tail의 반환값에서 제거해 deferred map으로 넘기고 마지막 item에서 한 번 적용한다.
- 합성 입력은 수동 저장 메타데이터임을 명시한다. 계약 검사는 2/8 셀 문단, 0/600 HU gap, HWP5-origin/pure 계보, 0/600/1200 HU 뒤 간격과 shape 공존을 검사한다.
- 기존 실제 입력 [156513948.hwpx](../../../samples/issue6044/156513948.hwpx)과 [한컴 PDF](../../../pdf/pr6940-156513948-source-2020.pdf)의 영향 20쪽을 Native/fresh WASM compare·overlay·review로 직접 판독한다.
- 제출 합성 입력은 별도 한컴 2020 변환: job `13f12b15-2c8a-403c-85cd-7df24135f0bd`, start→status succeeded→download, engine `2020`, Hancom `11.0.0.9136`, 1쪽, input_preprocess none. [기준 PDF](../../../pdf/pr7242/native-8-0-2020.pdf) SHA256 `90d6e84f4c24a5d0e79cde09cfd5d9addf1d8e92da322a13cb03f0dab9a2c3c3`.

## 초기 검증 기록

아래는 재빌드 전의 기록이며 최종 결과는 후단에 기록했다. 초기 Native focused 1 pass/2 fail은 오래된 build 산출물 사용 가능성을 발견해 판정을 유보했다. 소스 복사 시 과거 mtime이 보존되어 Cargo가 이미 빌드한 라이브러리로 판단한 경로를 확인했다. 소스 mtime 갱신 후 재컴파일과 새 binary 식별로 다시 검증하며 초기 결과를 최종 코드의 회귀 증거로 세지 않는다.

## Merge 후 contributor PR comment 계획

통합 PR의 최종 head CI와 merge가 끝나면 실제 merge SHA·CI URL, 원 기여와 메인터너 보정, 검증 범위·남은 차이를 한국어로 설명하고 감사한다. [Visual Sweep 정본](../../../mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다. 최종 Native/fresh WASM compare·standalone overlay·review PNG를 merge SHA raw URL로 본문에 표시한다. 비공개 검증 자료를 공개 자료로 바꾸어 쓰지 않는다. UTF-8 본문 파일과 `--body-file`로 게시하고 원문과 이미지 URL을 다시 확인한다. 관련 공개 issue는 없어 임의 종료하지 않는다.

## 최종 통합 후보와 시각 증거

- code head: `75a48488676d79a0357ba1cae6c863ac2120b668`, base `236a601da803b53429e9090eef652c661dd3bfe2`.
- Native SHA256: `46d87aedbeca44eb31a31ddccd2c6b7e8deebd4008bc2bbe98598af67bdc8ca9`.
- fresh WASM SHA256: `6488efc93f6635ef0fe193e09d7982cfd48231d99679007cd8d3116f61c2dbc5`, JS `a7353a7603b7e07db2d33ff93fff6b213ea79e01da91c190cbb607e752c6b5a7`.
- Mac arm64/Rust 1.93.1, 별도 verify checkout의 source/test를 위 head와 바이트 대조했다. review target은 `target/pr7239-7240-review-20260917`. Docker 표준 경로 대신 host `scripts/wasm-pack-locked.sh --target web --out-dir <scratch>/wasm-final --no-opt`를 실행했다. wasm-opt 통과로 주장하지 않는다.
- `venv/bin/python scripts/visual_sweep.py --file-target <key> <입력> <PDF> --rhwp-bin <scratch>/rhwp-current --pages <아래 쪽> --dpi 96 --out <scratch>/native-sweep`, WASM은 `--wasm-pkg <scratch>/wasm-final` 추가. 최종 head로 재캡처한 compare·standalone overlay·review를 직접 판독했다.
- 전체 nextest·Native Skia 3종은 이번 후보에서 미실행이다. #7242 시각 보류 사유가 있어 지침의 작은 경계/영향 페이지 확인을 먼저 완료했고, 대규모 회귀를 통과 근거로 대신하지 않는다. 최종 승인·PR 제출 준비 완료가 아니다.

## 최종 판정 근거 — 시각 증거 미충족

**머지 보류.** 새 Native 빌드에서 focused 3개는 통과했고 초기 두 실패는 stale library 재사용이었다. 그러나 계약 검사만으로 시각 승인을 하지 않는다.

- 합성 입력의 표 bbox는 Native/WASM 모두 `(48.0,139.4,384.0,170.7)`. Footer는 base `(399.0,232.7)`에서 수정 후 `(399.0,310.0)`으로 이동해 표 내부 겹침은 해소했다. 한컴 PDF의 Footer text bbox는 약 `(115.04,343.47)`px로, 표 높이·Footer 수평/수직 위치가 명백히 다르다. 이 입력은 수동 저장 정보가 있는 합성이며 그 사실만으로 불일치를 승인하지 않는다.
- 실물 `156513948.hwpx`는 32쪽 중 31쪽 SVG가 base와 동일, 20쪽만 변했다. 주석 3줄이 +2.0267px 이동하고 나머지 text/표 paint는 유지된다. Native/fresh WASM/PDF를 직접 비교했으며 글꼴과 기존 표 테두리·아래 주석 오차가 남는다. 작성자가 보고한 상대 간격 개선을 원 비공개 사례의 PDF 정합성 검증으로 대신하지 않는다.
- `fidelity_compare.py 0 31 --source samples/issue6044/156513948.hwpx --reference-pdf pdf/pr6940-156513948-source-2020.pdf --text-only --export-all-svg --layout-ledger`로 전수 후보도 수집했다. 자동 지표는 승인 근거가 아니다. 불필요한 JSON/TSV/log는 커밋하지 않는다.
- 해제 조건: 합성 저장 정보의 유효성을 독립 기준과 확인하고, 정상 한컴 생성/재저장 대조군 또는 공개 가능한 원 실패 입력에서 같은 경로의 표 하단·후속 글줄 간격·문단 종료를 직접 입증해야 한다. 큰 차이를 임의 좌표 보정/출력 숨김으로 제거하지 않는다.
- 원 head upstream CI [35218364377](https://github.com/edwardkim/rhwp/actions/runs/35218364377)는 성공(31 success/3 skipped)이지만 이 통합 후보의 미충족 시각 증거를 대체하지 않는다.

### 직접 판독한 PNG

| 입력·쪽 | Native | fresh WASM |
| --- | --- | --- |
| tail p1 | [compare](../assets/pr7242_review/native_tail_compare_001.png) · [overlay](../assets/pr7242_review/native_tail_overlay_001.png) · [review](../assets/pr7242_review/native_tail_review_001.png) | [compare](../assets/pr7242_review/wasm_tail_compare_001.png) · [overlay](../assets/pr7242_review/wasm_tail_overlay_001.png) · [review](../assets/pr7242_review/wasm_tail_review_001.png) |
| real_tail p20 | [compare](../assets/pr7242_review/native_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/native_real_tail_overlay_020.png) · [review](../assets/pr7242_review/native_real_tail_review_020.png) | [compare](../assets/pr7242_review/wasm_real_tail_compare_020.png) · [overlay](../assets/pr7242_review/wasm_real_tail_overlay_020.png) · [review](../assets/pr7242_review/wasm_real_tail_review_020.png) |

Merge 후 코멘트에는 위 **모든 영향 페이지**의 compare·overlay·review 링크를 실제 merge SHA의 raw URL로 치환한다. 대표 review만 넣고 standalone overlay를 빠뜨리지 않는다. 지금은 remote push/comment/merge를 수행하지 않았다.

### 최종 head 공통 검증 결과

- fmt check, Native Clippy, WASM32 lib Clippy, workspace build, workspace all-target Clippy(`-D warnings`), suite manifest base 비교: **모두 통과**.
- 최종 head focused 재실행: WMF fuzz **2**, golden **1**, column core **7**, column CLI **4**, 기존 page-break **11**, stored tail **3**, scaffold height **2** — **30 passed / 0 failed**. 각 원본에 `node scripts/run-rust-test.mjs <module>`를 test profile로 실행했다.
- Native/fresh WASM **각 9쪽** compare·standalone overlay·review, 총 18쪽 직접 확인. PNG 54개와 #7242 base 대조 PNG 6개를 개별 PR asset 경로에 보존했다. 총 60개이며 원 입력과 기준 PDF를 연결했다.
- test source·수치 baseline·허용치를 통과 목적으로 완화하지 않았다. source-side unit test 변경은 없어 unit-tier 비교는 비해당이다.
- 문서별 metadata 및 로컬 링크 검사, `git diff --check` 통과. 불필요한 log/JSON/TSV는 Git에 추가하지 않는다.

- 새 sample 3개(`stored-table-text-tail/native-8-0`, `issue7234/short_table_cell_row_height`, `issue7234/tall_table_cell_row_height`)의 hidden-text/injection/unicode 검사: **1 passed**. `RHWP_SECURITY_SWEEP_SAMPLES_JSON`으로 실제 대상을 지정해 실행했다.
