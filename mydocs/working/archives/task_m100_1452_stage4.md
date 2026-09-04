# Task M100 #1452 Stage 4 시작 기록

- 이슈: #1452 `rhwp-studio: 그림 삽입/배치 속성 및 Shift+Tab 내어쓰기 개선`
- 브랜치: `local/task_m100_1452`
- 작성일: 2026-06-21
- 선행 커밋:
  - `c25aa7ee task 1452: 그림 삽입과 Shift+Tab 내어쓰기 개선`
  - `f5b2b035 task 1452: PNG 알파 BinData 보존 검증 추가`
  - `ab601c83 task 1452: 개체 속성 창 크기 고정`

## 1. 배경

Stage 2에서 PNG 픽셀 알파는 HWPX BinData 원본 바이트로 보존되는 것을 확인했다. 다만 `개체 속성 > 그림`
탭의 전체 투명도는 모델 필드가 없고, HWPX `<hc:img alpha>`도 `0`으로 고정되어 있어 실제 편집/저장
기능으로 이어지지 않는다.

이번 스테이지에서는 픽셀 알파와 별개인 그림 개체 전체 투명도 기능을 구현한다.

## 2. 근거

- HWPX 그림 경로는 `<hc:img ... alpha="...">` 속성이 있다.
- 사용자 제공 샘플 `samples/투명도0-50.hwpx`에서 첫 번째 그림은 `alpha="0"`, 두 번째 그림은
  `alpha="127"`로 저장되어 있다. 한컴 UI의 투명도 50%는 HWPX 저장값 127(0~255 alpha byte)로
  materialize된다.
- HWP5 스펙 `mydocs/tech/한글문서파일형식_5.0_revision1.3.md` 표 116 `그림 추가 속성`에는 마지막
  `INT8` 필드로 `이미지 투명도`가 있다.
- 현재 HWP serializer는 raw tail이 없을 때 해당 위치에 `0`을 쓰고 있으므로, `0=불투명` 기본값과 호환된다.

## 3. 작업 범위

- `ImageAttr`에 그림 개체 전체 투명도 값을 추가한다.
- `getPictureProperties`/`setPictureProperties` JSON에 투명도 값을 포함한다.
- rhwp-studio `개체 속성 > 그림 > 투명도` 입력을 활성화하고 저장한다.
- HWPX parser/serializer에서 `<hc:img alpha>`를 0~255 저장값으로 읽고 쓴다.
- HWP parser/serializer에서 `raw_picture_extra`의 그림 추가 속성 마지막 바이트를 투명도 값으로 읽고,
  저장 시 기존 raw 보존을 유지하면서 값이 바뀐 경우 해당 바이트를 갱신한다.
- 렌더링 경로에서 이미지 노드 opacity를 적용한다.

## 4. 구현 결과

- 내부 모델은 한컴 UI 기준 0~100 퍼센트를 유지하고, HWP/HWPX 파일 입출력에서만 0~255 alpha byte로
  변환한다. 예: 50% → 127, 100% → 255.
- `samples/투명도0-50.hwp`, `samples/투명도0-50.hwpx`를 fixture로 추가해 두 파일 모두 첫 두 그림이
  `[0, 50]` 퍼센트로 파싱되는지 검증한다.
- `PictureProperties`에 `transparency`를 추가하고, Studio 그림 속성 창의 `투명도` 입력을 활성화했다.
- paint JSON, overlay image JSON, CanvasKit, Canvas2D, SVG, Skia 렌더러에 이미지 노드 opacity 적용을
  연결했다.

## 5. 검증 결과

- `cargo test --lib issue1452 -- --nocapture` 통과
  - `issue1452_picture_transparency_updates_hwp_extra_byte`
  - `issue1452_img_alpha_uses_hwp_alpha_byte`
  - `issue1452_picture_transparency_props_roundtrip`
  - `issue1452_picture_transparency_samples_parse_as_ui_percent`
  - 기존 Stage 1/2 `issue1452` 회귀 테스트 포함
- `cd rhwp-studio && npx tsc --noEmit` 통과
- `cargo fmt --check` 통과
- `git diff --check` 통과
