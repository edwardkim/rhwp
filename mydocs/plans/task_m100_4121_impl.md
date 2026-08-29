# 구현계획서 — Task M100-4121: 머리말/꼬리말 텍스트 선택 완성

## 1. 문서 상태와 승인 경계

- GitHub Issue: [#4121](https://github.com/edwardkim/rhwp/issues/4121)
- 작업 브랜치: `codex/issue-4121-hf-selection`
- 기준: `upstream/devel` @ `f6a6bee8f3` (Stage 4 최종 검증 전 merge)
- 수행계획: `mydocs/plans/task_m100_4121.md` 승인 및 로컬 체크포인트
  `e0622c2c7`
- 현재 단계: Stage 4 자동 통합 검증 완료, 사용자 수동 확인 대기

이 문서가 승인되기 전에는 제품 소스를 수정하지 않는다. 승인 뒤에도 Hyper-Waterfall에
따라 각 Stage의 코드·테스트·완료보고서를 한 묶음으로 검토하고 커밋한 뒤 다음 Stage로
넘어간다. 원격 push와 PR 생성은 별도 승인을 받는다.

## 2. 확정 요구사항

### 2.1 선택과 표시

1. 머리말/꼬리말에서 마우스 드래그, `Shift+클릭`,
   `Shift+Left/Right/Up/Down/Home/End`로 단일·다문단 선택을 만든다.
2. 선택의 논리적 소유자는 `preferredPage`가 아니라 현행 HF 정의 식별자인
   `(sectionIdx, isHeader, applyTo)`와 그 안의 anchor/focus다.
3. 같은 HF 정의가 표시되는 현재 렌더 페이지에는 같은 선택을 즉시 투영한다.
   화면 밖 페이지는 렌더 영역에 들어올 때 투영한다.
4. `Both`, `Odd`, `Even`의 실제 active target이 다른 페이지에는 강조를 그리지 않는다.
5. 다중 페이지 가로 배열에서도 페이지별 좌표가 맞아야 한다. 선택 overlay의 페이지 x
   좌표는 중앙 정렬 공식이 아니라 `VirtualScroll.getPageLeftResolved()`를 단일 진실원으로
   사용한다.

### 2.2 포커스와 모드 전환

1. 같은 HF 정의가 투영된 다른 페이지를 일반 클릭하면 기존 선택을 해제하고 그 페이지를
   새 `preferredPage`로 삼는다.
2. 클릭한 페이지의 active target이 다른 `Odd`/`Even` 정의라면 교차 선택을 만들지 않고
   해당 target으로 안전하게 전환한 뒤 새 캐럿/드래그를 시작한다.
3. 본문 클릭은 HF 선택과 HF 모드를 끝내고 클릭한 본문 위치로 이동한다.
4. 선택이 있는 상태의 `Esc`는 선택만 해제하고 HF 모드를 유지한다. 선택이 없을 때의
   `Esc`, `Shift+Esc`, 상단 닫기 버튼은 HF 모드를 종료한다.
5. 한글 2024의 “다른 영역 클릭 차단”과 고정 포커스는 복제하지 않는다.

### 2.3 선택 소비자

1. `Backspace`/`Delete`/잘라내기는 선택 범위를 한 번 삭제한다.
2. 입력·IME 입력·평문 붙여넣기는 선택 범위를 한 번 치환한다. 붙여넣기의 줄바꿈은 HF
   문단 경계로 보존한다.
3. 부분 글자 서식은 선택 범위에만 적용하며 여러 문단을 지원한다.
4. 복사는 `text/plain`과 기존 fallback HTML을 제공하고 코어 내부 클립보드에도 선택
   조각을 기록한다. HF로 붙여넣을 때는 이번 이슈에서 평문을 사용한다.
5. 본문↔HF, Header↔Footer, 서로 다른 HF 정의 사이의 교차 선택은 만들지 않는다.

## 3. 상태 모델과 불변식

`Cursor`에 본문 `anchor`, 각주 `fnAnchor`와 독립된 `hfAnchor`를 추가한다.

```ts
type HeaderFooterTextPosition = {
  sectionIdx: number;
  isHeader: boolean;
  applyTo: number;
  paraIdx: number;
  charOffset: number;
};
```

다음 불변식을 유지한다.

```text
HF-1  hfAnchor는 HF 모드에서만 존재한다.
HF-2  anchor와 focus는 같은 (sectionIdx, isHeader, applyTo)에 속한다.
HF-3  정렬은 (paraIdx, charOffset)의 사전식 순서다.
HF-4  빈 범위는 선택이 아니며 anchor를 정리한다.
HF-5  target 전환, 본문 진입, HF 종료는 hfAnchor를 지운다.
HF-6  preferredPage 변경은 논리 범위를 바꾸지 않지만 일반 클릭은 표준 편집 동작대로
      기존 선택을 지운 뒤 새 캐럿을 만든다.
HF-7  선택 복원 전에 현재 HF 문단 수·문자 오프셋을 다시 검증하고 실패하면 선택 없이
      복원한다.
```

`Cursor.hasSelection()`과 `clearSelection()`은 세 선택 종류를 모두 다루되, 범위 조회는
본문·각주·HF별 타입 안전한 메서드로 분리한다. 기존 본문 호출부가 HF 좌표를 본문
`DocumentPosition`으로 오인하지 않도록 범용 `getSelectionOrdered()`의 반환 타입은 넓히지
않는다.

## 4. 코어·WASM API 계획

JavaScript 공개 이름은 기존 HF API의 인자 순서와 `applyTo` 숫자 계약을 따른다.

### 4.1 페이지 지역 선택 기하

```text
getSelectionRectsInHeaderFooter(
  sectionIdx, isHeader, applyTo, pageNum,
  startParaIdx, startOffset, endParaIdx, endOffset
) -> [{ pageIndex, x, y, width, height }]
```

- `get_selection_rects_in_footnote_native`의 run 교차 계산을 기준으로 구현한다.
- 먼저 `resolve_header_footer_target(pageNum, isHeader)`가 요청한
  `(sectionIdx, applyTo)`와 같은지 검증한다. 다르면 빈 배열을 반환한다.
- HF 문단 marker와 모델 문자 오프셋을 사용하고, 표시 문자열과 모델 문자열의 길이가
  다른 필드도 기존 HF caret/hit-test와 같은 offset 공간을 사용한다.
- `get_cursor_rect_in_header_footer_native`도 같은 target 검증을 적용해 `Odd`/`Even`
  preferred page에서 다른 정의의 좌표를 잘못 잡지 않게 한다.
- `hitTestInHeaderFooter` 결과에 resolved `sectionIndex`와 `applyTo`를 포함해 Studio가
  클릭 페이지의 target을 추측하지 않게 한다.

### 4.2 원자적 범위 치환

```text
replaceRangeInHeaderFooter(
  sectionIdx, isHeader, applyTo,
  startParaIdx, startOffset, endParaIdx, endOffset,
  replacementText
) -> { ok, paraIdx, charOffset }
```

- 역방향 범위를 정렬하고 모든 경계를 검증한 뒤 mutation을 시작한다.
- 같은 문단은 선택 구간만 교체한다.
- 다문단은 시작 문단 prefix와 끝 문단 suffix를 보존하고 중간 문단을 제거한 뒤,
  replacement의 정규화된 `\n`을 새 문단 경계로 삽입한다.
- 기존 `split_header_footer_paragraph_native`/merge 로직의 문단·char-shape·control 처리
  규칙을 재사용한다.
- dirty 표시, raw 무효화, reflow/pagination은 전체 연산 뒤 한 번 수행한다.
- 빈 replacement는 범위 삭제이며 입력·IME·붙여넣기 치환과 같은 primitive를 쓴다.

### 4.3 선택 복사와 부분 글자 서식

```text
copySelectionInHeaderFooter(...range) -> 기존 ClipboardData JSON 계약
getCharPropertiesInHeaderFooter(...paraIdx, charOffset) -> CharProperties
applyCharFormatInHeaderFooter(...range, propsJson) -> 결과 JSON
```

- 복사는 본문 `copy_selection_native`의 paragraph slice/구조 control 제거 규칙을 HF
  paragraph slice에 적용하고 문단 사이 평문을 `\n`으로 연결한다.
- 글자 서식은 각 문단의 교차 구간에만 적용하고 char-shape table 생성과 범위 변경을
  한 코어 호출로 완료한다.
- 선택이 없는 HF 캐럿에서 “다음 입력 글자 서식 예약”까지 확장하지 않는다. 이번 범위는
  선택된 기존 텍스트의 부분 서식과 그 속성 조회다.

## 5. Studio 연결 계획

### 5.1 마우스와 키보드

- HF hit-test 성공 시 anchor를 세우고 기존 text selection drag/autoscroll 수명주기를
  재사용한다.
- 드래그 update는 pointer의 page와 active HF target을 확인한다. 같은 정의일 때만 focus를
  갱신하고 다른 정의에서는 교차 범위를 만들지 않는다.
- `Shift+클릭`은 현재 HF anchor가 유효할 때 focus를 확장하고, target이 다르면 기존
  선택을 지운 뒤 새 target의 캐럿으로 시작한다.
- 키보드는 Shift가 눌린 첫 이동 전에 anchor를 만들고 이동 뒤 focus를 갱신한다.
  `Up/Down`은 preferred x와 현재 HF 페이지의 caret/hit-test를 사용해 인접 시각 줄을 찾는다.
  Shift 없는 이동은 기존 선택을 해제한 뒤 움직인다.

### 5.2 반복 페이지 overlay

- `updateSelection()`에 HF 분기를 본문/각주보다 먼저 추가한다.
- `VirtualScroll.getVisiblePages()`로 현재 렌더 후보를 얻고, 각 페이지에 페이지 지역 HF
  selection query를 호출해 한 `SelectionRenderer` update로 합친다.
- `preferredPage`가 현재 화면에 있으면 후보에 포함하되 화면 밖 페이지의 render tree를
  선택 때문에 미리 만들지는 않는다.
- `viewport-scroll`, 확대/축소, document view 변경, history jump, formatting refresh에서
  선택이 존재할 때 overlay를 다시 계산한다. 스크롤 이벤트는 animation-frame당 한 번으로
  합쳐 과도한 WASM 호출을 막는다.
- `SelectionRenderer`가 페이지 위치를 자체 계산하지 않도록 page-left resolver를 주입한다.
  본문·셀·각주 선택의 기존 단일/다중 페이지 좌표 회귀 테스트를 함께 둔다.

### 5.3 입력·클립보드·서식 라우팅

- HF 모드의 text input 분기에서 삽입보다 먼저 HF 선택 존재 여부를 확인한다.
- 선택 치환은 `replaceRangeInHeaderFooter`를 한 `SubmodeSnapshotCommand`로 실행한다.
  선택이 없는 단일 글자 입력·삭제·문단 분할/병합은 기존 정밀 command를 유지한다.
- copy/cut clipboard event는 HF 선택을 먼저 판별하고 새 copy API를 호출한다. cut은 copy가
  성공한 뒤 같은 range replacement primitive로 삭제한다.
- paste는 HF에서 `text/plain`을 읽어 줄바꿈을 정규화한 뒤 선택 치환 또는 캐럿 삽입으로
  보낸다. OS별 rich clipboard round-trip은 비범위다.
- toolbar/shortcut 글자 서식 dispatcher는 “HF 선택이 있는 경우”에만 새 범위 서식 경로를
  허용한다.

## 6. Undo/Redo 계약

`EditContext.headerFooter`에 `preferredPage`를 추가한다. history 복원은 target과 캐럿뿐
아니라 편집 표면도 되살린다.

기존 `selectionBefore()`는 본문 전용 구조를 구분 가능한 body/HF union으로 바꾸고,
서식처럼 redo 뒤에도 선택을 유지해야 하는 명령을 위해 `selectionAfter()`를 추가한다.
복원은 항상 현재 문서 범위를 검증하고 실패 시 선택을 비운다.

| 연산 | 실행 직후 | Undo | Redo |
|------|-----------|------|------|
| Delete/Backspace/cut | 시작점에 접힌 캐럿 | 삭제 전 HF 선택 복원 | 다시 삭제, 접힌 캐럿 |
| typing/IME/paste 치환 | 삽입 끝에 접힌 캐럿 | 기존 본문 계약처럼 내용·캐럿만 복원, 선택은 복원하지 않음 | 다시 치환, 접힌 캐럿 |
| 부분 글자 서식 | HF 선택 유지 | 내용과 같은 HF 선택 복원 | 변경 서식과 같은 HF 선택 복원 |

범위 치환과 범위 서식은 문단·char-shape resource를 함께 바꿀 수 있으므로 HF 문맥을 가진
snapshot command로 원자화한다. 스냅샷 operation descriptor에 선택 복원 metadata를
추가하거나 이를 캡슐화한 `SubmodeSelectionSnapshotCommand`를 둔다. 일반 키 입력마다
스냅샷을 만들지는 않는다.

## 7. Stage별 변경과 RED/GREEN 게이트

### Stage 1 — 코어 계약과 페이지 지역 기하

수정 후보:

```text
src/document_core/queries/cursor_rect.rs
src/document_core/commands/header_footer_ops.rs
src/document_core/commands/clipboard.rs
src/wasm_api.rs
rhwp-studio/src/core/wasm-bridge.ts
tests/cases/issue_4121_header_footer_text_selection.rs
mydocs/working/task_m100_4121_stage1.md
```

RED:

- 단일/다문단 HF selection rect query 부재
- 요청 페이지 target 불일치 시 잘못된 rect/caret 반환
- 다문단 delete/replace, copy, 부분 서식 API 부재

GREEN:

- Header/Footer × Both/Odd/Even의 target 일치/불일치 기하 계약
- field 표시 문자열과 모델 offset 계약
- 다문단 치환·복사·부분 서식 및 invalid range fail-closed
- WASM binding과 Studio bridge 타입 계약

Stage 1 완료보고서와 focused Rust test를 검토·커밋한 뒤 Stage 2 승인을 요청한다.

### Stage 2 — 선택 생성과 반복 페이지 투영

수정 후보:

```text
rhwp-studio/src/engine/cursor.ts
rhwp-studio/src/engine/input-handler.ts
rhwp-studio/src/engine/input-handler-mouse.ts
rhwp-studio/src/engine/input-handler-keyboard.ts
rhwp-studio/src/engine/selection-renderer.ts
rhwp-studio/tests/issue-4121-header-footer-selection.test.ts
rhwp-studio/e2e/issue-4121-header-footer-selection.test.mjs
mydocs/working/task_m100_4121_stage2.md
```

RED:

- HF anchor/정렬 범위와 drag/Shift 생성 경로 부재
- 반복 페이지 및 scroll-in 페이지 overlay 부재
- 다중 페이지 가로 배열에서 selection page-left 오계산

GREEN:

- mouse/Shift 단일·다문단 선택
- 같은 HF 정의의 visible pages 즉시 투영, scroll-in 재투영
- `preferredPage` 전환, 다른 target 교차 선택 차단, 본문 클릭 종료
- Esc 2단계와 Shift+Esc 강제 종료
- 기존 본문·셀·각주 overlay 좌표 회귀 0

Stage 2 완료보고서와 Studio test/E2E 증적을 검토·커밋한 뒤 Stage 3 승인을 요청한다.

### Stage 3 — 선택 소비자와 history 계약

수정 후보:

```text
rhwp-studio/src/engine/command.ts
rhwp-studio/src/engine/input-handler.ts
rhwp-studio/src/engine/input-handler-keyboard.ts
rhwp-studio/src/engine/input-handler-text.ts
rhwp-studio/tests/issue-4121-header-footer-editing.test.ts
rhwp-studio/e2e/issue-4121-header-footer-selection.test.mjs
mydocs/working/task_m100_4121_stage3.md
```

RED:

- HF selection 소비자가 본문 range API로 빠지거나 no-op
- 다문단 delete/typing/paste/format/copy/cut의 분리 history 또는 잘못된 context 복원
- format undo/redo 뒤 선택 소실

GREEN:

- 선택 삭제·입력/IME/평문 붙여넣기 치환·copy/cut·부분 글자 서식
- 연산별 표에 정의한 undo/redo selection 계약
- HF target/caret/preferredPage 복원과 invalid selection fail-closed
- 기존 본문 selection delete/typing history 계약 회귀 0

Stage 3 완료보고서와 focused test/E2E를 검토·커밋한 뒤 Stage 4 승인을 요청한다.

### Stage 4 — 통합 검증과 최종 보고

- Rust/Studio/WASM 전체 필수 게이트를 실행한다.
- 4페이지 `Both`와 홀짝 분리 Header/Footer 문서로 mouse/Shift, 다문단, scroll,
  delete/replace/format/copy/cut, undo/redo를 실제 브라우저에서 확인한다.
- 사용자 VDI 관찰과 rhwp의 의도적 차이(즉시 반복 페이지 repaint, 클릭 차단 없음)를 최종
  보고서에 기록한다.
- 산출물: `mydocs/report/task_m100_4121_report.md`와 필요한 E2E screenshot/video 증적.

## 8. 최종 검증 명령

변경 범위는 Rust document core/WASM + rhwp-studio이며 PDF/SVG 출력 geometry는 바꾸지
않는다. 따라서 PDF visual sweep/Native Skia 3종 대신 실제 브라우저 DOM overlay 증적을
필수로 한다. 구현 중 renderer/layout 출력이 바뀌면 이 판정을 철회하고 Native Skia 3종과
시각 sweep을 추가한다.

```bash
git diff --check
cargo fmt --all
cargo fmt --all -- --check
node scripts/rust-unit-test-tiers.mjs --check
node scripts/run-rust-test.mjs issue_4121_header_footer_text_selection -- \
  --cargo-profile release-test --target-dir target/pr-review
cargo clippy --locked --all-targets -- -D warnings
cargo nextest run --locked \
  --cargo-profile release-test --target-dir target/pr-review \
  --tests --no-fail-fast
CARGO_TARGET_DIR=target/pr-review \
  scripts/wasm-pack-locked.sh --target web --out-dir pkg
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
```

- 같은 `target/pr-review`를 쓰는 Cargo 명령은 직렬 실행한다.
- 새 integration source는 `tests/cases/`만 commit한다. generated suite/manifest는 source
  branch에 stage하지 않는다.
- review 전용 worktree에서는 `rust-test-suite-manifest.mjs --prepare`와 `--check`를 수행해
  전체 회귀에 새 source를 포함하고 파생 변경을 stage하지 않는다.
- Studio E2E는 실제 browser에서 별도로 실행하고 viewport 크기, zoom, page layout과 결과
  screenshot을 보고서에 남긴다.

## 9. 비범위와 후속 분리 기준

- 한글 2024의 HF dialog/모양 갤러리/전용 ribbon 복제
- HF 모드에서 본문·다른 target 클릭을 전면 차단하는 focus lock
- OS별 rich clipboard 서식 round-trip
- 선택이 없는 HF 캐럿의 다음 입력 글자 서식 예약
- HF 내부 표·그림·개체 블록 선택
- 화면 밖 모든 페이지의 eager layout

위 항목이 #4121 완료에 필수라는 새 근거가 나오면 구현 중 조용히 범위를 넓히지 않고
별도 이슈 또는 수행계획 수정으로 돌린다.

## 10. 리스크와 되돌리기

| 리스크 | 방어 | 되돌리기 경계 |
|--------|------|----------------|
| 반복 페이지마다 query해 scroll 비용 증가 | visible pages만 조회, RAF coalescing, 선택 없으면 0회 | Stage 2 overlay 구독 commit |
| 홀짝 target 오판으로 다른 HF 편집 | query/hit/caret 모두 resolved target 검증 | Stage 1 target validation commit |
| 다문단 치환에서 char shape/control 손실 | 기존 split/merge 규칙 재사용, Rust round-trip test | Stage 1 range primitive commit |
| history 복원 시 stale range | 문서 bounds 재검증, 실패 시 selection 없이 복원 | Stage 3 history metadata commit |
| SelectionRenderer 좌표 변경의 본문 회귀 | page-left resolver와 기존 mode 회귀 test를 같은 Stage에 포함 | Stage 2 renderer wiring commit |
| snapshot 메모리 증가 | 선택 범위 복합 연산에만 한정, 일반 typing 정밀 command 유지 | Stage 3 snapshot command commit |

각 Stage는 독립 커밋으로 고정한다. 실패 시 뒤 Stage에서 조건문으로 우회하지 않고 해당
Stage commit을 되돌릴 수 있게 파일과 테스트를 함께 묶는다.

## 11. 다음 승인 요청

최신 `upstream/devel` 병합 뒤 Stage 4 자동 검증을 완료했다. 실제 Google Chrome E2E 50/50,
Studio 1,254 passed, focused Rust 6/6, clippy, production build, 최적화 WASM과 전체 nextest
8,558/8,558이 통과했다. 상세 결과는 `mydocs/report/task_m100_4121_report.md`에 기록했다.

다음 단계는 `http://127.0.0.1:7700/`에서 사용자가 직접 Both/Odd/Even 머리말·꼬리말 선택과
편집을 확인하는 것이다. 원격 push, PR 생성과 #4121 close는 사용자 수동 확인 및 별도 승인
전까지 수행하지 않는다.
