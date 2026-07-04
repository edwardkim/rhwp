# PR #1489 리뷰 계획서 - RowBreak table continuation 보정

- PR: https://github.com/edwardkim/rhwp/pull/1489
- 제목: `fix: prevent tiny rowbreak continuations`
- 작성일: 2026-06-26
- 컨트리뷰터: `oleg-sung` (Oleg Sungyrovsky)
- 관련 이슈: #1488 `[HWPX] Rowbreak table pagination emits extra continuation pages and clips/overlaps content`
- base/head: `edwardkim/rhwp:devel` <- `oleg-sung/rhwp:fix/rowbreak-chart-overlap`
- 처리 경로: collaborator-mediated 외부 PR 경로
- 작성 시점 참고 head: `14b9f7443fb54bf767d242133a8f35b2997dba48`
- 작성 시점 참고 상태: `MERGEABLE` / `CLEAN`, draft 아님, `maintainer_can_modify=true`
- 규모: 6 files, +1571 / -134

`mergeable`, `head SHA`, `CI 상태`는 변하는 값이므로 이 문서는 작성 시점 참고값으로만 기록한다.
최종 merge 판단은 최신 PR head 기준 GitHub Actions 통과, code review 완료, 작업지시자 시각 검증 및 승인 후 진행한다.

## 1. 초기 의도

이슈 #1488의 원래 문제는 `samples/rowbreak-problem-pages.hwpx`가 clean `devel`에서 한컴 기준보다 많은
페이지를 만들고, RowBreak/partial table continuation 주변에서 내용 겹침·잘림·빈 continuation 페이지를
발생시키는 것이다.

이슈 본문 기준 핵심 증상:

- clean `devel` native-skia PNG export가 24페이지를 산출.
- 2쪽: 도표 제목/라벨과 도표 내용 겹침.
- 7쪽: RowBreak 표 셀 텍스트가 행/열 경계를 넘어 겹침.
- 10쪽: 하단 표 clipping.
- 12쪽: 파란 배경 callout/textbox 내부 텍스트 clipping.
- 16쪽: continuation 상단 일부만 보이고 대부분 빈 페이지.
- 17~22쪽: 원본에 없는 거의 빈 continuation 페이지.
- 23쪽: 하단 텍스트/표 clipping.

PR의 초기 수정 방향은 빈 텍스트 + control 없는 표 문단을 spacer-only unit으로 취급하고, hard break가
빈 spacer 조각만 독립 continuation으로 만들지 못하게 하는 것이다. 이후 RowBreak split이 보이는 콘텐츠를
다음 페이지로 과도하게 밀거나, 반대로 footer/body 영역 밖으로 그리지 않도록 여러 보정이 추가되었다.

## 2. 추가 요청과 반영 여부

2026-06-23 collaborator 코멘트에서 기준 파일 2개가 추가되었다.

- `pdf/rowbreak-problem-pages-2024.pdf`
- `samples/rowbreak-problem-pages.hwp`

이 코멘트의 추가 요청은 다음 네 가지였다.

| 요청 | 현재 PR의 반영 후보 | 리뷰 확인 방법 |
|---|---|---|
| HWPX/HWP 모두 Hancom 2024 PDF와 같은 18페이지 | `tests/issue_rowbreak_chart_overlap.rs::rowbreak_final_pages_match_hancom_pdf_page_count` | 테스트 통과 + 실제 PNG/PDF 시각 확인 |
| 7/8쪽 큰 RowBreak 표 경계 정합 | page 7 host text, article 26, page 8 continuation 테스트 | 테스트가 텍스트 존재/비겹침만 보므로 시각 검증 필수 |
| 17/18쪽 마지막 RowBreak continuation의 extra page 제거 | page 17 database table tail, page 18 security section, no table pi=28 ci=0 테스트 | 테스트 + 시각 검증 필수 |
| page count 및 페이지 경계 회귀 테스트 추가 | `tests/issue_rowbreak_chart_overlap.rs` 11개 테스트 | 테스트 품질 리뷰: 너무 샘플/텍스트 취약하지 않은지 확인 |

컨트리뷰터는 2026-06-24 응답에서 HWPX/HWP가 모두 18페이지가 되었고, large RowBreak row의 visible tail,
final 1x1 continuation spacer, non-inline picture/shape control flow를 보정했다고 설명했다. 현재 review는
이 설명이 코드와 테스트로 실제 보장되는지 확인하는 단계다.

## 3. 변경 범위

| 파일 | 주요 변경 |
|---|---|
| `src/renderer/layout/table_layout.rs` | `CellUnit`에 spacer/mixed nested/vpos gap 메타 추가, RowBreak cut/rewind/visible tail/mixed nested split 계산 보정 |
| `src/renderer/layout/table_partial.rs` | partial table 렌더에서 mixed nested split, non-inline control visibility, flow height 반영 |
| `src/renderer/typeset.rs` | RowBreak TopAndBottom block의 para-relative y 보정, table flow ordering, partial table available height 보정 |
| `src/renderer/layout/picture_footnote.rs` | caption height 계산에 line segment height 상한 반영 |
| `src/renderer/layout/shape_layout.rs` | inline textbox shape bbox를 실제 content bottom까지 확장 |
| `tests/issue_rowbreak_chart_overlap.rs` | `rowbreak-problem-pages` 전용 회귀 테스트 11개 추가 |

## 4. 이미 확인된 상태

### 4.1 GitHub Actions

작성 시점 최신 head 기준:

| 체크 | 결과 |
|---|---|
| CI / Build & Test | success |
| CI / WASM Build | skipped |
| CodeQL / Analyze (javascript-typescript) | success |
| CodeQL / Analyze (python) | success |
| CodeQL / Analyze (rust) | success |
| CodeQL | success |
| Render Diff / Canvas visual diff | success |

### 4.2 conflict 해소

최신 `upstream/devel` 병합 시 유일한 content conflict는 `src/renderer/layout/table_layout.rs`였다.
해소 방향은 다음과 같다.

- 최신 `devel`의 `allow_para_top_bleed` 인자는 보존.
- PR의 RowBreak 좌표 모델을 보존하기 위해 table-level `table_y`에서는 `split_y_offset`를 빼지 않음.
- `split_y_offset`는 `layout_table_cells(..., split_y_offset, ...)` / `content_cell_y` 경로로 적용.

push 전 로컬 확인:

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_rowbreak_chart_overlap` | 11 passed |
| `cargo test --test svg_snapshot` | 8 passed |

## 5. 코드 리뷰 계획

### 5.1 PR 의도와 범위 확인

1. #1488의 목표가 "RowBreak/partial table pagination 정합"인지, PR이 unrelated 렌더 정책을 넓히지 않았는지 확인한다.
2. `rowbreak-problem-pages` 샘플 전용 조건이 코드에 숨어 있지 않은지 확인한다.
3. HWPX와 HWP 모두 같은 `Document` IR/렌더 경로에서 안정적으로 18페이지가 되는지 확인한다.

### 5.2 `table_layout.rs` 중심 검토

1. `CellUnit`의 `empty_spacer`, `vpos_gap_before`, `mixed_nested_*` 필드가 모든 생성 경로에서 일관되게 초기화되는지 확인한다.
2. 빈 문단 spacer 취급이 실제로 `text.trim().is_empty() && controls.is_empty()`에 제한되는지 확인한다.
3. `collapse_empty_rowbreak_spacer`가 1x1 RowBreak continuation에만 작동하며, 일반 빈 줄/여백/서식 문단을 삭제하지 않는지 확인한다.
4. `relaxed_hard_break`와 visible tail tolerance가 RowBreak 표에만 적용되고, `treat_as_char` 표나 일반 표 split에는 영향이 없는지 확인한다.
5. `rewind_rowbreak_orphan_before_hard_break`와 `rewind_rowbreak_tail_before_pending_hard_break`가 progress 보장과 무한 루프 방지를 깨지 않는지 확인한다.
6. `mixed_nested_split_from_cut`의 `visible_height`와 `flow_height` 분리가 partial 렌더와 pagination advance 양쪽에서 같은 의미로 쓰이는지 확인한다.
7. `cell_non_inline_control_flow_height`가 Square/Tight/Through non-inline control을 셀 flow에 포함하는 조건이 과도하지 않은지 확인한다.

### 5.3 `table_partial.rs` 렌더 경로 검토

1. cut range 밖 문단 skip 조건이 `mixed_nested_split` 또는 visible non-inline control이 있을 때 필요한 내용을 버리지 않는지 확인한다.
2. `NestedTableSplit.flow_height`가 visible bbox와 flow advance를 분리하는 의도대로 partial table 높이와 후속 `para_y`에 반영되는지 확인한다.
3. mixed nested split이 없는 기존 nested table/rowspan split 회귀를 만들지 않는지 확인한다.

### 5.4 `typeset.rs` 페이지네이터 검토

1. RowBreak TopAndBottom block에서 `first_seg.vertical_pos`를 `current_height`로 당기는 조건이 너무 넓지 않은지 확인한다.
2. `table_flow_tiebreak`가 non-TAC/TAC table 배치 순서를 바꾸면서 기존 float table 순서 정합을 깨지 않는지 확인한다.
3. `table_available = available - pagination_tolerance_px` 변경이 footer/body clipping 방지에는 필요하지만, 기존 borderline fit 문서를 과도하게 다음 페이지로 넘기지 않는지 확인한다.

### 5.5 caption/textbox 보조 변경 검토

1. `picture_footnote.rs`의 caption height 계산이 `line_seg_height.max(composed_height)`로 바뀌며 기존 caption 간격을 과도하게 키우지 않는지 확인한다.
2. `shape_layout.rs`의 inline textbox bbox 확장이 `parent_treat_as_char`일 때만 작동하는지 확인한다.
3. textbox bbox 확장이 hit-test, clipping, parent table height 계산에 부작용을 만들지 않는지 확인한다.

### 5.6 테스트 품질 검토

1. `tests/issue_rowbreak_chart_overlap.rs`가 #1488의 추가 요청 4개를 실제로 고정하는지 확인한다.
2. 텍스트 문자열 기반 assertion이 문서 내용 변경에 과도하게 취약하지 않은지 확인한다.
3. page index와 `pi/ci` hard-code가 이 샘플 회귀 가드로는 타당한지 확인한다.
4. 기존 `svg_snapshot` / `visual_roundtrip_baseline`과 겹치지 않는 독립 가치를 가지는지 확인한다.

## 6. 추가 로컬 검증 계획

최종 review 전 다음을 순서대로 확인한다.

```bash
cargo fmt --check
cargo test --test issue_rowbreak_chart_overlap
cargo test --test svg_snapshot
cargo test --test issue_1486
cargo test --test issue_1510
cargo test --release --lib
cargo clippy --all-targets -- -D warnings
```

`issue_1486` 또는 `issue_1510` 테스트 파일명이 실제 저장소와 다르면, 해당 이슈 관련 테스트명/모듈명을
`rg "1486|1510" tests src`로 확인한 뒤 targeted command를 조정한다.

## 7. 작업지시자 시각 검증 요청

자동 테스트는 bbox와 텍스트 존재를 확인하지만, 한컴 PDF와의 페이지 경계/시각 밀도는 사람이 봐야 한다.
다음 항목은 작업지시자 시각 검증이 필요하다.

1. **전체 페이지 수**
   - `samples/rowbreak-problem-pages.hwpx`: 18페이지
   - `samples/rowbreak-problem-pages.hwp`: 18페이지
   - 기준: `pdf/rowbreak-problem-pages-2024.pdf` 18페이지

2. **2쪽**
   - 도표 제목 `<민간 SaaS 연계공통기반 운영체계>`와 도표/라벨이 겹치지 않는지.
   - 도표가 한컴 PDF 대비 지나치게 아래로 밀리지 않았는지.

3. **7~8쪽 큰 RowBreak 표 경계**
   - 7쪽에 Hancom PDF처럼 표 내용이 충분히 남는지.
   - 8쪽이 너무 이른 continuation으로 시작하지 않는지.
   - `제26조` 시작 위치와 전후 행 경계가 한컴 PDF와 크게 어긋나지 않는지.

4. **10~13쪽 table/callout clipping**
   - 10/11쪽 하단 표가 body/footer 경계에서 잘리지 않는지.
   - 12/13쪽 파란 배경 callout/textbox 내부 텍스트가 bbox 안에 들어오는지.
   - `shape_layout.rs`의 inline textbox bbox 확장으로 도형 높이가 과도하게 커지지 않았는지.

5. **16~18쪽 마지막 continuation**
   - 16/17쪽 continuation 내용이 잘리지 않는지.
   - 17쪽에 최종 database table tail이 남고, 18쪽이 `보안 분야` 섹션으로 시작하는지.
   - 18쪽에 `pi=28 ci=0`의 빈/작은 continuation이 다시 나타나지 않는지.

권장 산출물:

```bash
cargo run --bin rhwp --features native-skia -- export-png samples/rowbreak-problem-pages.hwpx -o output/pr1489/hwpx --scale 1.0
cargo run --bin rhwp --features native-skia -- export-png samples/rowbreak-problem-pages.hwp -o output/pr1489/hwp --scale 1.0
```

위 PNG를 `pdf/rowbreak-problem-pages-2024.pdf`와 2, 7, 8, 10, 12, 13, 16, 17, 18쪽 중심으로 비교한다.

## 8. 현재 판단 보류 사유

현재 CI와 targeted 테스트는 통과했지만, PR은 렌더러 pagination 핵심 경로를 크게 바꾸고 있다.
특히 RowBreak split, mixed nested table flow, non-inline control flow, textbox bbox 확장이 동시에 들어가므로
자동 테스트만으로 merge 판단을 확정하지 않는다.

다음 단계는 코드 리뷰에서 차단 이슈 여부를 확인하고, 작업지시자 시각 검증 결과를 받은 뒤
수용/수정요청/보류 판단을 정리하는 것이다.

## 9. 리뷰 중 발견된 회귀와 collaborator 후속 보정

작업지시자 시각 검증에서 초기 PR head 및 conflict 해소 후 head에 대해 다음 불일치가 확인되었다.
이 항목들은 원 PR의 "tiny RowBreak continuation 방지" 의도와 같은 문제 영역에 속하므로, 별도 문서 PR을
만들지 않고 collaborator 후속 보정 commit으로 PR head에 반영한다.

| 영역 | 증상 | 보정 방향 |
|---|---|---|
| HWP 7~8쪽 | RowBreak 표 continuation에서 중첩 표 꼬리 라인이 잘리거나, 점선 박스가 다음 문단까지 포함 | mixed nested split의 visible height와 flow height를 분리하고 continuation offset을 조정 |
| HWP 12쪽 | 본문 일부가 body/footer 경계에서 잘림 | RowBreak continuation available height 및 partial flow 계산 보정 |
| HWP/HWPX 17쪽 | 상단 공백 때문에 `별도 테이블(table)로 구성·설계` 라인이 사라짐 | 1x1 RowBreak continuation의 vpos 기준을 원점 보정하고, 실제 보이는 텍스트 라인 보존 |
| HWPX 1쪽 | 첫 본문이 한컴 PDF보다 과도하게 아래로 밀림 | HWPX page path에서 stale forward vpos를 제한적으로 접음 |
| HWPX 4~6쪽 | `연계 개발 및 테스트` 표 continuation 및 후속 노트가 한컴 PDF보다 앞쪽 페이지로 당겨짐 | HWPX RowBreak 내부 top padding/vpos gap 보정 |
| HWPX 10쪽 | `서비스 보안인증제도(CSAP)` 표가 제목 아래가 아니라 페이지 하단에 위치 | HWPX stale forward vpos 보정 대상에 TAC table host를 포함 |
| HWP/HWPX 13쪽 | `<공공데이터 제공·관리 실무매뉴얼...>` 발췌 박스 내부 상하 간격 불일치와 하단 박스 공백 | 1x1 RowBreak excerpt table의 선형 empty spacer vpos를 보존 |

후속 보정은 원 PR의 핵심 동작을 넓게 뒤집지 않고, 다음 제한 조건을 둔다.

- 1x1 RowBreak table에서 `vertical_offset == 0`이고 선형 `LINE_SEG.vpos`가 존재할 때만 empty spacer vpos를 보존한다.
- HWPX stale forward vpos 접기는 HWPX source page path에 한정한다.
- mixed nested table continuation은 visible bbox와 pagination flow advance를 분리해, 박스가 다음 문단을 감싸지 않도록 한다.
- non-inline shape/control 보정은 continuation에서 실제 visible tail을 보존하는 경우로 제한한다.

## 10. 후속 보정 검증 기록

push 전 로컬에서 다음을 확인했다.

| 명령 | 결과 |
|---|---|
| `cargo fmt --check` | 통과 |
| `cargo test --test issue_rowbreak_chart_overlap -- --nocapture` | 20 passed |
| `cargo test --test issue_716 -- --nocapture` | 통과 |
| `cargo test --test svg_snapshot -- --nocapture` | 8 passed |
| `git diff --check` | 통과 |
| `cargo run --bin rhwp --features native-skia -- dump-pages samples/rowbreak-problem-pages.hwp -p 12` | 18페이지 유지 확인 |
| `cargo run --bin rhwp --features native-skia -- dump-pages samples/rowbreak-problem-pages.hwpx -p 12` | 18페이지 유지 확인 |
| `wasm-pack build --target web` | 통과 |

작업지시자 수동 시각 검증으로 다음 항목도 확인했다.

- HWP 12쪽 잘림 회귀가 해소됨.
- HWP 7~8쪽 continuation 내용 잘림이 해소됨.
- HWPX 1쪽, 4~6쪽, 7~8쪽, 10쪽 불일치를 보정 대상으로 반영함.
- HWP/HWPX 17쪽의 누락 라인 보존을 보정 대상으로 반영함.
- HWP/HWPX 13쪽 발췌 박스 spacing은 회귀 위험이 낮은 제한 조건으로 보정함.

최종 merge 판단은 collaborator 후속 보정 commit과 이 리뷰 문서 commit을 PR head에 push한 뒤,
GitHub Actions 최신 결과와 GitHub review 상태를 기준으로 다시 확정한다.
