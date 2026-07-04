# Task #1692 Stage 10 - SO-SUEOP 잔여 시각 차이 재탐색

## 배경

- 직전 커밋: `336746fe6 task 1692: SO-SUEOP p22 관계도 선문자 복원`
- 작업지시자 판정: `SO-SUEOP-2024.pdf` 기준으로 HWP/HWPX 파일을 페이지 단위로 다시 확인해야 한다.
- Stage 9에서는 22쪽 관계도 원 번호/연결선의 검은 박스 문제를 줄였지만, 전체 완료 판정은 별도 재검증이 필요하다.

## 계획

1. `samples/SO-SUEOP.hwp`, `samples/SO-SUEOP.hwpx`, `pdf/SO-SUEOP-2024.pdf`의 페이지 수와 대표 페이지 PNG를 다시 생성한다.
2. 자동 비교 후보와 수동 시각 확인을 함께 사용해 남은 차이를 좁힌다.
3. 실제 회귀 원인이 확인된 경우에만 최소 범위로 수정한다.

## 진행 기록

- `samples/SO-SUEOP.hwp`, `samples/SO-SUEOP.hwpx`, `pdf/SO-SUEOP-2024.pdf` 재렌더 결과 페이지 수는 모두 46쪽으로 일치했다.
- HWP3 22쪽에서 HWPX/PDF에는 있는 머리말 하단선이 HWP 렌더에는 빠진 것을 확인했다.
  - HWP3 머리말 내부의 `ShapeObject::Line`은 `treat_as_char=true`이지만 Header/Footer 경로에서는 inline 좌표 등록 전에 `layout_shape`가 호출되어 안전장치에 의해 스킵됐다.
  - Header/Footer 내부 선 개체에 한해 inline 좌표가 없어도 기존 위치 계산 fallback을 허용하도록 수정했다.
- 재렌더 확인:
  - `SO-SUEOP_022.svg`에 `<line x1="113.386..." y1="128.8" x2="680.266..." ...>`가 생성됐다.
  - HWP 22쪽 PNG에서 머리말 밑줄이 PDF 기준처럼 표시됨을 확인했다.

## 검증

- `cargo fmt`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 issue_1692_so_sueop_header_footer_page5_matches_reference_contract -- --nocapture`
- `env CARGO_INCREMENTAL=0 cargo test -q --test issue_1692 -- --nocapture`

## 남은 관찰

- 22쪽 관계도 검은 박스와 HWP3 머리말 밑줄은 복원됐지만, PDF 기준 세부 폰트 폭/줄바꿈/위치 차이는 아직 남아 있다.
- 후반 2단 페이지(예: 44쪽)는 자동 픽셀 차이 상위 후보이며, 다음 단계에서 줄폭/2단 흐름 차이를 별도로 좁혀야 한다.
