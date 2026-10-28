# Task 1440 구현 계획서

## 현재 판단

현재 코드에는 `LineSeg.column_start`와 `segment_width`를 어울림 영역으로 해석하는 기반이
이미 있다.

- 파서: `src/parser/body_text.rs`가 HWP5 `PARA_LINE_SEG`의 `column_start`, `segment_width`를 IR로 보존한다.
- 모델: `src/model/paragraph.rs`의 `LineSeg::is_in_wrap_zone()`은 `column_start > 0` 또는 `segment_width < col_w`를 wrap zone으로 정의한다.
- 렌더: `src/renderer/layout/paragraph_layout.rs`는 `has_picture_shape_square_wrap` 또는 `wrap_anchor`가 있을 때 LineSeg의 cs/sw를 TextLine x/width에 반영한다.
- 타입셋: `src/renderer/typeset.rs`는 `wrap_anchors`를 등록해 후속 문단을 anchor 그림 옆 wrap 문단으로 전달한다.

따라서 #1440은 새 레이아웃 모델 도입보다, 해당 샘플의 그림/본문 조합에서 기존 `wrap_anchor`
또는 `has_picture_shape_square_wrap` 분기가 빠지는 원인을 좁혀 수정하는 방식이 적합하다.

## 구현 단계

1. 진단 테스트 추가
   - `samples/[2027] 온새미로 1 본교재.hwp` 35쪽 render tree에서 우측 상단 그림과 y가 겹치는 TextLine bbox를 수집한다.
   - 수정 전에는 일부 TextLine이 그림 bbox를 침범하는 사실을 재현한다.
   - 수정 후에는 그림과 같은 y 대역의 TextLine이 그림 좌/우 wrap 영역 안에만 위치하도록 검증한다.

2. 원인 분기 확인
   - 해당 문단의 `line_segs`에서 `column_start`/`segment_width`가 한컴 wrap zone 값인지 출력/테스트로 확인한다.
   - `ColumnContent.wrap_anchors`에 해당 paragraph index가 등록되는지 확인한다.
   - 등록되지 않는 경우 anchor 매칭 조건을, 등록되지만 반영되지 않는 경우 `paragraph_layout` 적용 조건을 수정한다.

3. 최소 수정
   - 우선순위는 `wrap_anchors` 등록 조건 보강이다.
   - 단, 같은 paragraph 안에 비-TAC Square 그림이 있고 LineSeg가 이미 wrap zone을 담는 경우에는 `paragraph_layout`이 LineSeg cs/sw를 적용하도록 좁게 보강한다.
   - 기존 다단 필터링과 표 어울림 경로에는 영향이 가지 않도록 조건을 `TextWrap::Square` 및 실제 wrap zone LineSeg 존재 여부로 제한한다.

4. 시각 검증 자료
   - 한컴 PDF 35쪽 raster와 rhwp 수정 전/후 raster를 `mydocs/report/assets/task_m100_1440/` 아래에 보관한다.
   - 최종 보고서에 사용 샘플, 명령, 차이를 기록한다.

## 검증

- `cargo test --test issue_1440_onsamiro_picture_wrap`
- 관련 기존 wrap/table 테스트 선별 실행
- PR 전 필수 검증:
  - `cargo build --release`
  - `cargo test --release --lib`
  - `cargo test --profile release-test --tests`
  - `cargo fmt --check`
  - `cargo clippy --all-targets -- -D warnings`

## 보류/주의

- 사용자 제공 PDF/HWPX가 untracked로 존재하므로, 샘플 추가 여부는 별도 승인 후 결정한다.
- 이번 작업은 35쪽 그림-본문 감싸기 정합이 목표이며, 일반 폰트 폭/자간 재튜닝은 범위 밖으로 둔다.

