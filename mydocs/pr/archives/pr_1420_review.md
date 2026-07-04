# PR #1420 리뷰 - rhwp-studio 다크테마 지원

## 1. PR 개요

| 항목 | 내용 |
|---|---|
| PR | #1420 |
| 제목 | rhwp-studio 다크테마 지원 |
| 작성자 | jangster77 |
| 관련 이슈 | #1158 rhwp-studio 다크모드 지원 |
| base | `devel` |
| head | `jangster77:task_m100_1158` |
| draft | false |
| mergeable | `BLOCKED` |
| 초기 merge state | GitHub Actions 진행 중 |
| 현재 head | `667c47db` |
| 변경량 | 47 files, +3766 / -358 |

PR 본문에 `Closes #1158`가 포함되어 있으므로 merge 시 이슈 자동 close 대상이다.

## 2. 변경 범위

핵심 변경:

- `rhwp-studio/src/core/theme.ts`, `user-settings.ts`, `main.ts`
  - `system | light | dark` 테마 저장/적용/복원 구조 추가
- `rhwp-studio/src/command/commands/view.ts`, `index.html`
  - 보기 메뉴에서 테마 선택 항목 노출
- `rhwp-studio/src/styles/*.css`
  - 메뉴바, 툴바, 서식바, 상태바, 작업영역, 다이얼로그 dark token 정리
- `rhwp-studio/src/view/ruler.ts`
  - dark mode에서 눈금자 palette 재적용 및 redraw
- `rhwp-studio/public/images/icon_small_ko_dark.svg`
  - dark toolbar/menu 전용 스프라이트 추가
- `rhwp-studio/e2e/theme-mode.test.mjs`
  - 테마 저장/복원, 편집 용지 유지, 눈금자 dark tone 스모크 E2E
- `mydocs/report/assets/task_m100_1158_dark_*.png`
  - desktop/mobile 시각 검토 자료 추가
- `mydocs/pr/archives/pr_1419_review*.md`, `mydocs/orders/20260616.md`
  - PR #1419 후속 문서 정리와 주문서 기록 동반

## 3. 시각 검토 반영 사항

작업지시자 요청에 따라 다음 화면까지 PR 본문에 첨부했다.

- desktop dark toolbar
- desktop file menu
- mobile dark main screen
- mobile dark file menu

모바일 자료는 Chrome headless 뷰포트로 별도 캡처했다.

## 4. 로컬 검증

PR 준비 단계에서 로컬 필수 검증을 완료했다. 이후 문서 보강 커밋은 문서 범위만 바뀌므로 재실행 대신
`git diff --check`로 확인했다.

| 명령 | 결과 |
|---|---|
| `cd rhwp-studio && npm run build` | 통과 |
| `cd rhwp-studio && node e2e/theme-mode.test.mjs --mode=headless` | 통과 |
| `cargo build --release` | 통과 |
| `cargo test --release --lib` | 통과 |
| `cargo test --profile release-test --tests` | 통과 |
| `cargo fmt --check` | 통과 |
| `git diff --check` | 통과 |

## 5. GitHub Actions

문서 push 전 확인 상태:

| 체크 | 상태 |
|---|---|
| Analyze (javascript-typescript) | pass |
| Analyze (python) | pass |
| Analyze (rust) | pending |
| Build & Test | pending |
| Canvas visual diff | pending |
| CodeQL | skipping |
| WASM Build | skipping |

문서 커밋 push 후 GitHub Actions가 다시 실행되므로, 최종 merge 판단은 재실행된 checks 기준으로 진행한다.

## 6. 리스크

| 항목 | 평가 |
|---|---|
| 변경 범위 | 중간. CSS 표면은 넓지만 대부분 token 치환과 dark palette 정리 |
| 편집 용지/문서 렌더 회귀 | 낮음. 흰 편집 용지 유지와 눈금자 분리 원칙을 명시적으로 검증 |
| 모바일 표현 | 낮음~중간. mobile 시각 자료는 확보했지만 실제 터치 인터랙션 전체는 후속 확인 여지 |
| PR #1419 문서 동반 정리 | 낮음. 문서 rename과 주문서 갱신만 포함 |

## 7. 최종 권고

현재 상태에서는 GitHub Actions 재실행 통과 후 merge 가능으로 판단한다.

권고 순서:

1. archive 경로의 PR 리뷰 문서와 오늘할일 문서를 PR head에 push
2. PR diff에 `mydocs/pr/archives/pr_1420_review.md`, `mydocs/pr/archives/pr_1420_review_impl.md`, `mydocs/orders/20260616.md` 포함 확인
3. GitHub Actions 재실행 완료 대기
4. required checks 통과 시 merge
5. #1158 close 여부 확인
6. `upstream/devel` 동기화
