# PR #3549 검토 — HWPX 내부 OLE BinData size prefix 복원

- 검토일: 2026-07-29
- PR: [#3549](https://github.com/edwardkim/rhwp/pull/3549)
- 관련 이슈: [#3547](https://github.com/edwardkim/rhwp/issues/3547)
- 작성자 / reviewer: `@JamesPsh` / `@jangster77` (collaborator 매개 외부 PR)
- base / 현재 code head: `devel` `56b9db545c8be34101b2df8cbf4a59a7b3a47cb4` / `e1de8cf11b3284d68a62b308edce47d1f77baeee`
- 원 code 변경 규모: 2 files, +94 / -1 (검토 기록 추가 전)

## 변경 범위와 판정

HWPX 파서는 내부 `BinData/*.ole`를 `Storage`·`OLE`로 식별하고, `[4-byte LE size][CFB]` 형식의
선두 size prefix를 IR에서 제거한다. 이번 변경은 저장 직전에 대소문자 무관 `OLE` 확장자와 8-byte
CFB magic을 함께 확인한 경우에만 `u32` little-endian payload 길이를 다시 붙인다. 따라서
[#954](https://github.com/edwardkim/rhwp/pull/954)의 HWP5 CFB writer와 같은 역방향 계약을 HWPX ZIP
BinData 출력 경로에 적용한다.

CFB magic이 아닌 `.ole` 데이터는 분기에 들어가지 않으며, 외부 링크는 ZIP BinData를 쓰지 않는 기존
경로를 유지한다. renderer, layout, paint, HWPX XML 구조와 manifest 매핑을 바꾸지 않는 serializer
바이트 보존 수정이다.

신규 회귀는 기존 실물 fixture `samples/hwpx/143E433F503322BD33.hwpx`를 재저장한 뒤 내부 OLE마다
`[size][CFB]` header, prefix 길이, 원본과의 전체 바이트 동일성을 검증한다. 단순 재파싱 성공만으로
놓칠 수 있는 한컴의 OLE size 해석 경계를 직접 고정한다.

## 검증

| 검증 | 결과 |
| --- | --- |
| 최신 `devel` 위 merge simulation | conflict 없음, `git diff --check` 통과 |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_3547_ole_size_prefix` | 1 passed |
| `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` | exit code 0 |
| `CARGO_INCREMENTAL=0 cargo fmt --all -- --check` | 통과 |
| GitHub Actions — CI | preflight, Lint, archive, Native Skia, default-feature 8 shards, `Build & Test` 모두 success |
| GitHub Actions — CodeQL | preflight와 JavaScript/TypeScript·Python·Rust 분석, aggregate 모두 success |

이번 변경은 renderer/layout 출력 변경이 아니며, 회귀가 실물 HWPX의 내부 OLE ZIP 바이트를 직접
비교한다. 따라서 별도 PDF/SVG visual sweep은 merge 판단의 필수 근거로 사용하지 않는다.

## 권고와 merge 전 조건

**권고: 수용.** current code head `e1de8cf11b3284d68a62b308edce47d1f77baeee`의 full CI와 CodeQL이
성공했고 merge 상태는 `MERGEABLE`·`CLEAN`이었다. 이 archive review 문서만 추가한 최신 head가
review-only fast-pass preflight와 `Build & Test` aggregate를 통과하고 mergeable 상태를 유지하는지
재확인한 뒤, 작업지시자 승인 범위에서 squash merge한다. merge 뒤에는 #3547 종료, contributor comment,
devel sync와 review branch·worktree·전용 target 정리를 확인한다.
