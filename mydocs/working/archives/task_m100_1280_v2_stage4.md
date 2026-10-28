# Task #1280 (v2) 4단계 완료보고서 — 정합 회귀 e2e + 선택 ref 소비처 lifecycle 검증

## 목표

Stage 3에서 히트테스트가 겹침 시 **다른 개체(최상단)**를 선택하게 되었다. 메모리 룰
`audit-selection-ref-consumers`(PR #1254 교훈)에 따라 **선택 ref 소비처를 전수 감사**하고,
**선택 후 연산 lifecycle**(삭제/오려두기)이 올바른 개체를 처리하는지 e2e로 검증한다.

## 1. 선택 ref 소비처 전수 감사 (grep)

`getSelectedPictureRef`/`getSelectedPictureRefs`/`selectedPictureRef` 소비처 (test 제외):

| 파일 | 연산 |
|------|------|
| `engine/cursor.ts` | 저장/토글/해제 (`enterPictureObjectSelectionDirect`/`togglePictureObjectSelection`/`exitPictureObjectSelection`) |
| `engine/input-handler.ts` | `getSelectedPictureRef(s)` 공개 패스, 컨텍스트메뉴(1258), 복사(2502)/오려두기(2556)/삭제(2600) |
| `engine/input-handler-keyboard.ts` | Delete/Cut/Copy 단축키, 화살표/Tab 이동 선택 (8곳) |
| `engine/input-handler-mouse.ts` | 멀티선택/테두리 클릭/리사이즈·이동 시작/클릭 선택 (10곳) |
| `engine/input-handler-table.ts` | 표 내 멀티선택 (393-394) |
| `engine/input-handler-picture.ts` | 멀티선택/속성 (269, 298) |
| `command/commands/insert.ts` | 속성/수식편집/crop/도형(flip·rotate·arrange)/삭제/그룹/멀티 (13곳) |
| `command/commands/format.ts` | 속성 대화상자 (454) |

### 핵심 결론 — PR #1254 와 본질적으로 다름 (배선 누락 위험 없음)

- PR #1254는 **새 식별 경로(cellPath sentinel)**를 도입하고 일부 소비처만 배선해 결함이 났다.
- 본 Stage 3는 **새 ref 형태를 도입하지 않는다.** `controlToRef`(input-handler-picture.ts)가
  반환하는 ref 는 수정 전 Pass 1 첫-적중 반환과 **필드 구성이 완전히 동일**
  (`sec/ppi/ci/type/cellIdx/cellParaIdx/outerTableControlIdx/cellPath/noteRef/headerFooter`).
  바뀌는 것은 **어느 개체를 가리키느냐(최상단)** 뿐이다.
- 따라서 모든 소비처는 이미 이 ref 형태를 처리하고 있으며(수정 전에도 첫-적중 개체에 대해 동작),
  추가 배선이 불필요하다. 겹침 최상단은 보통 **본문 floating 글상자(type=shape, cellPath 없음)**라
  가장 단순·지원이 확실한 ref 형태다.

## 2. lifecycle e2e (신규: `topmost-lifecycle.test.mjs`)

`samples/textbox-under-image.hwp`(글상자 plane3 위 / 이미지 plane2 아래, 겹침)에서 **실제 hit-test
경로**(`findPictureAtClick` → `selectPictureObject`)로 최상단 글상자를 선택한 뒤:

| 시나리오 | 선택 | 연산 | 결과 |
|---------|------|------|------|
| 삭제 | shape ci=2 | `performDelete()` | shapes 1→**0**, images 1→**1** (글상자만 삭제, 이미지 잔존) |
| 오려두기 | shape ci=2 | `performCut()` | shapes 1→**0**, images 1→**1** (동일) |

→ 선택 ref 소비처(삭제/오려두기)가 **최상단 글상자만** 처리하고 이미지는 건드리지 않음을 입증.
엉뚱한 개체 처리(PR #1254식 결함)가 없음을 실측 확인.

## 3. 회귀 e2e

```
node e2e/topmost-hittest.test.mjs           # Stage 3 — PASS
node e2e/topmost-lifecycle.test.mjs         # 신규 — PASS
node e2e/textbox-picture-1171.test.mjs      # #1171 nested picture hit — PASS
node e2e/textbox-picture-ops-1273.test.mjs  # 리사이즈/회전/floating lifecycle — PASS
node e2e/textbox-picture-insert-1171.test.mjs # 글상자 위 이미지 드롭 — PASS
```

(모두 headless Chrome, WASM 재빌드 후. `#516` 전용 e2e는 부재이나 BehindText Pass 2 경로는
코드 무변경이며 Pass 1 은 topmost-hittest 가 커버.)

## Rust 회귀

Stage 2~4는 프런트(TS/e2e) 전용으로 Rust 소스 무변경 → Stage 1의 `cargo test --lib`
**1581 passed; 0 failed** 결과가 그대로 유효.

## 다음 단계

Stage 5 — 글상자 삽입 기본값 `floating + InFrontOfText` 교정(#1280 인라인 결정 되돌림) +
#1280 텍스트 입력/붙여넣기 회귀 확인.

## 승인 대기

본 보고서와 e2e 커밋 후 승인 요청. 승인 후 Stage 5 진행.
