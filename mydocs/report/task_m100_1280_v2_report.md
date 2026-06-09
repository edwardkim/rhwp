# Task #1280 (v2) 최종 결과보고서 — 글상자 위 이미지 배치(plane)/클릭 우선순위

## 요약

rhwp-studio에서 글상자 위에 이미지를 올리면 (1) 이미지가 글상자를 가리고(한컴은 반대),
(2) 겹친 곳을 클릭하면 보이는 이미지가 아닌 밑의 글상자가 잡히는 **WYSIWYG 불일치**가 있었다.
원인은 z-order가 아니라 **plane**이었고, **삽입 글상자를 한컴과 정반대(Square+인라인)로 만든 것**이
직접 원인이었다. 두 축(히트테스트 최상단 선택 + 글상자 삽입 기본값 floating·InFrontOfText 교정)으로
해결했다. **작업지시자 육안 판정(삽입 floating, 이미지 뒤로, 겹침 클릭=최상단 + 삭제/오려두기) 통과.**

## 배경 / 근본 원인

- #1280 본편(stage1~4)은 삽입 글상자가 `text_box` 없는 Rectangle로 생성되던 결함을 고쳐 텍스트
  입력을 복구하며 글상자를 **인라인(treat_as_char=true, Square)** 으로 배치했다.
- 한컴 권위 샘플 `samples/textbox-under-image.hwp` 실측(`rhwp dump`): 글상자 **글앞으로
  (InFrontOfText), floating, Paper/Paper**, 이미지 **어울림(Square)**. z는 오히려 이미지(1)>글상자(0).
- 즉 한컴에서 이미지가 글상자 뒤로 가는 건 **글상자가 plane 3(InFrontOfText)** 이라 어울림
  이미지(plane 2) 위에 그려지기 때문. 렌더 정렬키 `(plane, z_order, stable_index)`에서 plane이 상위.
- rhwp-studio 글상자는 plane 2 + 인라인이라 정반대로 깔렸고, 히트테스트 1차 패스는 emit 순서
  **첫 적중**을 반환해 최하단 개체를 잡았다.

## 해결 (2축)

- **축 A — 히트테스트 "클릭 = 최상단 개체"**: `findPictureAtClick` 1차 패스를 적중 후보 중
  `(plane, zOrder, stableIndex)` 최댓값 선택으로 교체. Rust 정렬키 `paper_node_sort_key`를 그대로
  노출·재사용(단일 진실 원천). #1171/#516 패스 보존.
- **축 B — 글상자 삽입 기본값 교정**: 한컴 정답값 `floating(treat_as_char=false) + InFrontOfText
  + Paper/Paper`로 생성(인라인 결정 되돌림). text_box·margin(283)은 유지.

## 단계별 산출 (5단계)

| 단계 | 내용 | 커밋 |
|------|------|------|
| 1 | 레이아웃 쿼리에 `plane/zOrder/stableIndex` 노출 (+wrap, Rust 단위) | b87adc17 |
| 2 | 프런트 `ControlLayoutItem` 필드 확장 (동작 무변화) | 9ccffca9 |
| 3 | `findPictureAtClick` 최상단 선택 + e2e | c7a351cc |
| 4 | 선택 ref 소비처 전수 감사 + lifecycle e2e (메모리 룰) | dacfed2f |
| 5 | 글상자 삽입 floating+InFrontOfText 교정 + #1280 회귀 | 825e3f08 |

(구현계획서 `bb16c749`. 단계별 완료보고서: `working/task_m100_1280_v2_stage{1..5}.md`)

## 변경 파일

- 백엔드: `src/renderer/layout.rs`(paper_node_sort_key 노출), `src/document_core/queries/rendering.rs`
  (plane/zOrder/stableIndex/wrap 방출), `src/document_core/commands/object_ops.rs`(floating 글상자
  attr/relto + 단위 테스트).
- 프런트: `rhwp-studio/src/core/types.ts`(타입), `rhwp-studio/src/engine/input-handler-picture.ts`
  (최상단 히트테스트), `rhwp-studio/src/engine/input-handler.ts`(floating 삽입).
- e2e(신규): `topmost-hittest`, `topmost-lifecycle`, `textbox-insert-floating-1280v2`.

## 검증 결과

- `cargo test --lib` → **1584 passed; 0 failed** (신규 Rust 단위 4종: 레이아웃 정렬키 정합 +
  floating/inline/floating 텍스트입력). `rustfmt --check` clean(변경 파일).
- `npx tsc --noEmit` → 신규 오류 0 (잔여 3건은 `canvaskit-wasm` 미설치 베이스라인).
- `npm test`(타입 스트리핑) → 54 passed.
- e2e(WASM 재빌드 후 headless): `textbox-insert-floating-1280v2`(plane=3/wrap=inFrontOfText),
  `topmost-hittest`, `topmost-lifecycle`, `issue-1280-textbox-text-input`(본편 회귀),
  `textbox-picture-1171`/`-ops-1273`/`-insert-1171`(#1171/#1273 회귀) → **전부 PASS**.
- **작업지시자 육안 판정(rhwp-studio)**: ① 삽입 글상자 floating, ② 이미지가 글상자 뒤,
  ③ 겹침 클릭=최상단 글상자 선택 + 삭제/오려두기 정상 → **확인 완료**.

## 메모리 룰 적용 (audit-selection-ref-consumers)

히트테스트가 겹침에서 다른 개체(최상단)를 선택하게 되므로, 선택 ref 소비처 30+곳을 전수 grep
감사했다. 본 변경은 **새 ref 경로를 도입하지 않고**(controlToRef가 기존과 동일 형태 반환) 가리키는
개체만 바뀌므로 추가 배선 불필요. lifecycle e2e(선택→삭제/오려두기)로 엉뚱한 개체 미처리 확인.

## 범위 밖 (별도 이슈)

- **글상자 안 이미지 붙여넣기** 무음 실패(`merge_from`이 controls 미병합) — #1280 stage4에서 별개
  결함으로 분리됨. 본 v2 비대상.

## 통합 / 후속 (메인테이너 영역)

- 통합: Fork `origin` push → upstream `devel` PR. merge·마일스톤·이슈 클로즈·`orders/` 갱신은
  메인테이너 영역.
- 권장 후속: `npm test` 스크립트의 `.ts` 로더(node `--experimental-strip-types`) 정비(별개·소규모).
