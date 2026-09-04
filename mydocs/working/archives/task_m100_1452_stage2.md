# Task M100 #1452 Stage 2 시작 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋: `c25aa7ee task 1452: 그림 삽입과 Shift+Tab 내어쓰기 개선`

## 1. Stage 2 목표

Stage 1에서 남은 PNG 투명도 관련 피드백을 분리해 확인한다.

1. PNG 파일의 픽셀 알파가 삽입/저장/렌더 경로에서 보존되는지 재검증한다.
2. 그림 개체 전체 opacity/alpha 속성이 현재 모델과 저장 포맷에 없는 문제를 별도 개선 대상으로 볼지 판단한다.
3. 실제로 픽셀 알파를 훼손하는 코드 경로가 확인되면 이번 스테이지에서 수정한다.
4. 개체 opacity는 설계 범위가 커지면 후속 이슈 또는 후속 스테이지로 분리한다.

## 2. 현재 가설

- 삽입 시 `BinDataContent.data`에는 원본 이미지 바이트가 그대로 들어간다.
- HWP/HWPX 저장 경로도 PNG 바이트 자체를 변환하지 않는다.
- 브라우저/SVG 렌더 경로는 `data:image/png;base64,...`를 사용하므로 픽셀 알파는 보존될 가능성이 높다.
- HWPX `<hc:img alpha>`는 현재 `0` 고정이며, 이는 픽셀 알파가 아니라 그림 개체 전체 opacity로 보인다.

## 3. 확인할 코드 경로

- `src/document_core/commands/object_ops.rs`
  - 그림 삽입 시 BinDataContent 적재
- `src/serializer/cfb_writer.rs`
  - HWP BinData 저장
- `src/serializer/hwpx/mod.rs`
  - HWPX BinData 저장
- `src/serializer/hwpx/picture.rs`
  - `<hc:img alpha>` 직렬화
- `src/renderer/svg.rs`
  - PNG data URI 렌더

## 4. 검증 후보

- 투명 픽셀이 포함된 최소 PNG 바이트를 삽입한 뒤 `bin_data_content`가 원본과 동일한지 확인한다.
- HWPX serialize 결과의 `BinData/image*.png` 바이트가 원본과 동일한지 확인한다.
- SVG export 또는 renderer 결과가 `image/png` data URI를 유지하는지 확인한다.

## 5. 진행 기록

### 2026-06-21

- 알파 채널이 있는 1×1 PNG(color type 4) 바이트를 HWPX로 직렬화한 뒤 ZIP 내부 `BinData/*.png`
  엔트리가 원본 바이트와 동일한지 확인하는 focused 테스트를 추가했다.
- 검증:
  - `cargo test --lib issue1452_hwpx_preserves_alpha_png_bindata_bytes -- --nocapture`
  - 결과: 통과
- 현재 판정:
  - HWPX 저장 경로는 PNG 알파 채널 바이트를 훼손하지 않는다.
  - 사용자가 본 투명도 문제는 HWP 저장 경로, Studio 렌더 경로, 한컴 표시 경로, 또는 그림 개체 전체 opacity
    요구 중 어느 쪽인지 추가 분리가 필요하다.

## 6. 제외 후보

- 그림 전체 opacity UI 추가
- HWP/HWPX/HWP5의 모든 그림 효과 매핑
- 한컴별 PNG 렌더러 차이 보정
