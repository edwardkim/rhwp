# PR #1398 검토 - replaceAll exportHwp raw_stream 캐시 무효화

- PR: https://github.com/edwardkim/rhwp/pull/1398
- 제목: `fix: replaceAll 치환이 exportHwp에서 유실 — raw_stream 캐시 무효화 누락 (#1385)`
- 작성일: 2026-06-12
- 작성자: `oksure`
- 작성자 상태: contributor
- 관련 이슈: #1385 `[bug] replaceAll/replaceText 변경이 exportHwp 직렬화에 반영되지 않음`
- base: `devel`
- head: `oksure:contrib/fix-1385-replace-all-raw-stream`
- 처리 상태: merge 준비 가능

## 1. 요약 판단

**merge 가능**으로 판단한다.

PR #1398은 `replace_all_native`가 문단을 직접 조작한 뒤 HWP5 구역의 `raw_stream` 캐시를
무효화하지 않아 `exportHwp()` 저장 시 치환 결과가 유실되던 문제를 수정한다. 수정은 affected
section recompose 루프에서 `raw_stream = None`을 추가하는 작은 범위이며, 본문 문단과 표 셀
경로에 대한 회귀 테스트 2건을 포함한다.

GitHub Actions와 로컬 사전 검증이 모두 통과했다.

## 2. PR 정보

| 항목 | 값 |
|---|---|
| PR 상태 | open |
| draft | false |
| base | `devel` |
| head | `oksure:contrib/fix-1385-replace-all-raw-stream` |
| author association | `CONTRIBUTOR` |
| maintainer can modify | true |
| mergeable | true |
| mergeStateStatus | `BLOCKED` |
| 연결 이슈 | #1385 |

`mergeStateStatus=BLOCKED`는 필수 체크 대기/상태 반영 중 표시였고, 최종 GitHub Actions는 통과했다.

## 3. 변경 범위

| 파일 | 내용 |
|---|---|
| `src/document_core/queries/search_query.rs` | `replace_all_native` affected section 재구성 직전 `raw_stream = None` 무효화 추가 |
| `tests/issue_1385_replace_export_roundtrip.rs` | 본문 문단/표 셀 `replaceAll -> exportHwp -> reparse` 회귀 테스트 추가 |

주요 변경 지점:

- `src/document_core/queries/search_query.rs`
  - affected section 목록을 dedup한 뒤 각 section의 raw HWP5 stream 캐시를 무효화한다.
  - 캐시가 남아 있으면 serializer가 수정된 IR 대신 원본 section stream을 그대로 반환하므로 #1385 증상이 재현된다.
- `tests/issue_1385_replace_export_roundtrip.rs`
  - `samples/2022년 국립국어원 업무계획.hwp` 본문 텍스트 치환 보존 확인
  - `samples/복학원서.hwp` 표 셀 텍스트 치환 보존 확인
  - 글상자 텍스트 순회와 `replace_all_native` JSON 결과 필드 파싱도 포함

## 4. 이전 리뷰 문서 동반 범위

작업지시자 지시에 따라 이번 PR merge에는 아직 upstream/devel에 반영되지 않은 이전 PR 리뷰
기록도 함께 포함해야 한다. 로컬 검토 브랜치 `local/pr1398-review`에는 다음 문서 커밋이 포함되어
있다.

- PR #1374 처리 기록
  - `mydocs/orders/20260612.md`
  - `mydocs/pr/archives/pr_1374_review.md`
  - `mydocs/pr/archives/pr_1374_review_impl.md`
- PR #1376 처리 기록
  - `mydocs/pr/archives/pr_1376_review.md`
  - `mydocs/pr/archives/pr_1376_review_impl.md`

주의: PR #1376의 코드 실패 커밋은 포함하지 않고 리뷰 문서만 포함했다.

## 5. 검증 결과

### 5.1 GitHub Actions

최종 GitHub 체크는 모두 통과했다.

| 체크 | 결과 |
|---|---|
| Build & Test | pass, 13m 54s |
| CodeQL | pass |
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pass |
| WASM Build | skipped |

### 5.2 로컬 검증

`mydocs/manual/pr_review_workflow.md`의 로컬 사전 검증 기준으로 다음 명령을 수행했다.

| 명령 | 결과 |
|---|---|
| `cargo test --test issue_1385_replace_export_roundtrip` | 통과, 2 passed |
| `cargo build --lib` | 통과 |
| `cargo test --lib` | 통과, 1724 passed / 0 failed / 6 ignored |
| `cargo clippy -- -D warnings` | 통과 |
| `cargo test --doc` | 통과, 0 passed / 0 failed / 1 ignored |
| `cargo test --test svg_snapshot` | 통과, 8 passed |

## 6. 리스크

| 리스크 | 평가 |
|---|---|
| 수정 범위 | 낮음. affected section raw stream 무효화 3줄 |
| 저장 회귀 | 낮음. 본문/표 셀 export roundtrip 회귀 테스트 추가 |
| 기존 serializer raw stream 보존 경로 영향 | 낮음. replaceAll로 실제 편집된 section에만 적용 |
| 문서 동반 누락 | 관리 필요. #1374/#1376 리뷰 문서가 PR head에 함께 반영되어야 함 |

## 7. 권고

PR #1398은 merge 준비 가능하다.

다만 merge 전에 다음을 확인한다.

1. PR head에 #1374/#1376 리뷰 문서 커밋이 실제 포함되어 있는지 확인한다.
2. PR diff가 코드 2파일과 문서 archives 파일만 포함하는지 확인한다.
3. merge 후 #1385가 자동 close되지 않으면 수동 close한다.
