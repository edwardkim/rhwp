# PR #1751 리뷰 — Task #1750 저장 LINE_SEG 전체 리셋 신호 분할 가드

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1751 |
| 작성자 | planet6897 |
| base | devel |
| head | pr/devel-1750 |
| 제목 | Task #1750: 분할 경로가 저장 LINE_SEG 전체-리셋 신호를 무시하는 결함 수정 |
| 문서 작성 시점 참고값 | draft=false, mergeable=MERGEABLE |
| 로컬 검토 브랜치 | `local/pr_page2_batch_check` |

## 체리픽 검토

- 오래된 순서 누적 검토: #1746 -> #1751 -> #1752 -> #1754
- 적용 커밋: `a7721d7a00bc77ae12cbc2c7a0f9b6b910de1c7a`
- 로컬 체리픽 커밋: `bdc07c181`
- PR 내부 merge commit `b1a3d7dfea2a6a42723e944317edf3b205b32a06`는 체리픽 검토에서 제외했다.
- #1746 적용 후 `upstream/devel` 기준 누적 체리픽 충돌 없음.

## 변경 범위

- `src/renderer/typeset.rs`: 문단 전체가 새 쪽 상단으로 저장 LINE_SEG에 인코딩된 경우 분할 대신 페이지 넘김을 수행하는 `stored_whole_para_reset` 가드 추가.
- `tests/issue_1750_split_guard_spacing_before.rs`: pi=22가 1쪽 분할 잔류 없이 2쪽 시작에 배치되는지 검증.
- `samples/task1750/split_guard_spacing_before.hwp`와 관련 문서 추가.

## 검토 결과

- 가드는 전체 배치 실패 후 분할 진입 직전에만 적용된다.
- 단일 단, 첫 실줄 vpos가 near-top 범위 `(0, 2500]`, 직전 문단이 페이지 하단부인 조건을 모두 요구해 일반 흐름 영향 범위가 좁다.
- #1746의 wrap strip/소급 기록 경로와 직접 상태를 공유하지 않아 누적 적용 간섭은 발견되지 않았다.

## 로컬 검증

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo fmt --check`: 통과
- `git diff --check`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1750_split_guard_spacing_before -- --nocapture`: 1 passed
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings`: 통과

## PR 내용 기준 검증

- 이 PR의 핵심은 저장 LINE_SEG가 문단 전체의 새 쪽 상단 배치를 나타내는 경우, 분할 경로로 들어가지 않고 페이지 넘김을 수행하는지다.
- `tests/issue_1750_split_guard_spacing_before.rs`가 대상 문단 pi=22의 배치 쪽과 분할 잔류 여부를 직접 확인한다.
- 보조 untracked 샘플 `samples/task1750/split_guard_spacing_before.hwpx`도 `dump-pages`로 확인했다. pi=22는 1쪽에 없고 2쪽 `vpos=700..2620`에서 시작한다.
- `samples/task1750/split_guard_spacing_before-2024.pdf` 기준 visual sweep에서도 HWP/HWPX 모두 자동 후보 0건이다.

## 시각 검증

- `samples/task1750/split_guard_spacing_before.hwp` vs `samples/task1750/split_guard_spacing_before-2024.pdf`
  - command: `python3 scripts/task1274_visual_sweep.py --key pr1751-split-guard-hwp --hwp samples/task1750/split_guard_spacing_before.hwp --pdf samples/task1750/split_guard_spacing_before-2024.pdf --out output/pr-page2-visual --rhwp-bin target/release-test/rhwp --pixel-diff-threshold 32`
  - SVG/PDF pages: 5/5
  - flagged: 0/5
  - review contact sheet: `output/pr-page2-visual/pr1751-split-guard-hwp/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 7.42244, worst 0.91055
- `samples/task1750/split_guard_spacing_before.hwpx` vs `samples/task1750/split_guard_spacing_before-2024.pdf`
  - command: `python3 scripts/task1274_visual_sweep.py --key pr1751-split-guard-hwpx --hwp samples/task1750/split_guard_spacing_before.hwpx --pdf samples/task1750/split_guard_spacing_before-2024.pdf --out output/pr-page2-visual --rhwp-bin target/release-test/rhwp --pixel-diff-threshold 32`
  - SVG/PDF pages: 5/5
  - flagged: 0/5
  - review contact sheet: `output/pr-page2-visual/pr1751-split-guard-hwpx/review_contact_sheet.png`
  - `visual_accuracy_proxy_percent`: average 7.42244, worst 0.91055

## PR 코멘트 처리 시 PNG 위치

- 정상 대표 샘플 1장의 로컬 위치만 기록한다.
- 코멘트 처리 시 제시할 PNG: `mydocs/pr/assets/pr_1751_visual_review_p2.png`
- 코멘트 요지: p2에서 pi=22가 1쪽에 분할 잔류하지 않고 2쪽 시작으로 배치되며, HWP/HWPX visual sweep 자동 후보는 모두 0/5이다.
- 코멘트 요지에 다음 요청도 포함한다: 이후 시각 대조가 필요한 샘플을 추가할 때는 한컴 2020, 한컴 2024 등 기준 환경에서 저장한 PDF 파일도 함께 업로드해 달라고 요청한다.

## 결론

PR 내용 기준 로컬 검증과 기준 PDF visual sweep 모두 통과했다. #1746 다음 순서의 merge 후보로 판단한다.

## 후속 처리 결과

- 통합 PR: #1810
- merge commit: `716fbca92ef4d5c67194ec6575bcc06413beacf6`
- 원 PR 코멘트: https://github.com/edwardkim/rhwp/pull/1751#issuecomment-4866609986
- 원 PR 상태: superseded close 완료
- 관련 이슈: #1750 close 완료, https://github.com/edwardkim/rhwp/issues/1750#issuecomment-4866621421
