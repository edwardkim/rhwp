# Task #1280 (v2) 5단계 완료보고서 — 글상자 삽입 기본값 floating+InFrontOfText 교정

## 목표

#1280 본편이 정한 인라인 글상자 삽입을, 한컴 정답값 **floating(treat_as_char=false) +
글앞으로(InFrontOfText)** 로 되돌린다. 그래야 글상자 위 어울림(Square) 이미지가 글상자 뒤로 가고
(plane 3>2), 로드된 기존 글상자와도 정합한다.

## 권위 샘플 실측 (rhwp dump samples/textbox-under-image.hwp)

- 글상자(사각형): **배치=글앞으로(InFrontOfText), 글자처럼=false, 가로/세로=용지(Paper) 기준**, z=0,
  글상자 margins=(283,283,283,283).
- 이미지: 배치=어울림(Square), z=1.

→ floating 글상자의 common-attr 은 floating 사각형과 동일(Paper/Paper/InFrontOfText). 글상자
특수성은 margin(283)+text_box 뿐.

## 변경 내용

### 1. 백엔드 — `create_shape_control_native` (`src/document_core/commands/object_ops.rs`)

`inline_textbox = (shape_type=="textbox" && treat_as_char)` 게이트 도입:
- **attr**: inline 글상자만 0x0A0210(Para/Column/Square), 그 외(floating 글상자·도형)는
  0x046A4000(Paper/Paper/InFrontOfText+절대크기). serializer(`control.rs:1768`)가 `common.attr!=0`
  이면 그대로 직렬화하므로 enum 필드와 함께 정합.
- **vert_rel_to / horz_rel_to**: inline 글상자만 Para/Column, 그 외 Paper/Paper.
- margin(283)·has_textbox(text_box 생성)은 textbox 면 그대로 유지.
- → treat_as_char=true 인 inline 글상자는 #1280 본편 동작 보존(회귀 가드 테스트 추가).

### 2. 백엔드 — wrap 노출 보강 (`src/document_core/queries/rendering.rs`)

Stage 1 계획 항목("wrap 을 shape/line/group 에도 노출")의 잔여 구현. floating 글상자 검증 중
e2e 에서 shape 의 `wrap` 미노출이 드러나 보완. 이미지 외 컨트롤은 `node.layer.text_wrap` 에서
`wrap` 을 파생해 layer_str 에 포함(이미지는 자체 wrap_str 유지, 중복 방지).

### 3. 프런트 — `finishTextboxPlacement` (`rhwp-studio/src/engine/input-handler.ts`)

- 종이 기준 오프셋 계산에서 글상자 제외(`!== 'textbox'`)를 제거 → 글상자도 드래그 위치에 floating
  배치(기존 사각형 경로 재사용).
- `createShapeControl` 에 글상자일 때 `treatAsChar:false` + `textWrap:'InFrontOfText'` 명시 전달.

## 검증

### Rust 단위 (네이티브)
```
cargo test --lib issue_1280   → 7 passed (기존 4 + 신규 3)
cargo test --lib              → 1584 passed; 0 failed; 6 ignored
rustfmt --check (rendering.rs, object_ops.rs)  → clean
```
신규 테스트:
- `create_floating_textbox_is_in_front_paper`: floating 글상자가 Paper/Paper/InFrontOfText +
  text_box 보유 + attr 비트(bit0=0, bit3-4=0, bit8-9=0, bit21-23=3) 정합.
- `create_inline_textbox_preserves_para_column`: inline(treat_as_char=true)은 Para/Column 보존(회귀 가드).
- `insert_text_into_floating_textbox`: floating 글상자도 텍스트 입력 정상(#1280 회귀 없음).

### 프런트/e2e (WASM 재빌드 후 headless)
```
npx tsc --noEmit                              → 신규 오류 0 (canvaskit 3건 베이스라인)
node e2e/textbox-insert-floating-1280v2.test.mjs  → PASS (삽입 글상자 plane=3, wrap='inFrontOfText')
node e2e/issue-1280-textbox-text-input.test.mjs   → PASS (#1280 본편 회귀: 삽입→텍스트 입력)
node e2e/topmost-hittest.test.mjs                 → PASS
node e2e/topmost-lifecycle.test.mjs               → PASS
node e2e/textbox-picture-1171.test.mjs            → PASS (1회 navigation timeout 플레이크 → 재실행 통과)
```

신규 e2e `textbox-insert-floating-1280v2`: 실제 삽입 경로(enterTextboxPlacementMode+드래그)로 만든
글상자가 레이아웃에서 plane=3(InFrontOfText)/wrap='inFrontOfText' 임을 확인 → 프런트→백엔드 전 경로
floating 정합 입증.

## 다음 단계

전체 5단계 완료. 최종 결과보고서(`report/task_m100_1280_v2_report.md`) 작성 → 승인 → 통합(PR).

## 승인 대기

본 보고서와 소스 커밋 후 승인 요청.
