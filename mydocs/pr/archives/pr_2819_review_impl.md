# PR #2819 메인테이너 보정 기록

## 누락 경로

원 PR은 Rust SVG·WebCanvas·Skia의 `TOTAL` 분기를 고쳤지만 rhwp Studio 기본 렌더 선택지인
CanvasKit의 fill-mode 해석은 보정하지 않았다. 따라서 같은 문서가 백엔드에 따라 다시 원본 크기로
그려질 수 있었다.

## 보정

- CanvasKit image replay에서 `total`을 `fitToSize`와 같은 stretch 경로로 처리했다.
- LayerTree 타입과 flow image 경로에 fill-mode 전달을 보존했다.
- `CanvasKit image TOTAL fill stretches like fitToSize` 회귀 테스트를 추가했다.
- 보정 커밋: `46f5e0cbc`.

## 판정

Studio 전체 505/505, TypeScript, WASM 빌드와 작업지시자 브라우저 검증을 통과했다. 동일 의미가
렌더 백엔드마다 갈리지 않도록 하는 필수 통합 보정으로 수용한다.
