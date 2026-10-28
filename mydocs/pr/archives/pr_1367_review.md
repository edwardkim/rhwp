# PR #1367 검토 — merge_from 컨트롤 병합 보강

- PR: https://github.com/edwardkim/rhwp/pull/1367
- 제목: fix(model): merge_from 컨트롤 병합 보강 — 글상자/표 셀 이미지 붙여넣기 무음 누락 수정 (#1323)
- 작성일: 2026-06-11
- 작성자: `johndoekim`
- 관련 이슈: #1323 "글상자/표 셀 안 이미지(컨트롤) 붙여넣기가 조용히 누락 — merge_from 컨트롤 미병합·빈 텍스트 early-return"
- base: `devel` (`430d5edc`)
- head: `johndoekim:fix/issue-1323` (`9afab018`)
- 로컬 검토 브랜치: `local/pr1367-upstream`

## 1. 요약 판단

**수용 가능**으로 판단한다.

PR은 #1323의 근본 원인인 `Paragraph::merge_from()`의 두 한계, 즉 `text="" + controls=[Picture]`
문단 early-return과 controls/ctrl_data_records 미병합을 model 계층에서 직접 보강한다. 이 방향은
셀/글상자 붙여넣기뿐 아니라 같은 병합 경로를 쓰는 백스페이스 문단 병합에도 도움이 된다.

수용 시 contributor 작업 문서 6개는 활성 폴더에서 archive로 이동하는 것이 좋다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| 상태 | open |
| draft | false |
| mergeable | `MERGEABLE` |
| 변경량 | 10 files, +1217 / -8 |
| 작성자 | `johndoekim` |
| 관련 이슈 | #1323 |

커밋:

- `29fda516` — 수행계획서 작성
- `37a7a4d0` — 구현계획서 작성
- `a1dd1ad2` — merge_from 컨트롤 병합 보강
- `7d509b1b` — 셀/글상자 paste·병합 컨트롤 보존 통합 테스트
- `2c7d974e` — SVG 렌더링 검증·전체 회귀·최종 보고서
- `9afab018` — 시각 판정 통과 반영 + 후속 절차 정정

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

## 3. 변경 검토

### 3.1 `Paragraph::merge_from`

`src/model/paragraph.rs:716` 부근:

- `other.text.is_empty()`만으로 반환하던 early-return을 `other.text.is_empty() && other.controls.is_empty()`로 좁힘
- self 문단 끝의 trailing control을 8 code unit으로 반영해 `utf16_end` 계산
- `controls`, `ctrl_data_records`, `control_mask`를 병합
- `ctrl_data_records[i] ↔ controls[i]` 정렬을 유지하도록 self 쪽 누락 레코드를 `None`으로 패딩
- `char_count = text + 8×controls + 문단끝`으로 재계산
- 텍스트 또는 컨트롤이 있으면 `has_para_text = true`

판단:

- 이슈 본문이 지적한 원인과 정확히 대응한다.
- 컨트롤 위치는 기존 `char_offsets` gap / `control_text_positions()` / HWP5 `serialize_para_text` 계약과 맞는다.
- `ctrl_data_records` 인덱스 보존까지 포함해 그림·바이너리 참조가 붙여넣기 후에도 유지된다.

### 3.2 테스트 보강

`src/model/paragraph/tests.rs`:

- controls-only 문단 병합
- split → 컨트롤 문단 merge → right-half merge 흐름
- `ctrl_data_records` alignment
- 양쪽 중간 컨트롤 위치 보존
- `field_ranges.control_idx` 보정

`src/wasm_api/tests.rs`:

- 표 셀 그림 붙여넣기
- path 기반 셀 붙여넣기
- 그림 caption 내부 붙여넣기
- 본문 문단 병합 control 보존
- 셀 문단 병합 control 보존
- HWP5 serialize → parse round-trip
- SVG `<image>` 방출 확인

`src/document_core/commands/object_ops.rs`:

- 글상자 안 그림 붙여넣기 테스트 추가
- #1280 주석을 #1323 해소 사실로 갱신

판단:

- 단위, 명령 경로, 직렬화 round-trip, SVG 렌더까지 넓게 덮는다.
- 이 PR의 위험 지점인 model 계층 병합 변경에 비해 테스트 범위가 충분하다.

### 3.3 Contributor 문서

PR은 다음 문서를 활성 폴더에 추가한다.

- `mydocs/plans/task_m100_1323.md`
- `mydocs/plans/task_m100_1323_impl.md`
- `mydocs/working/task_m100_1323_stage1.md`
- `mydocs/working/task_m100_1323_stage2.md`
- `mydocs/working/task_m100_1323_stage3.md`
- `mydocs/report/task_m100_1323_report.md`

수용 시 archive 이동 권장:

- `mydocs/plans/archives/task_m100_1323.md`
- `mydocs/plans/archives/task_m100_1323_impl.md`
- `mydocs/working/archives/task_m100_1323_stage1.md`
- `mydocs/working/archives/task_m100_1323_stage2.md`
- `mydocs/working/archives/task_m100_1323_stage3.md`
- `mydocs/report/archives/task_m100_1323_report.md`

## 4. 로컬 검증

검토 브랜치: `local/pr1367-upstream`

| 명령 | 결과 |
|---|---|
| `git diff --check local/devel...HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| source patch `git apply --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib merge_from -- --nocapture` | 통과, 8 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib paste_picture_into -- --nocapture` | 통과, 6 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib preserves_controls -- --nocapture` | 통과, 2 passed |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |

GitHub Build & Test도 pass 상태다.

## 5. 리스크

| 리스크 | 평가 | 비고 |
|---|---|---|
| model 계층 `merge_from` 변경 영향 범위 | 중간 | 붙여넣기, 백스페이스 병합 등 공용 경로. 테스트가 주요 경로를 보강함 |
| non-BMP 문자 포함 병합의 `char_count` 정밀도 | 낮음 | 기존 코드도 char count 기반. 이번 이슈 경로와 무관하지만 장기적으로 UTF-16 count 정규화 여지 있음 |
| contributor 문서 활성 폴더 잔류 | 중간 | 수용 시 archive 이동 필요 |
| 시각 회귀 | 낮음 | GitHub test pass, PR 자체 시각 판정 기록, SVG `<image>` 방출 테스트 포함 |

## 6. 권장 수용 절차

작업지시자 승인 후:

1. PR 커밋을 `local/devel`에 cherry-pick
2. contributor 작업 문서 6개를 archive 폴더로 이동
3. 검증
   - `cargo fmt --check`
   - `git diff --check`
   - `CARGO_INCREMENTAL=0 cargo test --lib merge_from -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo test --lib paste_picture_into -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo test --lib preserves_controls -- --nocapture`
   - `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings`
4. 필요 시 WASM 빌드 및 rhwp-studio 시각 판정
5. 처리 보고서 작성
6. 승인 시 `devel` no-ff merge, push, PR #1367 close, Issue #1323 close

## 7. 승인 요청

위 검토 결과 기준으로 PR #1367 수용 절차를 진행해도 되는지 승인 요청한다.
