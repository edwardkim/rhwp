# fix(renderer): 글자겹침 charSz 축소를 테두리 있는 겹침으로 한정 (#4085)

## 요약

글자겹침(CharOverlap)의 `charSz`(IR `inner_char_size`) 축소가 **테두리를 그리지 않는 겹침에도**
적용돼, 한컴 대비 60% 크기로 렌더되던 결함을 수정합니다.

`charSz` 는 OWPML 스키마상 **"테두리 내부 글자의 크기 비율. 단위 %"**
(`mydocs/manual/OWPML SCHEMA/ParaList XML schema.xml:571`)인데, 렌더 경로는 `border_type` 을 보지
않고 항상 축소를 적용했습니다. 증상 문서는 `border_type=0`(`circleType="CHAR"`, 테두리 없음)이라
축소할 "테두리 내부"가 없는데도 `charSz=-4 → 0.60` 이 걸렸습니다.

- 규칙을 `composer.rs` 의 `char_overlap_size_ratio` 하나로 단일화 — 종전에는 같은 규칙이 5곳에
  인라인으로 흩어져 있었고 그중 `skia/text_replay.rs` 만 음수 분기가 없어 **같은 문서가
  SVG/CanvasKit 60% vs PNG 100%** 로 갈렸습니다
- 게이트를 raw `border_type` 이 아니라 **실제로 테두리를 그리는지**(`effective_border`)에 걸어, PUA
  다자리 숫자가 원형 테두리로 승격되는 combined 경로의 동작은 그대로 둡니다

## 근거 — 서로 반대 방향인 두 오라클

| fixture | border_type | charSz | 한컴 실측 |
| --- | --- | --- | --- |
| `samples/hwpx/k-water-rfp.hwpx` p13 | 4 (`SHAPE_REVERSAL_RECTANGLE`) | -2 | 0.80 배 축소 (PR #1101) |
| 관세청 월간 수출입 현황 p1 | 0 (`CHAR`, 테두리 없음) | -4 | **축소 없음** (본 작업) |

음수 `charSz` → 10% step 축소 규칙은 PR #1101 에서 도입됐고, 당시 실측 fixture 는 `border_type=4`
한 건뿐이었습니다. PR 리뷰 문서도 미검증 가설로 명시하며 재검증을 권고했습니다
(`mydocs/pr/archives/pr_1101_review.md:90,164`). 본 PR 이 그 재검증입니다.

한컴 PDF content stream — 마커와 뒤따르는 본문이 같은 글자 크기, 같은 baseline:

```
/F5 101 Tf  2 Tr 2.02 w   1 0 0 -1  335 1613 Tm  [<0003>]TJ   ← 글자겹침 마커
/F6 101 Tf  2 Tr 2.02 w   1 0 0 -1  436 1613 Tm  [<0001>]TJ   ← 뒤따르는 본문
```

마커 폭 335→436 = 101 = 정확히 1em.

## 검증

`origin/devel` `d76d4e98b` 위에서 재실행한 결과입니다.

- `cargo test --profile release-test --lib char_overlap` — 10 passed / 0 failed
- `cargo clippy --all-targets -- -D warnings` — exit 0, warning 0
- `cargo fmt --check` — 실제 포맷 diff 0
- `cargo test --profile release-test --tests --no-fail-fast` — **465 스위트 ok / 3 FAILED**
- Native Skia — `skia --lib` 58 / `issue_2225_missing_picture_placeholder` 2 /
  `render_p37_direct_pdf_export` 4 passed
- WASM 빌드 (Docker) — 컴파일·wasm-bindgen·wasm-opt·packaging 전 단계 통과

실패 3건(`every_normal_sample_is_clean`, `ir_field_sweep_does_not_regress`,
`negative_corpus_sweep_is_clean_across_all_three_detectors`)은 모두 `samples/` 에 **untracked 로
놓인 로컬 파일**이 원인이며 CI 에는 존재하지 않습니다. 같은 테스트 바이너리로 파일 존재 여부만
바꿔 재실행해 확인했습니다 — 격리하면 세 스위트 모두 통과(14/2/6 passed). 상세는 보고서 5절.

### 시각 증적

**관세청 p1 (수정 대상)**

| | 수정 전 | 수정 후 | 한컴 |
| --- | --- | --- | --- |
| 마커 `font-size` | `13.60` | `22.67` | 본문과 동일 |
| 비율 | 0.60 | **1.00** | 1.00 |

render tree — 좌여백 20mm(75.6px)에서 정확히 1em, 뒤따르는 `’` 위치 한컴 109.7px vs rhwp 109.3px:

```json
{"type":"TextRun","text":"󰊱","bbox":{"x":75.6,"y":344.3,"w":22.7,"h":22.7}}
```

**k-water-rfp p13 (회귀 금지)** — `font-size = 18.13` = 22.66 × 0.80 으로 PR #1101 기록값과 동일,
반전 사각형 3개 모두 유지.

## 문서

- 계획서 `mydocs/plans/task_m100_4085.md`
- 단계 기록 `mydocs/working/task_m100_4085_stage1.md`
- 최종 보고서 `mydocs/report/task_m100_4085_report.md`

## 파생 이슈

| 이슈 | 내용 |
| --- | --- |
| #4086 | 폰트 폴백 체인 이름 불일치 + PUA 글리프 소재 오인 |
| #4088 | 한국어 `instruction_override` 규칙이 절 경계를 넘어 오탐 |
| #4089 | Windows 네이티브 `wasm-pack` 이 `wasm-opt` 무한 재실행 |

Closes #4085
