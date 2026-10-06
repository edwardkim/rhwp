# PR #1367 처리 보고서 - merge_from 컨트롤 병합 보강

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1367 |
| 제목 | fix(model): merge_from 컨트롤 병합 보강 - 글상자/표 셀 이미지 붙여넣기 무음 누락 수정 (#1323) |
| 작성자 | `johndoekim` |
| 관련 이슈 | #1323 |
| PR base | `devel` |
| 검토 브랜치 | `local/pr1367-upstream` |
| 원 PR head | `9afab018` |
| 처리 기준 | `local/devel` |
| 통합 방식 | PR 커밋 6개 cherry-pick + contributor 문서 archive 정리 |
| 리뷰 문서 커밋 | `f57a5028` |
| 문서 정리 커밋 | `d12b9637` |
| 처리 보고서 커밋 | `3188adcb` |
| devel merge | `3c13532f` |
| PR close | `2026-06-11T00:20:08Z` |
| Issue #1323 close | `2026-06-11T00:20:21Z` |

## 2. 처리 내용

작업지시자 리뷰 보고서 승인 후 PR #1367의 원 커밋 6개를 `local/devel` 위에 cherry-pick했다.

원 PR 커밋:

```text
29fda516 Task #1323: 수행계획서 작성
37a7a4d0 Task #1323: 구현계획서 작성
a1dd1ad2 Task #1323: merge_from 컨트롤 병합 보강 (stage1)
7d509b1b Task #1323: 셀/글상자 paste·병합 컨트롤 보존 통합 테스트 (stage2)
2c7d974e Task #1323: SVG 렌더링 검증·전체 회귀·최종 보고서 (stage3)
9afab018 Task #1323: 시각 판정 통과 반영 + 후속 절차 컨트리뷰터 워크플로우로 정정
```

devel 반영 커밋:

```text
11363acd Task #1323: 수행계획서 작성
ec018d04 Task #1323: 구현계획서 작성
56633c01 Task #1323: merge_from 컨트롤 병합 보강 (stage1)
906f1300 Task #1323: 셀/글상자 paste·병합 컨트롤 보존 통합 테스트 (stage2)
8158b999 Task #1323: SVG 렌더링 검증·전체 회귀·최종 보고서 (stage3)
46adaca2 Task #1323: 시각 판정 통과 반영 + 후속 절차 컨트리뷰터 워크플로우로 정정
```

Contributor 작업 문서는 PR 처리 문서와 분리하기 위해 archive로 이동했다.

- `mydocs/plans/archives/task_m100_1323.md`
- `mydocs/plans/archives/task_m100_1323_impl.md`
- `mydocs/working/archives/task_m100_1323_stage1.md`
- `mydocs/working/archives/task_m100_1323_stage2.md`
- `mydocs/working/archives/task_m100_1323_stage3.md`
- `mydocs/report/archives/task_m100_1323_report.md`

## 3. 변경 내용

`src/model/paragraph.rs`:

- `Paragraph::merge_from()`에서 `other.text`가 비어 있어도 `other.controls`가 있으면 병합하도록 early-return 조건을 보강
- self 문단 끝 trailing control의 8 code unit을 병합 기준 위치에 반영
- `controls`, `ctrl_data_records`, `control_mask` 병합 추가
- `ctrl_data_records[i]`와 `controls[i]`의 인덱스 정렬을 유지하도록 누락 레코드 `None` 패딩
- 병합 후 `char_count`와 `has_para_text` 재계산

테스트:

- controls-only 문단 병합
- split 후 control 포함 문단 재병합
- `ctrl_data_records` alignment
- 양쪽 문단 중간 control 위치 보존
- `field_ranges.control_idx` offset 보정
- 표 셀, path 기반 셀, 그림 caption, 글상자 내부 그림 붙여넣기
- HWP5 serialize/parse round-trip
- SVG `<image>` 방출 확인

## 4. 검증 결과

GitHub checks:

| 체크 | 결과 |
|---|---|
| Build & Test | pass |
| CodeQL | pass |
| Analyze rust | pass |
| Analyze javascript-typescript | pass |
| Analyze python | pass |
| WASM Build | skipped |

로컬 검증:

| 명령 | 결과 |
|---|---|
| `git diff --check origin/devel..HEAD` | 통과 |
| `cargo fmt --check` | 통과 |
| source patch `git apply --check` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib merge_from -- --nocapture` | 통과, 8 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib paste_picture_into -- --nocapture` | 통과, 6 passed |
| `CARGO_INCREMENTAL=0 cargo test --lib preserves_controls -- --nocapture` | 통과, 2 passed |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |
| `CARGO_INCREMENTAL=0 cargo check --lib --target wasm32-unknown-unknown -j 2` | 통과 |
| `docker compose --env-file .env.docker run --rm wasm` | 통과, `Done in 1m 53s` |

작업트리:

| 항목 | 결과 |
|---|---|
| `git status --short --branch` | clean on `local/devel` |

## 5. 판정

**수용 가능**.

이 PR은 #1323의 본질 원인인 controls-only 문단 병합 누락과 control data 미병합을 model 계층에서
직접 해결한다. 수정 범위는 `Paragraph::merge_from()` 중심으로 좁고, 표 셀·글상자·caption·HWP5
round-trip·SVG 이미지 방출까지 회귀 테스트가 보강되어 있다.

남는 주의점은 기존 코드가 가진 non-BMP 문자 포함 시 `char_count` 정밀도 문제다. 이번 변경이 새로
도입한 문제는 아니며, #1323의 그림 컨트롤 병합 경로와 직접 관련도 낮다. 장기적으로 UTF-16 code unit
기준 정규화를 검토할 수 있다.

## 6. 후속 절차

처리 완료:

- [x] `mydocs/pr/pr_1367_report.md` 및 주문서 갱신 커밋 — `3188adcb`
- [x] `local/devel`을 `devel`에 no-ff merge — `3c13532f`
- [x] `origin/devel` push — `3c13532f`
- [x] PR #1367에 처리 코멘트 작성
- [x] PR #1367 close — `2026-06-11T00:20:08Z`
- [x] Issue #1323 close — `2026-06-11T00:20:21Z`

PR 검토/처리 문서는 현재 `mydocs/pr/`에 유지한다.
