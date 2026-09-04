# 완료 보고서 — Task M100-2953

- 이슈: #2953
- 제목: fix(hwpx): 도형/셀 배경 imgBrush mode 12종 중 9종이 저장 시 TILE 로 붕괴
- 작성일: 2026-07-22
- 브랜치: `task/m100-2953-imgbrush-mode-roundtrip`

## 1. 완료 내용

`src/parser/hwpx/section.rs::parse_shape_fill_brush` 는 `<hc:imgBrush mode="...">`
속성을 12종 전부 개별 `ImageFillMode` 값으로 정확히 적재하는데(#2563 에서 4종 → 12종
확장), 역방향인 `src/serializer/hwpx/shape.rs::write_fill_brush` 는 `FitToSize`/
`Total`/`Center` 3종만 구분하고 나머지 전부(`TileHorzTop`/`TileHorzBottom`/
`TileVertLeft`/`TileVertRight`/`CenterTop`/`CenterBottom`/`LeftTop`, 7종)를
`"TILE"` 로 뭉개고 있었다. IR 에는 정확한 배치가 남아 있지만 emit 이 하드코딩돼
저장 왕복에서 사용자가 지정한 이미지 채우기 배치가 유실되는 문제였다.

`write_fill_brush` 는 도형(`hp:rect`/`hp:line`)뿐 아니라 `header.rs` 의 `borderFill`
(셀·쪽 배경)도 공유하므로, 영향 범위는 도형 이미지 채우기와 셀/쪽 배경 이미지 채우기
양쪽 모두다.

## 2. 주요 변경

- `src/serializer/hwpx/shape.rs`
  - `write_fill_brush` 의 `mode` match 에 파서가 지원하는 나머지 7종
    (`TileHorzTop`/`TileHorzBottom`/`TileVertLeft`/`TileVertRight`/`CenterTop`/
    `CenterBottom`/`LeftTop`) arm 을 추가해 파서와 대칭이 되도록 했다.
  - `task2943_img_brush_mode_roundtrip_not_collapsed_to_tile` 테스트 추가:
    `ImageFillMode::TileHorzTop` 을 직렬화해 `mode="TILE_HORZ_TOP"` 로 방출되는지 확인.

## 3. 검증 결과

- red → green: 수정 전 임시로 되돌려 테스트가 `mode="TILE"` 로 실패하는 것을 확인한 뒤
  (`TileHorzTop 이 TILE 로 붕괴함: <hc:fillBrush><hc:imgBrush mode="TILE"/></hc:fillBrush>`),
  수정을 재적용해 통과함을 확인.
- `cargo check --lib` 통과
- `cargo test --lib task2943_img_brush_mode_roundtrip_not_collapsed_to_tile` 통과
- `rustfmt --edition 2021 src/serializer/hwpx/shape.rs` 적용

## 4. 남은 이슈

없음. `CommonObjAttr` (src/model/shape.rs) 는 건드리지 않았다.
