# Task 1282 Stage 3 — rhwp-studio 드래그 경로 회귀 검증

## 목적

Stage 2의 Rust by-path cell height 보정이 rhwp-studio 실제 마우스 리사이즈 경로에서도
동일하게 적용되는지 검증한다.

## 변경 내용

- `rhwp-studio/e2e/table-picture-resize-1282.test.mjs` 추가
  - `samples/ta-pic-001-r.hwp`를 로드한다.
  - 표 셀 안 회전 picture를 `cellPath`로 선택한다.
  - 실제 InputHandler 경로(`mousedown` → `mousemove` → `mouseup`)로 `se` 리사이즈 핸들을 드래그한다.
  - 드래그 상태의 `cellPath` 보존, picture height 증가, owner cell height 증가, bbox 중심 과도 점프 방지,
    undo picture size 복구를 검증한다.

## 검증

통과:

```text
wasm-pack build --target web --out-dir pkg
cd rhwp-studio && node e2e/table-picture-resize-1282.test.mjs --mode=headless
```

확인 결과:

- `pictureResizeState.ref.cellPath` 보존
- picture height 증가
- owner cell height 증가 및 `picture.vertOffset + picture.height + padding` 충족
- live/finish bbox center jump 48px 이내
- undo 후 picture width/height 복구

메모:

- 드래그 후 `horzOffset`은 기존 Studio 저장 경로와 동일하게 unsigned u32 표현으로 노출될 수 있다.
  이번 stage는 중심/크기와 cell height 안정성 검증 범위로 두고 별도 UI 표시 정규화는 후속 이슈로 분리한다.
