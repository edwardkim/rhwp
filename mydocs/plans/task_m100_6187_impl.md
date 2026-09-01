# 구현 계획 — Task M100 #6187

- Issue: [#6187](https://github.com/edwardkim/rhwp/issues/6187)
- 작성일: 2026-08-31 KST
- 상태: PR #6570 생성·code candidate CI 성공·maintainer self-review 승인 / trailing 기록 검증·정상 merge 대기.
- 수행 계획: [task_m100_6187.md](task_m100_6187.md)
- Stage 1 결과: [상시 표시·입력 정책 구현 보고](../working/task_m100_6187_stage1.md)
- Stage 2 결과: [resize 갱신 원자화 보고](../working/task_m100_6187_stage2.md)
- Stage 3 결과: [전체 테스트·브라우저 검증 및 사용자 승인 기록](../working/task_m100_6187_stage3.md)
- 최종 결과: [눈금자 상시 표시와 resize 무깜빡임 보고](../report/task_m100_6187_report.md)
- 브랜치: `codex/issue-6187-always-visible-ruler`
- 기준: `upstream/devel@e50792c6341a0b61afc3ffeb687a92fc6a807e69`

## 1. 설계 경계

표시 여부, 조작 가능 여부, 그림 갱신 시점을 서로 다른 책임으로 유지한다.

- CSS는 눈금자의 20px grid 슬롯과 반응형 배치만 결정한다.
- 입력 처리는 실제 pointer 종류와 drag 수명주기를 판단한다. CSS breakpoint로 조작 권한을 정하지 않는다.
- 눈금자 renderer는 최신 geometry를 읽어 크기 변경과 가로·세로 paint를 한 갱신 안에서 끝낸다.
- 문서 여백/들여쓰기 변경은 기존 `onCommitPin` 경로를 유지한다. WASM 직접 변경 경로를 추가하지 않는다.

## 2. Stage 1 — 상시 표시와 입력 계약

### 반응형 CSS

대상: `rhwp-studio/src/styles/responsive.css`, 필요 시 `src/styles/editor.css`.

1. 최신 devel의 1023px 및 767px 이하 눈금자 숨김과 grid→flex 전환을 새 상시 표시 계약에 맞게
   제거한다. 원 PR commit이나 구현 파일을 체리픽하지 않는다.
2. 낮은 화면 높이 + 모바일 조건의 `#editor-area` flex override도 제거한다. 도구 영역 축약 자체는 유지한다.
3. `20px minmax(0, 1fr)` 행·열 계약을 유지하고 canvas의 intrinsic size가 track 축소를 방해하지 않는지
   확인한다. 추가 `min-width/min-height`는 실제 overflow 근거가 있을 때만 최소 범위로 적용한다.
4. scroll-container의 기존 touch-action과 스크롤 동작을 보존한다.
5. 인쇄 media의 눈금자 숨김·출력용 flex는 그대로 둔다. 주석에서 CSS 수정만으로 resize 깜빡임까지
   해결됐다는 주장을 제거한다.

### 실제 입력 종류로 조작 분기

대상: `rhwp-studio/src/view/ruler.ts`; 필요하면 작은 순수 입력 판정 helper를 같은 view 모듈에 둔다.

현재 `mousedown/mousemove/mouseup` 경로는 입력 출처를 구분하지 않는다. touch 뒤 호환 mouse event가
발생해 문서 서식을 바꾸지 않도록 pointer event 경로로 일원화하는 설계를 제안한다.

- 마우스·트랙패드(`pointerType: mouse`)의 주 포인터/왼쪽 버튼만 핀 drag를 시작한다.
- 손가락(`touch`)은 hover/drag/commit을 시작하지 않는다. 별도 펜 UX는 범위 밖이며 이 작업의
  기본 허용 목록은 mouse로 한정한다. 펜/미확인 종류는 읽기 전용으로 두는 것을 승인 대상으로 제안한다.
- `pointerId`를 drag 상태와 연결해 다른 손가락·pointer의 이동/해제가 진행 중인 drag를 commit하지 않게 한다.
- `pointercancel`, 필요 시 capture 상실·창 blur·dispose에서 drag와 listener를 정리하고 commit하지 않는다.
- 실제 이동 없는 클릭은 commit하지 않는 기존 규칙을 유지한다. 드래그 완료는 기존 좌표 계산·clamp·
  `onCommitPin`만 사용한다.
- 입력 전환 때 갱신되는 hover cursor를 적용한다. 읽기 전용이라는 이유로 숫자나 표시 핀을 흐리게 하지 않는다.
- touch 입력에 `preventDefault()`나 전역 `touch-action: none`을 적용하지 않는다. 문서 스크롤·pinch를
  막지 않으며 눈금자에서 시작한 제스처를 문서로 전달하는 신규 gesture 시스템도 만들지 않는다.

### Focused 테스트

- 새 상시 표시 계약 테스트를 작성한다. 원 PR에만 있는 `tests/narrow-ruler-policy.test.ts`는
  가져오지 않는다. CSS 정규식은 보조 검사이며 실제 작은 viewport 검증을 대신하지 않는다.
- 신규 입력 동작 테스트로 touch/pen 거부, mouse 허용, pointer 혼입, cancel/dispose, 클릭 무변경과
  drag 한 번당 commit 한 번을 검증한다.
- 기존 `ruler-pin-geometry`, `ruler-scale`, `active-page` 관련 focused test를 함께 실행한다.
- devel의 `e2e/responsive.test.mjs`에 모든 편집 화면 너비에서의 눈금자 표시 검증을 추가하고
  767/768, 1023/1024 및 낮은 높이 조건을 보강한다. 전체 실제 브라우저 검증은 Stage 3에서 실행한다.

## 3. Stage 2 — resize의 bitmap 초기화·paint 원자화

대상: `rhwp-studio/src/view/ruler.ts`, `tests/ruler-document-load-refresh.test.ts`, 신규 동작 테스트.

### 수정 전 실패를 고정

컨테이너·canvas·event bus·rAF를 제어하는 테스트로 실제 Ruler 갱신 경로를 실행한다.
기존 테스트의 소스 문자열 매칭만으로 순서를 증명하지 않는다. 필요한 타입 import의 명시와 상대
경로 정리 등은 테스트 가능한 경계를 만드는 최소 변경으로 한정한다.

1. 눈금자를 한 번 그린 뒤 `viewport-resize`를 발생시킨다.
2. 다음 draw가 실행되기 전 width/height 대입으로 기존 bitmap이 지워지는지 기록한다.
3. 동일 크기 이벤트의 불필요한 bitmap 초기화와 연속 이벤트의 예약 횟수를 기록한다.
4. 이 실패가 수정 뒤 통과하도록 하며, 구 구현을 통과시키는 테스트는 회귀 증거로 채택하지 않는다.

### 제안 갱신 구조

- resize/document-view-loaded 이벤트는 크기 갱신이 필요하다는 사실과 repaint를 함께 예약한다.
  이벤트 callback에서 먼저 backing bitmap을 지우지 않는다.
- 한 번 예약한 갱신 callback에서 최신 컨테이너 geometry와 DPR을 읽고 필요한 크기만 적용한 뒤
  가로·세로 눈금자를 모두 그린다. bitmap 초기화와 paint 사이에 추가 rAF·timeout·await를 두지 않는다.
- canvas `width/height`는 계산한 물리 크기가 달라질 때만 대입한다. 동일 크기라도 대입하면
  bitmap/context 상태가 초기화되는 특성을 테스트로 고정한다.
- CSS 크기 대입도 불필요한 반복을 피한다. 소수 DPR·가로만/세로만 크기 변경을 검증한다.
- 최초 생성, 문서 재열기, 0 크기→유효 크기, theme/zoom/scroll, dispose 경로를 같은 갱신 계약으로 맞춘다.
- 가로·세로 축이 사용하는 focus/geometry 기준이 paint 중 서로 달라지지 않게 한다. resize는 문서 모델이나
  편집 focus를 변경하는 command가 아니다.

### 명시적으로 피하는 변경

- bitmap을 지운 뒤 긴 debounce 동안 빈 띠를 유지하거나, resize 동안 눈금자를 숨기는 우회.
- 이전 화면을 복사하는 이중 버퍼를 근거 없이 도입하는 변경.
- 전역 `ViewportManager`의 ResizeObserver→setTimeout 정책을 무조건 rAF로 바꾸는 변경.
- `CanvasView`의 anchor·reflow 순서나 #6149 LOD를 근거 없이 수정하는 변경.

Stage 2 구현에서는 별도 resize dirty flag 대신 공통 `update()`의 paint 직전에 항상 최신 크기를
비교한다. 두 축의 container 측정을 먼저 마친 뒤 달라진 bitmap/CSS 차원만 쓴다. 따라서 resize
예약과 다른 repaint 예약이 합쳐지거나 DPR이 바뀌어도 크기 갱신을 놓치지 않으며, 변화 없는
bitmap은 초기화하지 않는다. DPR은 갱신당 한 번 읽어 크기 계산과 두 축 paint에 공유한다.

눈금자 내부 수정 후에도 실제 화면에서 stale geometry/공백이 관측되면, 관측값과 필요한 추가 파일을
보고하고 설계를 재승인받는다. 전역 이벤트 순서 변경은 이 계획의 자동 승인 범위가 아니다.

### 회귀 테스트

- 같은 크기 resize에서 backing store 재설정 0회.
- 연속 resize가 한 갱신으로 합쳐지고 마지막 geometry를 사용함.
- bitmap 크기 변경과 두 축 그리기가 같은 callback에서 끝남.
- 문서 전환 이벤트로 새 문서의 크기·눈금을 갱신하며 과거 핀/문맥이 잔류하지 않음.
- dispose 뒤 예약 갱신과 전역 listener가 남지 않음.

## 4. Stage 3 — 통합 검증과 로컬 확인

### 자동 검증

Stage 2 결과 승인 뒤 아래 범위의 실행 승인을 받아 수행한다.

- `rhwp-studio`의 `npx --no-install tsc --noEmit`.
- `npm --prefix rhwp-studio test` 전체. 자식 process 테스트는 sandbox 밖에서 실행한다.
- 반응형 E2E와 새 눈금자 resize/input E2E, 문서 전환 E2E.
- E2E 파일 추가/변경 시 `e2e/MANIFEST.md`와 manifest 검사 결과 기록.
- `git diff --check`, 변경 문서 경로·링크 확인.

실제 브라우저 검증을 수행하는 시점에는 브라우저 스킬과 저장소 개발 환경/E2E 가이드를 먼저 읽고,
그 환경에서 허용된 실행 경로를 사용한다. 실행 가능한 자동 E2E와 수동 확인만 가능한 범위를 구분해
기록하고, 미실행 항목을 통과로 간주하지 않는다.

### 프레임 검증

신규 E2E 예정 파일은 `rhwp-studio/e2e/ruler-resize.test.mjs`다. 테스트 쪽 계측을 우선하고 제품
코드에 검증 전용 전역 API를 추가하지 않는다.

- 공개 sample을 로드하고 10/50/100% × 세로/가로 쪽 이동을 실행한다.
- 1023↔1024 각 10회 왕복과 더 넓은 연속 resize를 수행한다. 정착을 기다린 뒤 최종 상태만 읽는
  방식이 아니라 resize 중 bitmap/paint 상태와 샘플링 프레임을 관측한다.
- visibility와 canvas 크기가 정상이더라도 눈금 bitmap이 비어 있으면 실패다. 문서 없음/용지 전체가
  화면 밖인 정상 빈 영역과 구분되는 sample·focus·검사 영역을 선택한다.
- 프레임 샘플러 자체의 실행 순서로 생기는 미탐을 막기 위해 수정 전 후보에서도 실패 검출을 확인한다.
  rAF 픽셀 관측만으로 실제 합성 화면의 모든 프레임이 증명됐다고 주장하지 않는다.
- 실제 창 드래그 영상 또는 연속 화면 증거와 작업지시자의 육안 확인을 함께 사용한다.
- 기존 공개 sample `exam_kor.hwp`를 우선 사용하고 문서 수·마지막 편집 focus·눈금 기준 쪽·배율·
  쪽 이동 설정·outer overflow·console 오류를 비교한다.

### 표시·입력 무회귀

- 모바일·경계 너비와 낮은 높이에서 두 눈금자와 grid 슬롯을 확인한다.
- 좁은 데스크톱 마우스 drag로 실제 여백/들여쓰기 변경 및 undo를 확인한다.
- touch로 같은 위치를 조작해도 값/undo 항목이 바뀌지 않는지 확인하고, 같은 세션의 mouse로는
  조작 가능한지 검증한다. 모바일 에뮬레이션과 실제 모바일 검증을 구분한다.
- 밝은/어두운 테마, 문서 재열기, 스크롤·확대와 resize를 조합한다.
- 편집 용지·문단 모양의 기존 수치 변경 경로를 스모크 검증한다.

### 전달과 종료 게이트

1. 후보 코드가 실제로 제공되는 로컬 Vite 서버를 확인하고 URL과 검증 sample 링크를 전달한다.
2. 사용자가 창을 천천히/빠르게 양방향 드래그하고 모바일 크기·마우스 핀 조작을 확인할 수 있게 한다.
3. 보고서에는 실행 결과, 미실행/제한, 독립 구현 commit을 적는다. 원 PR은 참고 SHA와 대체 이유로만
   기록하고, 체리픽 적용 SHA나 외부 PR 보정으로 표현하지 않는다.
4. 원 영상은 로컬 진단 참고로만 사용한다. 공개 PR 증적은 공개 sample로 새로 생성해 개인 창/탭 정보가
   포함되지 않게 한다. 이번 변경은 문서 조판이 아닌 Studio UI이므로 한컴 PDF 전수 sweep을 요구하지 않는다.
5. 사용자 확인 전에는 ‘PR 준비 완료’나 ‘깜빡임 최종 해결’로 확정하지 않는다.
6. 결과 승인 뒤에도 push·PR 생성은 별도 게이트다. 원 PR은 새 PR merge와 종료 승인 전까지 유지한다.

### 2026-08-31 실행 체크포인트

전체 npm·TypeScript 및 지원되는 Browser API로 snapshot 177개, 마우스 핀 drag/undo,
수치 입력·새 문서·테마 스모크를 실행했다. 새 snapshot 검사와 부정 대조 4개를 추가했다.
제품 코드는 Stage 2 commit `35a1e4a63` 이후 변경하지 않았다.

현재 Browser 도구에는 native touch dispatch나 canvas context 읽기 기능이 없으므로, 실제 입력
계약과 전체 합성 프레임 검증을 완료했다고 간주하지 않는다. page global에 직접 쓰는 기존 E2E도
이 도구로 우회 실행하지 않았다. 상세 미실행 항목은 Stage 3 기록에 남겼고, 원 계획의 수용 기준을
snapshot 통과만으로 축소하지 않는다. 사용자에게 실제 창 드래그용 서버를 제공한 체크포인트다.

### 2026-09-01 사용자 승인과 PR 준비

작업지시자는 실제 OS 브라우저 창을 드래그해 이번 변경의 상시 표시와 resize 무깜빡임을 확인하고
Stage 3 결과를 승인했다. 확인 중 발견한 세로 끝 번호 `42`는 PR #6458의 가로 끝 라벨 정책에 대칭
보정하고, macOS Firefox 트랙패드 pinch 이탈은 별도 결함으로 추적하기로 했다. 두 항목을 #6187 구현에
추가하지 않는다.

PR 준비 승인에 따라 최신 `upstream/devel@0d1540931`을 merge commit `7d4f4a18f`로 통합했다. 통합 뒤
Studio 전체 npm 테스트 1350 passed / 0 failed / 1 skipped와 TypeScript 검사가 통과했다. 최신 head를
인앱 브라우저에서 다시 불러 767px·1024px의 두 눈금자·교차 코너·20px grid와 console 무오류를
확인했다. 최종 보고서와 PR 본문 초안을 작성해 별도 승인을 기다렸다.

### 2026-09-01 PR #6570 self-review

작업지시자의 “권장 순서대로 진행” 승인에 따라 `task_m100_6187` branch를 push하고 PR #6570을
생성했다. code candidate `88ca4d1bfd1d766aa6e0ff8b426576a285daa443`의 CI·CodeQL·Canvas visual
diff·Proptest·Adapter inter-diff가 모두 성공했고 GitHub 상태는 `MERGEABLE`·`CLEAN`이다.
`mydocs/pr/archives/pr_6570_review.md`에서 maintainer self-review 판정을 `승인`으로 확정했다.
review·오늘 작업 기록만 추가한 trailing commit은 fast-pass를 확인한 뒤 정상 merge하고, 그 뒤에만
#6187과 대체된 PR #6432를 정리한다.
