# Task M100 #4739 Stage 1 - local face 선택 RED 계약

## 목적

Canvas2D 첫 paint와 정부상징 successor 선택을 고치기 전에 현재 실패를 재현 가능한 계약으로
고정한다. 이 단계는 테스트만 추가하며 제품 동작을 바꾸지 않는다.

## 고정한 실패

1. `KoPub바탕체 Light/Medium/Bold`가 `GulimChe` 중심의 monospace chain으로 간다.
2. `정부상징 부처명_16040911` exact face가 없고 `대한민국정부상징체 R`만 확인돼도 successor를
   선택하지 않는다. HWPX가 선언한 `한컴바탕`도 generic chain 앞에 보존되지 않는다.
3. `initializeDocument()` 안에서 저장된 local-font snapshot을 읽기 전에
   `canvasView.loadDocument()`가 첫 paint를 시작한다.
4. `local-fonts-changed`는 toolbar만 갱신하고 main의 backend별 view repaint를 요청하지 않는다.
5. Rust style resolver는 non-embedded `FontFace.subst_font`를 표시용 family chain에 보존하지
   않으며, renderer에는 구형 정부상징 face에서 ROKG로 가는 명시적 successor chain이 없다.

## RED 실행 결과

- `node tests/font-substitution.test.ts`: 9건 중 7 통과, 2 실패
  - KoPub바탕체 serif 계약 실패
  - exact → ROKG → 문서 대체 face 순서 실패
- `node tests/document-initialization-order.test.ts`: 6건 중 4 통과, 2 실패
  - 저장 snapshot 선행 실패
  - local-font 갱신 view repaint 실패
- `cargo test --target-dir target/issue4739-red --lib
  test_lookup_font_preserves_non_embedded_document_substitute -- --nocapture`: 의도한 assertion 실패
- 같은 target의 `test_base_family_without_weight_suffix`: 정부상징 renderer chain assertion 실패

## 다음 단계

Stage 2에서 문서 대체 face 보존, 정부상징 successor와 KoPub serif 해소, 저장 snapshot의 첫 paint
선행을 구현한다. Stage 3에서는 `local-fonts-changed`를 backend별 단일 view repaint로 연결한다.
