# Task M100-4121 Stage 2 완료보고서 — HF 선택 생성과 반복 페이지 투영

## 결과

머리말/꼬리말 편집 모드에 독립적인 논리 선택 상태를 추가하고 마우스·Shift 키보드 입력을
연결했다. 같은 HF 정의가 표시되는 현재 viewport의 모든 페이지에는 선택 사각형을
투영하며, 화면 밖 페이지는 스크롤로 visible 상태가 될 때 같은 선택을 다시 그린다.

이번 Stage는 선택의 생성과 표시까지만 연결한다. 선택 범위 삭제·입력 치환·복사·잘라내기·
부분 서식·Undo/Redo 선택 복원은 Stage 3 범위이므로 #4121 전체가 해결된 상태는 아니다.

## 상태 모델과 불변식

- `Cursor`에 본문·각주 anchor와 독립된 `hfAnchor`를 추가했다.
- 논리 위치는 `(sectionIdx, isHeader, applyTo, paraIdx, charOffset)`으로 소유한다.
- anchor와 focus가 같은 `(sectionIdx, isHeader, applyTo)`일 때만 선택 범위를 반환한다.
- 역방향 선택은 `(paraIdx, charOffset)` 사전식 순서로 정렬한다.
- target 전환·본문 진입·HF 종료와 명시적 선택 해제는 HF anchor도 함께 지운다.
- `preferredPage`는 표시·수직 이동 기준일 뿐 논리 범위의 소유자가 아니다.
- HF 수직 이동은 현재 caret 좌표와 HF 전용 hit-test를 사용하며 같은 target 안에서만
  focus를 옮긴다.

## 마우스·키보드 동작

- HF 텍스트에서 일반 클릭과 실제 pointer drag로 단일·다문단 선택을 만든다.
- `Shift+클릭`, `Shift+Left/Right/Up/Down/Home/End`가 현재 HF 캐럿에서 선택을 확장한다.
- 선택이 있는 상태에서 일반 `Esc`는 선택만 해제하고 HF 모드를 유지한다.
- 선택이 없는 `Esc`와 `Shift+Esc`는 기존 동작대로 HF 모드를 종료한다.
- 다른 Odd/Even target을 클릭하면 교차 선택을 만들지 않고 target을 안전하게 전환한다.
- 본문 클릭은 의도한 rhwp 동작대로 HF 모드와 선택을 끝낸다. 한글 2024의 본문 클릭 차단
  또는 강제 포커스 고정은 복제하지 않는다.

## 반복 페이지 overlay

- HF 선택이 있으면 `VirtualScroll.getVisiblePages()`가 반환한 페이지에만 Stage 1의
  `getSelectionRectsInHeaderFooter`를 호출한다.
- Stage 1 target 검증 때문에 같은 `Both`/`Odd`/`Even` 정의를 쓰는 페이지에만 사각형이
  생기며 다른 정의의 페이지는 빈 결과를 반환한다.
- viewport scroll·resize 때 선택을 재계산한다. scroll 이벤트는 기존
  `ViewportManager`의 animation-frame coalescing을 그대로 사용한다.
- `SelectionRenderer`의 페이지 x 좌표를 단일 열 중앙 정렬 추정식에서
  `VirtualScroll.getPageLeftResolved()`로 바꿔 facing/grid 배치도 같은 좌표 진실원을 쓴다.
- 선택을 표시하기 위해 화면 밖 페이지를 eager layout하거나 render하지 않는다.

## RED/GREEN 회귀

초기 focused test 6건은 HF anchor API, 마우스/키보드 생성 경로, 반복 페이지 overlay가 없어
모두 실패했다. 구현 중 다중 페이지 page-left 회귀 1건을 추가해 최종 7건으로 고정했다.

1. target 소유 HF anchor와 동적 ordered range
2. 역방향 다문단 범위 정렬
3. 같은 target 안의 수직 이동
4. 마우스 target 식별과 drag lifecycle
5. Shift 키보드 선택과 Esc 2단계
6. visible page별 선택 기하 및 scroll 재계산
7. resolved page-left를 사용하는 selection renderer

## 실제 Chrome E2E

`biz_plan.hwp`의 6페이지 문서에서 반복되는 같은 HF target을 준비하고 다음 사용자 여정을
headless Chrome으로 실행했다.

1. 머리말 텍스트를 실제 마우스로 드래그해 논리 선택 생성
2. `Esc` 한 번으로 HF 모드는 유지하고 선택만 해제
3. 실제 `Shift+클릭`과 `Shift+End`로 선택 확장
4. 시작 페이지의 overlay 확인
5. 같은 HF 정의의 화면 밖 페이지로 스크롤해 논리 선택 유지와 새 overlay 확인
6. 시작 페이지로 돌아와 같은 선택 overlay 재표시 확인

12개 판정이 모두 통과했다. E2E HTML 보고서는 로컬 `output/e2e` 산출물이며 source
commit에는 포함하지 않는다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| Stage 2 focused Studio test | 7개 통과, 실패 0 |
| Stage 1 focused Rust integration 재검증 | 6개 통과, 실패 0 |
| Stage 1 Studio bridge + mutation routing 재검증 | 14개 통과, 실패 0 |
| Studio 전체 `npm test` | 1,247개 통과, 실패 0, 기존 skip 1 |
| Studio `npm run build` | TypeScript·Vite build 통과, 239 modules |
| 실제 Chrome `npm run e2e:issue-4121` | 12개 판정 통과 |
| `npm run e2e:manifest-check` | tracked 121 / manifest 121, 통과 |
| `cargo fmt --all`, `cargo fmt --all -- --check`, `git diff --check` | 통과 |

## Stage 3 경계

아직 다음 선택 소비자와 history 계약은 연결하지 않았다.

- `Backspace`/`Delete`/cut의 선택 범위 원자 삭제
- typing·IME·평문 paste의 선택 범위 원자 치환
- HF 선택의 copy와 다문단 평문 구성
- 선택 범위에만 적용되는 부분 글자 서식
- 연산별 Undo/Redo 뒤 HF target·caret·선택 복원

따라서 Stage 2만으로 #4121을 닫지 않는다. 이 변경을 독립 체크포인트로 고정한 뒤 별도
승인을 받아 Stage 3을 시작한다.
