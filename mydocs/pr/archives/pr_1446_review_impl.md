# PR #1446 처리 계획

- PR: https://github.com/edwardkim/rhwp/pull/1446
- 관련 이슈: #1443
- base: `devel`
- head: `task_m100_1443`
- 처리 상태: GitHub Actions 대기

## 1. 현재 커밋 범위

PR 생성 시점의 주요 커밋:

| 순서 | 커밋 | 제목 |
|---:|---|---|
| 1 | `37579ed7` | `task 1443: 셀 선택 모드 마우스 드래그 구현` |
| 2 | `f9cf6108` | `task 1443: 일반 셀 드래그 선택과 표 메뉴 보강` |
| 3 | `b56af3b1` | `task 1443: Alt 방향키와 보호 셀 클릭 복귀 보강` |
| 4 | `ced7d62b` | `task 1443: 셀 크기 균등화 선택 범위 적용` |
| 5 | `21442339` | `task 1443: Alt+C 모양복사 1차 구현` |
| 6 | `6dabd071` | `task 1443: 셀 모양복사 확장` |
| 7 | `834a9ca1` | `task 1443: 표 선택 후 셀 편집 진입 복구` |
| 8 | `f4860959` | `task 1443: 선택 셀 높이 조정 보정` |
| 9 | `24486e83` | `task 1443: 표 외곽선 리사이즈와 높이 균등화 보정` |
| 10 | `065f787f` | `task 1443: 셀 크기 균등화 렌더링 정합 보정` |
| 11 | `5e101a6e` | `task 1443: Shift 셀 단독 리사이즈 구현` |
| 12 | `dfa5c1c2` | `task 1443: 셀 segment 리사이즈 한컴 동작 보정` |
| 13 | `db371fba` | `task 1443: 셀 안 여백 속성 표시와 렌더 보정` |
| 14 | `57dcc9c6` | `task 1443: 표 외곽 구조를 한컴 PDF 기준으로 보정` |
| 15 | `2599bae5` | `task 1443: 안 여백 텍스트 정합 확인` |
| 16 | `7b67259e` | `task 1443: 안 여백 off 리플로우 기준 보정` |
| 17 | `c6cbf988` | `task 1443: 표 이동과 속성 동기화 보정` |
| 18 | `3c897c48` | `task 1443: 모양 붙여넣기 메뉴 추가` |
| 19 | `d2692537` | `task 1443: 모양복사 단축키와 일회성 적용 보정` |
| 20 | `38ee2b05` | `task 1443: 일회성 모양복사 메뉴 정리` |
| 21 | `65013dbc` | `task 1443: 렌더링 회귀 테스트 보정` |
| 22 | `b7b1dd20` | `task 1443: 표 셀 로컬 resize 회귀 보정` |
| 23 | `94537560` | `task 1443: 전체 컬럼 resize clamp 보정` |
| 24 | `b815d1c1` | `task 1443: PR 준비 문서 추가` |

이 문서 커밋은 PR 생성 후 후속 문서 커밋으로 추가한다.

## 2. 후속 처리 절차

1. PR review 문서와 오늘할일 문서를 같은 PR head에 push한다.
2. GitHub Actions 완료를 기다린다.
3. 실패 시:
   - 실패 로그를 확인한다.
   - `mydocs/pr/archives/pr_1446_review.md`와 오늘할일에 원인을 기록한다.
   - 필요한 수정 후 같은 PR head에 push한다.
4. 통과 시:
   - merge 가능 여부를 최종 확인한다.
   - 작업지시자 지시에 따라 merge한다.
5. merge 후:
   - #1443 자동 close 여부를 확인한다.
   - 필요하면 수동 close 코멘트에 릴리즈 반영 예정 버전을 남긴다.

## 3. 검증 기록

PR 생성 전 완료한 검증:

- `cargo build --release`
- `cargo test --release --lib`
- `cargo test --profile release-test --tests`
- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings`
- `cd rhwp-studio && npx tsc --noEmit`
- `cd rhwp-studio && npm test`
- `wasm-pack build --target web --out-dir pkg`

## 4. 주의 사항

- PR은 이미 Open PR로 생성했다.
- 이후 문서 커밋 push로 GitHub Actions가 다시 돌 수 있다.
- merge 전에는 문서 커밋이 PR diff에 포함되어 있는지 확인한다.

