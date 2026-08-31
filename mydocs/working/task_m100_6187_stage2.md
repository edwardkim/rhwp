# Stage 2 보고 — Task M100 #6187

- Issue: [#6187](https://github.com/edwardkim/rhwp/issues/6187)
- 작성일: 2026-08-31 KST
- 상태: resize 갱신 수정·focused 검증 완료. Stage 2 결과 및 Stage 3 착수 승인 대기.
- 브랜치: `codex/issue-6187-always-visible-ruler`
- 수정 전 기준: Stage 1 commit `a24b353b2`
- 수행 계획: [task_m100_6187.md](../plans/task_m100_6187.md)
- 구현 계획: [task_m100_6187_impl.md](../plans/task_m100_6187_impl.md)
- 앞 단계: [Stage 1 보고](task_m100_6187_stage1.md)

## 1. 승인과 범위

Stage 1 결과 보고·commit 뒤 작업지시자의 `진행해줘.`를 Stage 1 결과 및 Stage 2 착수 승인으로
기록했다. 이번 단계는 눈금자 내부의 bitmap 크기 갱신·paint 수명주기와 focused 회귀 테스트에 한정했다.

제품 코드 변경은 `rhwp-studio/src/view/ruler.ts` 한 파일이다. Stage 1의 표시·입력 정책은 유지했다.
`ViewportManager`의 ResizeObserver→setTimeout 정책, `CanvasView`의 배치·anchor·focus,
Rust/WASM, 문서 파일은 변경하지 않았다. 원 PR #6432의 commit도 가져오지 않았다.

## 2. 수정 전 실패 재현

실제 `Ruler`와 `EventBus`를 실행하면서 canvas width/height setter, CSS 크기 대입, 배경 paint,
transform과 rAF 실행 경계를 계측했다. canvas의 같은 크기 재대입도 그림을 지우는 동작은
[HTML canvas 표준](https://html.spec.whatwg.org/multipage/canvas.html#the-canvas-element)에 맞춰
모의 canvas setter에 구현했다. 이는 실제 브라우저 pixel 검사와는 구분되는 동작 테스트다.

구 구현은 다음 순서였다: resize 이벤트에서 네 width/height를 무조건 대입 → `scheduleUpdate()` →
다음 rAF에서 가로·세로 paint. 이벤트 callback이 끝났지만 다음 paint는 아직 실행하지 않은 시점에
두 bitmap이 지워져 있었다.

제품 코드 수정 전 resize 시나리오 17개를 실행했고 **2 passed / 15 failed / 0 skipped**를 확인했다.

| 관측 | 수정 전 결과 |
| --- | --- |
| resize 이벤트 직후, 다음 rAF 실행 전 | 기존 숫자·눈금이 지워져 공백 상태 검사 실패 |
| 동일 크기 resize 10회 | bitmap 차원 대입 40회(가로·세로 각 width/height), 불필요한 초기화 |
| 연속 이벤트 뒤 실제 container 크기가 더 변경됨 | 이벤트 때 읽은 옛 크기가 남음 |
| 이벤트 후 DPR 변경 | backing 크기와 paint transform이 서로 다른 DPR을 사용 |
| 문서 재열기 | 같은 크기에서도 bitmap을 먼저 초기화 |

이 결과는 bitmap 초기화→비동기 repaint 공백이라는 코드 경로를 재현한 증거다. 첨부 영상에 나온
모든 합성 프레임의 원인이 이것 하나라고 확정하거나 실제 창에서 깜빡임이 완전히 사라졌다고
판정하는 증거는 아니다. 그 최종 검증은 Stage 3에서 수행한다.

## 3. 수정 구조

- resize와 document-view-loaded 이벤트는 다른 repaint 이벤트처럼 갱신만 예약한다.
- 생성자도 같은 초기 갱신을 예약한다. 크기만 설정하고 외부 이벤트를 기다리는 경로를 없앴다.
- 공통 `update()`에서 DPR을 한 번 읽고 `syncCanvasSize(dpr)` → 가로 paint → 세로 paint를
  동기적으로 실행한다. 크기 변경과 paint 사이에 rAF·timeout·await를 추가하지 않는다.
- 두 축 container 크기를 먼저 읽고, 현재 backing 크기와 다른 차원만 대입한다.
  CSS width/height 역시 값이 바뀔 때만 쓴다.
- 별도 dirty flag 대신 모든 repaint 직전에 크기를 비교한다. 연속 이벤트가 합쳐졌거나
  다른 repaint와 동시에 크기/DPR이 바뀌어도 최신 상태를 읽는다.
- 기존 `resize()`는 외부 호출이 없음을 검색으로 확인하고, paint 직전 전용 private helper로 바꿨다.
- 두 축 draw는 크기 계산에 쓴 같은 DPR을 받는다. 기존 쪽 선택·눈금 간격·핀 좌표 계산은 유지한다.
- 동일 크기에서도 paint 자체는 실행한다. 문서 내용·테마·scroll·zoom만 바뀐 갱신을 건너뛰지 않는다.

따라서 이벤트 직후에는 마지막 그림을 유지하고, 다음 한 번의 갱신이 끝나면 최신 크기의 두 눈금자가
함께 준비된다. 서로 다른 프레임에 걸쳐 이벤트가 계속 와도 같은 계약을 적용한다.

## 4. 테스트 변경과 결과

- `tests/ruler-resize.test.ts`: 실제 Ruler 동작 시나리오를 자식 Node에서 실행하고 실패·skip을 판정한다.
- `tests/support/ruler-resize.cases.mjs`: 공백 경계, 동일 크기, 연속 이벤트, 축별 크기, 소수 DPR,
  초기 생성, 0 크기→문서 로드, 문서 전환, focus·scroll·zoom, dispose를 검증한다.
- `tests/support/ruler-harness.mjs`: Stage 1 harness에 bitmap setter·paint·프레임 계측을 추가했다.
  입력 23개 시나리오도 다시 실행해 harness 보강과 갱신 변경의 무회귀를 확인했다.
- `tests/ruler-document-load-refresh.test.ts`: 직접 `resize()` 호출을 요구하던 문자열 검사를 새 예약
  경로로 정정했다. 실제 크기와 눈금 갱신 여부는 새 동작 테스트로 검증한다.

최초 17개가 수정 후 전부 통과했다. 이후 초기 생성 테스트가 다른 focus/문단 이벤트에 의존하지 않게
강화하고, 40개 연속 프레임의 resize 및 DPR 단일 읽기 시나리오를 추가했다. 최종 resize **19개**와
기존 입력 **23개**가 모두 통과했다. 추가·강화한 최종 검사 전체를 구 구현에서 재실행한 것은 아니다.

`rhwp-studio`에서 실행한 최종 명령:

```sh
node --test \
  tests/ruler-visibility.test.ts tests/ruler-input.test.ts tests/ruler-resize.test.ts \
  tests/ruler-pin-geometry.test.ts tests/ruler-scale.test.ts \
  tests/ruler-document-load-refresh.test.ts tests/active-page.test.ts \
  tests/active-page-integration.test.ts tests/responsive-toolbar-layout.test.ts \
  tests/mutation-routing-guard.test.ts tests/theme-skin.test.ts
node --experimental-transform-types --no-warnings --test --test-reporter=spec \
  tests/support/ruler-resize.cases.mjs tests/support/ruler-input.cases.mjs
npx --no-install tsc --noEmit
```

| 검사 | 결과 |
| --- | --- |
| focused 테스트 11개 파일 | 57 passed / 0 failed / 0 skipped |
| resize·입력 내부 시나리오 직접 실행 | 42 passed(19+23) / 0 failed / 0 skipped |
| TypeScript | 통과 |
| `python3 scripts/check_markdown_links.py` — 수행·구현 계획, Stage 1·2 보고서 | 통과: 문서 4개 내부 상대 링크 이상 없음 |
| `git diff --check` | 통과 |

42개 내부 시나리오는 위 57개 중 두 parent 테스트가 실행하는 하위 검사이며, 총 테스트 수에 더하지
않는다. 자식 process 테스트는 프로젝트 EPERM 방지 규칙에 따라 sandbox 밖에서 실행했다.
검증한 제품·테스트 코드는 이 보고서와 함께 commit한다.

## 5. 제한과 다음 승인 게이트

- 실제 browser layout/compositor, native 창 드래그, 모바일 장치, 공개 sample의 연속 화면은 아직
  검증하지 않았다. 이번 Node 테스트를 시각적 최종 해결의 증거로 대체하지 않는다.
- Stage 1에서 추가한 반응형 E2E의 실제 실행, resize/input E2E 작성·실행, 문서 전환 E2E는 Stage 3다.
- 전체 `npm test`는 실행하지 않았다. Stage 3 통합 검증 승인 뒤 실행한다.
- Rust 변경이 없어 Cargo 검사·WASM 재빌드는 수행하지 않았다.
- 로컬 서버의 최종 후보 확인과 사용자 직접 검증용 URL 전달도 Stage 3에서 수행한다.
- remote push, PR 생성, GitHub comment/review, 기존 PR/이슈 종료는 수행하지 않았다.

Stage 2 결과를 보고·commit하고 다음 승인을 기다린다. 요청하는 다음 승인 범위는 **Stage 3 전체
Studio 테스트·실제 브라우저 검증·로컬 서버 확인**이다. 사용자 로컬 확인과 별도 승인 전에는
push나 새 PR 생성을 하지 않는다.
