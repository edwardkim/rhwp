# PR #1351 처리 보고서 — useFontSpace 직렬화 시 항상 false 고정 수정

## 1. 개요

| 항목 | 내용 |
|---|---|
| PR | #1351 |
| 작성자 | `Martinel2` |
| 관련 이슈 | #1350 |
| 검토 브랜치 | `local/pr1351-upstream` |
| 통합 방식 | 현재 `local/devel` 기준 본질 커밋 단일 cherry-pick + 메인테이너 보강 |
| 원 PR head | `232d6c14` |
| 원 PR 본질 커밋 | `d9107849` |
| 반영 커밋 | `26802b18`, `57b0c725` |
| devel merge | `e2e809c3` |
| PR close | `2026-06-10T01:46:34Z` |
| Issue #1350 close | `2026-06-10T01:46:47Z` |

## 2. 처리 내용

PR #1351은 head 브랜치 전체에 #1298/#1321 과거 작업과 fork `devel` merge commit이 함께
섞여 있었다. 따라서 PR 전체를 merge하지 않고, #1350 본질 커밋만 현재 `local/devel`에
cherry-pick했다.

반영 커밋:

- `26802b18` — `fix: useFontSpace IR 필드 추가 및 HWP5/HWPX 파서·직렬화기 반영 (#1350)`
- `57b0c725` — `fix: preserve useFontSpace in HWP5 serializer`

제외한 PR 범위:

- #1298 / #1321 계열 과거 작업 문서와 커밋
- PR 내부 merge commit
- 현재 `devel` 최신 작업을 되돌릴 수 있는 fork `devel` 전체 diff

## 3. 변경 내용

### 3.1 PR 본질 커밋

- `CharShape`에 `use_font_space: bool` 필드 추가
- HWP5 `CharShape` attr bit 25를 `use_font_space`로 파싱
- HWPX `<hh:charPr useFontSpace="...">` 속성을 파싱
- HWPX serializer가 `useFontSpace`를 항상 `0`으로 내보내던 하드코딩 제거
- HWPX serializer가 `cs.use_font_space` 값을 `useFontSpace` 속성으로 출력
- HWP5/HWPX 파서·직렬화 회귀 테스트 추가

### 3.2 메인테이너 보강

리뷰 중 HWP5 serializer에서 `cs.use_font_space`를 attr bit 25로 재구성하지 않는 빈틈을 확인했다.

보강 내용:

- `src/serializer/doc_info.rs::serialize_char_shape()`에서 `cs.use_font_space`를 attr bit 25에 반영
- `use_font_space=true`일 때 bit 25가 set되고, `false`일 때 기존 attr bit 25도 clear되는 테스트 추가

이 보강으로 HWPX에서 읽은 `useFontSpace="1"` 값도 `raw_data` 없는 CharShape를 HWP5로 저장할 때
bit 25로 보존된다.

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
| `cargo fmt --check` | 통과 |
| `git diff --check HEAD~2..HEAD` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_parse_char_shape_use_font_space -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib write_char_pr_use_font_space_roundtrip -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_serialize_char_shape_use_font_space_bit -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib test_serialize_char_shape_roundtrip -- --nocapture` | 통과 |
| `CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx::header -- --nocapture` | 통과, 7 passed |
| `CARGO_INCREMENTAL=0 cargo clippy --lib -- -D warnings` | 통과 |

## 5. 판정

**수용 가능**.

이번 변경은 렌더러 동작을 직접 바꾸는 것이 아니라 CharShape metadata의 parser/serializer 보존을
정정한다. HWPX에서는 `useFontSpace`가 `header.xml`의 `<hh:charPr>` 속성으로 저장되며, 현재
rhwp는 이 값을 파싱하지 않고 HWPX 직렬화 시 항상 `0`으로 내보냈다.

PR 본질 커밋은 HWP5/HWPX parser와 HWPX serializer 경로를 고쳤고, 메인테이너 보강으로 HWP5
serializer attr bit 25 재구성도 맞췄다. 따라서 `HWP5 → IR → HWPX`, `HWPX → IR → HWPX`,
`HWPX → IR → HWP5` 경로에서 `useFontSpace` 보존성이 개선된다.

시각 판정은 생략 가능하다고 본다. 현재 렌더러가 `use_font_space`를 직접 사용하는 경로는 없고,
이번 PR의 본질은 저장/내보내기 metadata 보존이다.

## 6. 후속 절차

처리 완료:

- [x] `mydocs/pr/pr_1351_report.md` 및 주문서 갱신 커밋 — `8abb3294`
- [x] `local/devel` → `devel` no-ff merge — `e2e809c3`
- [x] `origin/devel` push — `e2e809c3`
- [x] PR #1351에 처리 코멘트 작성 — https://github.com/edwardkim/rhwp/pull/1351#issuecomment-4665702342
- [x] PR #1351 close — `2026-06-10T01:46:34Z`
- [x] Issue #1350 close — `2026-06-10T01:46:47Z`
