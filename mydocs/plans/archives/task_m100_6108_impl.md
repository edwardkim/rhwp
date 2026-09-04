# 구현 계획 — Task M100 #6108

- **이슈**: [#6108](https://github.com/edwardkim/rhwp/issues/6108)
- **브랜치**: `codex/issue-6108-zoom-fit`
- **기준 commit**: `upstream/devel` `2166f4065`
- **문서 성격**: 구현 전 파일 단위 설계

## 계산 계약

### 쪽 배치 점유 크기

| 쪽 배치 | 열 | 행 | 맞춤 의미 |
| --- | ---: | ---: | --- |
| 자동 | 1 | 1 | 현재 자동 열 수와 무관하게 한 쪽 기준 |
| 한 쪽 | 1 | 1 | 한 페이지 기준 |
| 두 쪽 | 2 | 1 | 연속 두 페이지와 내부 gap 기준 |
| 맞쪽 | 2 | 1 | 첫 빈 슬롯과 무관한 두 열 펼침 기준 |
| 여러 쪽 | 지정 columns | 지정 rows | 전체 가로×세로 배열과 내부 gap 기준 |

- 폭 맞춤은 `availableWidth / (pageWidth × columns)`를 기준으로 한다.
- 쪽 맞춤은 위 가로 비율과 `availableHeight / (pageHeight × rows)` 중 작은 값을 사용한다.
- 프레임 padding과 내부 page gap을 계산에 한 번만 포함한다.
- 결과는 `MIN_DOCUMENT_ZOOM`~`MAX_DOCUMENT_ZOOM`, 즉 10~500%로 제한한다.
- invalid geometry는 기존 안전 기본값을 유지하고 NaN·Infinity를 반환하지 않는다.

## 파일별 구현

### `rhwp-studio/src/view/zoom-fit.ts`

- `PageArrangement`에서 표시 블록의 columns·rows를 구하는 순수 helper를 둔다.
- 폭·쪽 맞춤을 하나의 arrangement-aware calculator로 통합한다.
- `resolveZoomFitZoom()`이 두 모드 모두 같은 calculator와 동일한 metrics를 사용하게 한다.
- 400%로 별도 선언된 상한을 제거하고 `page-arrangement.ts`의 문서 배율 10~500% 상수를 재사용한다.

### `rhwp-studio/src/view/page-arrangement.ts`

- 여러 쪽만을 위한 중복 배율 계산은 공통 calculator로 이관한다.
- 쪽 배치 타입·정규화·문서 배율 상수 책임은 유지한다.

### `rhwp-studio/src/view/zoom-dialog-state.ts`

- 고정·사용자·폭 맞춤·쪽 맞춤을 최종 배율로 바꾸는 경로가 `resolveZoomFitZoom()`을 사용하게 한다.
- 여러 쪽은 비율 radio 값과 무관하게 전체 배열 맞춤을 선택하지만 별도 수학 구현은 갖지 않는다.
- 수치 배율은 같은 10~500% 상수를 사용하되, invalid 사용자 입력 오류 표시는 #6109에 남긴다.

### `rhwp-studio/src/command/commands/view.ts`

- 현재 문서·뷰포트·쪽 배치에서 공통 fit metrics를 만드는 helper를 둔다.
- 보기 메뉴와 확대/축소 대화상자의 초기 맞춤값·확인 적용이 동일한 metrics와 calculator를 사용하게 한다.
- 상태 표시줄은 기존처럼 command dispatcher만 호출하며 자체 계산을 추가하지 않는다.

### `rhwp-studio/src/main.ts`

- 저장된 맞춤 복원은 `userSettings`의 정규화된 `PageArrangement` snapshot과 공통 calculator를 사용한다.
- CSS 소비처가 없는 `is-neutral` class toggle과 이를 위한 DOM lookup을 제거한다.
- 배율 표시·range 값·100% 눈금 자체는 그대로 유지한다.

### `rhwp-studio/src/ui/zoom-dialog.ts`

- 여러 쪽을 선택하면 적용되지 않는 비율 radio를 disabled 상태로 표시한다.
- 자동·한 쪽·두 쪽·맞쪽으로 돌아오면 기존 비율 선택을 복원해 다시 활성화한다.
- 사용자 입력의 오류 메시지·`aria-invalid`·제출 차단은 #6109 범위로 남긴다.

### `rhwp-studio/src/view/canvas-view.ts`

- 저장소 전체 소비처를 감사한 뒤 emit되지 않는 `page-arrangement-changed` 구독을 제거한다.
- `page-view-settings-changed`와 `setPageViewSettings()`를 유일한 배치+이동 적용 경로로 유지한다.
- 외부 소비처가 없는 단일 필드 getter/setter wrapper를 제거하고 필요한 메서드 가시성을 좁힌다.
- 실제 배치 전환·앵커·Canvas 해제 로직은 변경하지 않는다.

## 테스트

### 순수 계산

- `zoom-fit.test.ts`
  - 자동/한 쪽 1×1, 두 쪽/맞쪽 2×1, 여러 쪽 1×1·2×2·4×1·8×8
  - 폭 맞춤과 쪽 맞춤의 내부 gap·가로/세로 frame padding
  - 10% 하한과 500% 상한
- `zoom-dialog.test.ts`
  - 다섯 쪽 배치의 비율 선택 결과
  - 여러 쪽이 공통 전체 배열 맞춤을 사용하는지 확인
- `zoom-fit-mode-persistence.test.ts`
  - 저장된 폭/쪽 맞춤이 현재 쪽 배치 기준으로 복원되는지 확인

### 통합·구조 회귀

- `zoom-dialog-integration.test.ts`
  - 메뉴·상태 표시줄·대화상자가 같은 command/calculator 경로를 사용하는지 확인
  - 여러 쪽 비율 UI disabled 계약
- `canvas-view-page-arrangement.test.ts`
  - 무사용 이벤트·wrapper 제거 뒤 `page-view-settings-changed` 단일 경로 유지
- 실제 browser
  - 자동/한 쪽의 한 페이지 맞춤
  - 두 쪽/맞쪽의 두 페이지 맞춤
  - 여러 쪽 가로×세로 전체 맞춤
  - 메뉴·상태 표시줄·대화상자 진입점 결과 일치

## 커밋 경계 후보

1. `docs(test): #6108 쪽 배치별 맞춤 계산 계약`
2. `fix(studio): 쪽 배치별 맞춤 배율 계산을 단일화한다`
3. `refactor(studio): 보기 설정의 무사용 경로를 정리한다`
4. `docs(test): #6108 통합 검증 결과`

각 경계는 해당 Stage 결과 승인 뒤에만 커밋한다. #6109 변경은 이 브랜치에 포함하지 않는다.

## #6109 stacked PR 연결 계획

1. #6108 code·test·문서 head를 로컬 검증하고 결과 승인을 받는다.
2. #6109 결과 승인 뒤 #6108 head에서 `codex/issue-6109-zoom-dialog-transaction`을 만든다.
3. 두 branch를 `gh stack init --base devel codex/issue-6108-zoom-fit
   codex/issue-6109-zoom-dialog-transaction` 순서로 native stack에 등록한다.
4. 두 PR의 로컬 검증·문서·한국어 제목·본문 초안을 준비한 뒤 별도 게시 승인으로 `gh stack submit`을
   실행한다. bottom #6108은 `devel`, top #6109는 #6108 branch를 base로 게시한다.
5. PR 본문에는 stack 순서, 각 이슈의 독립 수용 기준, bottom-first merge 조건을 명시한다.
6. #6108을 먼저 merge하면 GitHub가 남은 #6109를 stack trunk인 `devel`에 자동 rebase·retarget한다.
   이후 `gh stack sync`로 로컬 상태를 맞추고 #6109의 최신 head를 다시 검증한다.

현재 GitHub CLI는 공식 `gh stack` 명령을 제공하고 이 저장소의 collaborator 권한으로 같은 저장소
branch stack을 게시할 수 있다. 이 기능은 GitHub public preview이므로 게시 직전에 CLI 동작과 stack
map을 다시 확인한다. `gh stack submit`은 push와 PR 생성을 함께 수행하므로 저장소 승인 게이트를
별도로 통과한 뒤 실행한다. stacked PR은 auto-merge를 지원하지 않으므로 merge는 bottom부터 수동으로
진행한다.
