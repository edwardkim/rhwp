# Task M100 #661 Stage 1 완료보고서

## 단계 목표

구현 전 기준 상태를 고정하고, Issue #661의 재현 경로와 관련 PR/코드 경로를 확인한다.

## 수행 일시

- 2026-05-08

## 기준 브랜치 및 커밋

| 항목 | 값 |
|------|----|
| 작업 브랜치 | `local/task661` |
| 작업 브랜치 HEAD | `2fe386c` |
| upstream 기준 | `upstream/devel` |
| upstream/devel HEAD | `2fe386c` |
| PR #664 head | `b1b18c2` |
| PR #664 base | `2fe386c` |

`git fetch upstream` 후 확인 결과, `local/task661`은 여전히 `upstream/devel`과 동일한 커밋이다.

## GitHub 상태 확인

| 항목 | 상태 | 확인 내용 |
|------|------|-----------|
| Issue #661 | OPEN | 댓글 없음, 2026-05-08 04:11:08Z 갱신 |
| PR #664 | OPEN / MERGEABLE | base=`devel`, base oid=`2fe386c`, head=`b1b18c2` |
| PR #693 | OPEN / MERGEABLE | 그리드 모드 좌표 정정, base oid=`2fe386c` |

## PR #664와의 관계

Issue #661은 #658 수정 확인 중 발견된 follow-up 증상이다. 현재 `devel`에는 #664 변경이 아직 병합되지 않았다.

확인한 #664 변경 파일:

```text
rhwp-studio/src/engine/caret-renderer.ts
rhwp-studio/src/engine/input-handler-mouse.ts
rhwp-studio/src/engine/input-handler.ts
rhwp-studio/src/engine/selection-renderer.ts
src/document_core/queries/cursor_nav.rs
tests/issue_658_text_selection_rects.rs
examples/inspect_658_selection.rs
```

#664는 selection rect overflow를 고치면서 드래그 경량 경로를 추가한다.

- `input-handler-mouse.ts`: 드래그 중 `updateCaret()` → `updateCaretDuringDrag()`
- `input-handler.ts`: `updateCaretDuringDrag()` 신규 추가
- `caret-renderer.ts`: `updateLive()` 신규 추가
- `selection-renderer.ts`: div pool 재사용
- `cursor_nav.rs`: selection rect cursor bias 정정

따라서 #661 구현은 #664 변경을 선행 기준으로 삼아야 한다. 현재 `devel`만 기준으로 구현하면 Stage 2에서 #664의 `updateCaretDuringDrag()`와 같은 메서드를 재작성하게 되어 PR #664 병합 순서에 따라 충돌 위험이 커진다.

## 현재 devel 코드 경로

현재 `devel`의 드래그 경로:

- `rhwp-studio/src/engine/input-handler-mouse.ts:1099`
  - rAF throttle 사용
  - rAF 콜백 안에서 원본 `MouseEvent`를 `hitTestFromEvent(e)`로 재해석
  - `this.updateCaret()` 전체 경로 호출

확인한 코드 위치:

```text
input-handler-mouse.ts:1099  if (this.isDragging) {
input-handler-mouse.ts:1104    const hit = this.hitTestFromEvent(e);
input-handler-mouse.ts:1107    this.updateCaret();
```

현재 `updateCaret()` 경로:

```text
input-handler.ts:1505  this.scrollCaretIntoView(rect);
input-handler.ts:1507  this.updateSelection();
input-handler.ts:1508  this.emitCursorFormatState();
```

현재 caret 스크롤 경로:

```text
input-handler.ts:1812  private scrollCaretIntoView(...)
input-handler.ts:1822  if (caretDocY < scrollTop + margin) ...
input-handler.ts:1825  else if (caretDocY + caretHeight > scrollTop + viewHeight - margin) ...
```

현재 pageIndex mismatch 폴백:

```text
cursor.ts:681  if (this.rect && this.position.cursorRect &&
cursor.ts:682      this.rect.pageIndex !== this.position.cursorRect.pageIndex) {
cursor.ts:683    console.warn('[CursorState] 캐럿 페이지 불일치 ...');
cursor.ts:685    this.rect = { ...this.position.cursorRect };
```

현재 Rust cursor rect 페이지 후보 경로:

```text
cursor_rect.rs:33   let pages = self.find_pages_for_paragraph(section_idx, para_idx)?;
cursor_rect.rs:890  let pages = self.find_pages_for_paragraph(section_idx, parent_para_idx)?;
```

## PR #664 코드 경로

PR #664 기준 드래그 경로:

```text
input-handler-mouse.ts:1104  const hit = this.hitTestFromEvent(e);
input-handler-mouse.ts:1107  this.updateCaretDuringDrag();
```

PR #664 기준 `updateCaretDuringDrag()`:

```text
input-handler.ts:1540  const rect = this.cursor.getRect();
input-handler.ts:1544  this.caret.updateLive(rect, zoom);
input-handler.ts:1545  this.scrollCaretIntoView(rect);
input-handler.ts:1547  this.updateSelection();
```

#661의 본질 의심 지점은 이 `scrollCaretIntoView(rect)`가 드래그 중에도 남아 있는 부분이다.

## 재현 문서 구조 확인

명령:

```bash
cargo run --quiet --bin rhwp -- dump-pages samples/exam_social.hwp -p 1
```

결과 요약:

- 문서: `samples/exam_social.hwp`
- 페이지: 2/4 (`global_idx=1`, `section=1`, `page_num=2`)
- body area: `x=70.7 y=149.3 w=886.7 h=1215.1`
- 2단 구성

하단 좌측 후보:

```text
단 0
  Table pi=12 ci=0 1x1 411.9x202.2px wrap=TopAndBottom tac=true vpos=64058
  Table pi=13 ci=0 5x2 408.2x114.4px wrap=TopAndBottom tac=true vpos=80306
```

상단 우측 자료 박스 후보 (#658 대표 영역):

```text
단 1
  FullParagraph pi=15
  Table pi=16 ci=0 1x1 411.9x293.3px wrap=TopAndBottom tac=true vpos=4240
  Table pi=17 ci=0 5x2 408.2x225.5px wrap=TopAndBottom tac=true vpos=27316
```

## 대상 표 상세

하단 좌측 1x1 표 후보:

명령:

```bash
cargo run --quiet --bin rhwp -- dump samples/exam_social.hwp -s 1 -p 12
```

요약:

- 문단: `section=1`, `para=12`
- 표: `1행×1열`
- treat_as_char: `true`
- wrap: `위아래`
- size: `30894×15168 HU` (`109.0×53.5mm`)
- cell[0]: padding `(850,850,850,850)`
- cell paragraph: `p[0]`, `text_len=1131`
- line segments: 12개

```text
p[0] text_len=1131
ls[0] vpos=0
ls[1] vpos=1138
...
ls[11] vpos=12518
```

#658 대표 영역인 우측 1x1 표:

명령:

```bash
cargo run --quiet --bin rhwp -- dump samples/exam_social.hwp -s 1 -p 16
```

요약:

- 문단: `section=1`, `para=16`
- 표: `1행×1열`
- cell[0] paragraphs: 7개
- 대표 cell paragraph:
  - `p[0] text_len=209`, line segments 3개
  - `p[6] text_len=469`, line segments 6개

## Stage 1 판단

1. #661은 현재 `devel` 단독보다 PR #664 이후 상태에서 가장 정확히 다루는 것이 맞다.
2. 다만 PR #664는 아직 open 상태이므로, `local/task661`에서 직접 #664 전체를 합치는 것은 별도 승인 없이 진행하지 않는다.
3. Stage 2 구현은 다음 중 하나의 정책 결정이 필요하다.
   - 옵션 A: `devel` 기준으로 #664의 드래그 경량 경로 중 #661에 필요한 최소 변경만 함께 구현한다.
   - 옵션 B: 작업지시자 승인 후 `local/task661`을 PR #664 head 위에 스택하고 #661만 추가 수정한다.
4. 충돌/중복 관점에서는 옵션 B가 가장 명확하다. #661은 #664의 follow-up이므로 소스 diff도 작아진다.

## 권장 다음 단계

Stage 2는 옵션 B를 권장한다.

진행 방식:

1. `local/task661`을 `refs/remotes/upstream/pr/664` 위로 이동 또는 재생성한다.
2. Stage 2에서 `updateCaretDuringDrag()`에서 `scrollCaretIntoView(rect)`를 제거한다.
3. Stage 3에서 포인터 기반 auto-scroll을 추가한다.
4. Stage 4에서 rAF snapshot 정합화를 진행한다.

## 검증 상태

Stage 1은 기준 정리와 구조 확인 단계이므로 앱 빌드/테스트는 수행하지 않았다.

실행한 명령:

```bash
git fetch upstream
gh pr view 664 --repo edwardkim/rhwp --json ...
gh issue view 661 --repo edwardkim/rhwp --json ...
gh pr view 693 --repo edwardkim/rhwp --json ...
cargo run --quiet --bin rhwp -- dump-pages samples/exam_social.hwp -p 1
cargo run --quiet --bin rhwp -- dump samples/exam_social.hwp -s 1 -p 12
cargo run --quiet --bin rhwp -- dump samples/exam_social.hwp -s 1 -p 16
```

## 승인 요청

Stage 1을 완료했다.

다음 Stage 2 진행 전 결정이 필요하다.

- 권장: 옵션 B — `local/task661`을 PR #664 head(`b1b18c2`) 위에 스택한 뒤 #661 수정 진행
- 대안: 옵션 A — 현재 `devel` 기준에서 #664 필요 변경을 포함하여 #661 수정 진행

옵션 B로 Stage 2를 진행해도 되는지 승인 요청한다.
