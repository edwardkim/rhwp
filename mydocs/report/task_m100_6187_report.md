# 최종 보고서 — #6187 눈금자 상시 표시와 resize 무깜빡임

- **이슈**: [#6187](https://github.com/edwardkim/rhwp/issues/6187)
- **대체 대상**: [PR #6432](https://github.com/edwardkim/rhwp/pull/6432)
- **작업 브랜치**: `codex/issue-6187-always-visible-ruler`
- **최초 기준**: `upstream/devel@e50792c6341a0b61afc3ffeb687a92fc6a807e69`
- **PR 준비 기준**: `upstream/devel@0d15409319c0bcaec71bb85a061090b637b9e4f2`
- **완료일**: 2026-09-01 KST
- **판정**: PR #6570 생성·code candidate CI 성공·maintainer self-review 승인, trailing 기록 검증·정상 merge 대기

## 1. 결과

편집 화면의 가로·세로 눈금자와 교차 코너를 viewport 너비·높이와 무관하게 항상 표시한다. 인쇄 시
숨김은 유지한다. 좁은 화면에서도 mouse/trackpad pointer는 기존 여백·들여쓰기 핀을 조작할 수 있고,
touch·pen·알 수 없는 pointer는 눈금자를 읽을 수만 있으며 문서 서식 commit을 만들지 않는다.

resize 이벤트는 canvas backing bitmap을 즉시 초기화하지 않고 갱신만 예약한다. 예약 callback의 한
`update()` 안에서 두 축의 최신 container 크기와 DPR을 읽고, 달라진 bitmap 크기만 맞춘 뒤 가로·세로
눈금자를 모두 paint한다. bitmap reset과 repaint 사이의 빈 프레임을 만드는 기존 순서를 제거했다.

## 2. 단계와 commit

| 단계 | commit | 결과 |
| --- | --- | --- |
| 계획 | `ddd9fe37d` | 독립 구현·단일 PR·상시 표시 및 입력별 조작 정책 확정 |
| Stage 1 | `a24b353b2` | 상시 표시 CSS와 mouse-only pointer drag 계약 |
| Stage 2 | `35a1e4a63` | resize와 paint의 동일 갱신 원자화 |
| Stage 3 | `84ead42c9` | 전체 회귀·177개 browser snapshot·실제 mouse drag/undo 증적 |
| 최신 devel 통합 | `7d4f4a18f` | `upstream/devel@0d1540931` 충돌 없는 통합 |

단계별 근거는 [수행 계획](../plans/archives/task_m100_6187.md),
[구현 계획](../plans/archives/task_m100_6187_impl.md), [Stage 1](../working/archives/task_m100_6187_stage1.md),
[Stage 2](../working/archives/task_m100_6187_stage2.md), [Stage 3](../working/archives/task_m100_6187_stage3.md)에 있다.

## 3. 검증

| 게이트 | 결과 |
| --- | --- |
| 최신 통합 head TypeScript | 통과 |
| 최신 통합 head Studio 전체 test | 1350 passed / 0 failed / 1 skipped (총 1351) |
| Stage 3 browser snapshot | 177개 통과 |
| Stage 3 실제 mouse 조작 | 767px 여백·들여쓰기 drag, commit·undo·수치 입력 통과 |
| 사용자 실제 OS 창 드래그 | resize 깜빡임 제거와 상시 표시 확인·승인 |
| 최신 통합 head browser smoke | 767px·1024px 두 눈금자/교차 코너/20px grid, warning·error 0건 |
| E2E manifest | Stage 3 당시 tracked 123개 / manifest 123행, 이상 없음 |

기존 skip 1개는 `pkg-node/rhwp.js`가 필요한 `pending-char-shape.test.ts`의 자식 프로세스·WASM 왕복
테스트다. #6187 변경과 무관하며 통과로 세지 않았다. 이번 PR diff에는 Rust·WASM·조판 변경이 없어
Cargo, WASM build와 한컴 PDF 시각 sweep은 적용하지 않는다.

실제 touch/pen, 기존 E2E 원본 자동 실행, 모든 compositor frame의 직접 관측은 수행하지 못했다.
Node 입력 계약 23개, resize 동작 회귀, snapshot·mouse 조작과 사용자의 native 창 드래그를 함께 근거로
작업지시자가 Stage 3를 승인했다. 제한은 [Stage 3 보고](../working/archives/task_m100_6187_stage3.md)에 그대로 남긴다.

## 4. 사용자 확인에서 분리한 후속 결함

1. 세로 눈금자의 마지막 번호 `42`는 가로 마지막 번호 숨김 정책과 대칭으로 처리한다. 해당 정책을
   소유한 PR #6458의 보정 commit 범위이며 #6187에 넣지 않는다.
2. macOS Firefox의 10%·13%·14% 트랙패드 pinch가 브라우저 zoom으로 이탈하는 문제는 #6187이
   변경하지 않은 `ViewportManager`의 wheel listener 범위에서 시작된 별도 결함이다. 실제 Firefox
   event trace를 추가한 독립 이슈·PR로 처리한다.

두 항목은 #6187의 직접 회귀가 아니며, 사용자는 이를 분리한 상태로 Stage 3 결과를 승인했다.

## 5. 원 PR과 출처

PR #6432는 #6187의 반응형 눈금자 문제를 먼저 다룬 선행 시도다. 그러나 767px 이하에서 눈금자를
숨기는 정책이 최종 제품 원칙과 달랐고 resize blank frame의 원인 검증도 포함하지 않았다. 이번 branch는
PR #6432 commit을 cherry-pick하지 않고 최신 devel에서 독립 구현했다. 새 PR 본문에는 선행 시도와 대체
관계를 명시하되 체리픽 통합으로 표현하지 않는다. PR #6432 종료는 새 PR merge 뒤 별도 승인으로 처리한다.

## 6. PR 제목과 본문 초안

제목:

```text
fix(studio): 눈금자를 항상 표시하고 resize 깜빡임을 제거한다
```

본문:

```markdown
## 요약

- 편집 화면의 가로·세로 눈금자와 교차 코너를 모든 viewport에서 표시합니다.
- mouse/trackpad pointer는 좁은 화면에서도 핀을 조작할 수 있고, touch·pen 입력은 읽기 전용으로 둡니다.
- resize 이벤트에서 canvas를 먼저 비우지 않고, 한 갱신 안에서 크기 동기화와 두 축 paint를 완료합니다.

## 검증

- `npm test`: 1350 passed / 0 failed / 1 skipped
- `npx --no-install tsc --noEmit`: 통과
- 10/50/100% × 세로/가로, 1023↔1024 왕복을 포함한 browser snapshot 177개 통과
- 767px 실제 mouse 여백·들여쓰기 drag/undo와 수치 입력 통과
- 실제 OS 창 드래그에서 눈금자 상시 표시와 resize 무깜빡임 확인
- 최신 devel 통합 뒤 767px·1024px browser smoke와 warning/error 0건 확인

## 출처와 후속 범위

- PR #6432의 선행 시도를 참고했지만 commit을 가져오지 않고 확정된 정책과 원인 검증으로 독립 구현했습니다.
- 세로 끝 라벨 대칭 보정은 PR #6458, macOS Firefox pinch 이탈은 별도 후속 이슈로 분리합니다.

Supersedes #6432

Closes #6187
```

사용자 승인 뒤 [PR #6570](https://github.com/edwardkim/rhwp/pull/6570)을 생성했다. code candidate
`88ca4d1bfd1d766aa6e0ff8b426576a285daa443`의 CI·CodeQL·Canvas visual diff·Proptest·Adapter
inter-diff가 모두 성공했고, maintainer self-review 판정은 `승인`이다. review·오늘 작업 기록만 추가한
trailing commit의 fast-pass와 최신 `MERGEABLE`·`CLEAN` 확인 뒤 정상 merge한다. PR #6432 종료와
이슈·PR comment는 #6570의 실제 merge 뒤 수행한다.
