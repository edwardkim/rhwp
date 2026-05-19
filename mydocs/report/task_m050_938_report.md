# Task M050 #938 최종 보고서 — 복학원서 워터마크 정답지 톤 보정

## 대상

- GitHub Issue: https://github.com/edwardkim/rhwp/issues/938
- 대상 파일: `samples/복학원서.hwp`
- 기준 자료: `samples/복학원서.pdf`, `pdf/복학원서-2022.pdf`

## 문제

`복학원서.hwp` 1쪽 중앙 워터마크가 rhwp/rhwp-studio에서 옅은 사각 배경을 가진 이미지처럼 보였다.

원인은 워터마크가 alpha 없는 JPEG이고, rhwp가 JPEG 전체 영역에 grayscale, brightness/contrast, opacity/multiply를 적용하면서 근백색 배경까지 함께 합성했기 때문이다.

## 구현

워터마크 JPEG에 한정해 한컴 정답지 PDF에 가까운 opaque PNG로 선보정한다.

- JPEG 디코딩 후 외곽/전체 근백색 비율로 워터마크성 이미지를 제한 판정
- 근백색 배경은 opaque white로 고정
- 비배경 픽셀은 정답 PDF에서 추출한 회색 톤 분포에 맞춰 piecewise gray mapping 적용
- 선보정이 성공한 워터마크에는 후단 filter, opacity, mix-blend-mode를 중복 적용하지 않음
- SVG, WASM Canvas 직접 렌더, rhwp-studio overlay JSON/DOM 렌더 경로에 동일 정책 적용

## 변경 파일

- `Cargo.toml`
- `src/renderer/svg.rs`
- `src/document_core/queries/rendering.rs`
- `src/renderer/web_canvas.rs`
- `rhwp-studio/src/view/page-renderer.ts`
- `tests/issue_938.rs`
- `tests/issue_514.rs`
- `tests/golden_svg/issue-677/bokhakwonseo-page1.svg`

## 검증

최종 재검증:

```text
cargo test --release --test issue_938 --test issue_514 --test issue_516 --test svg_snapshot
결과: 성공
issue_938: 2 passed
issue_514: 3 passed
issue_516: 8 passed
svg_snapshot: 8 passed

npm run build
결과: 성공
```

이전 단계에서 추가로 확인한 항목:

```text
cargo check --target wasm32-unknown-unknown --release --lib
결과: 성공

docker-compose --env-file .env.docker run --rm wasm
결과: 성공
```

rhwp-studio 개발 서버:

```text
http://127.0.0.1:7700/
```

## 범위 정리

작업 중 전체 `cargo fmt`로 생긴 범위 밖 tracked 변경은 작업지시자 승인 후 복원했다. 현재 tracked diff는 #938 관련 파일 7개만 남는다.

추가 검증 중 발견된 `PageBackground` 이미지 `fill_mode=Center` 무시 문제는 #938 범위가 아니므로 별도 이슈로 분리했다.

- https://github.com/edwardkim/rhwp/issues/975

## 남은 사항

정답지 PDF와 비교하면 중앙 워터마크의 크기는 거의 같지만, 위치가 약 8.3pt 정도 좌상단에 치우치는 차이가 남아 있다. 이번 작업에서는 sample 전용 offset을 하드코딩하지 않았다. 위치 보정은 이미지 anchor/page origin 계산을 별도로 분석한 뒤 처리해야 한다.

이슈 close는 작업지시자 승인 전까지 수행하지 않는다.
