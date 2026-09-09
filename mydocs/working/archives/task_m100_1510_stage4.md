# Task #1510 Stage 4 — PDF 단위/본문 피치 시각 잔차 보정

## 1. 출발점

Stage 3 커밋: `06c45988 task 1510: HWPX visible float 페이지네이션 보정`

Stage 3 결과:

- HWPX `filler 29/30` 페이지 분할은 한컴 2024 PDF와 맞음.
- HWPX 본문 줄 피치는 `textheight + spacing` 기준으로 보정됨.
- HWPX page 2는 거의 맞지만, page 1 상단 표 셀 높이 차이가 남음.
- HWP PDF는 한컴 A4 PDF와 rhwp PDF page size가 달라 직접 좌표가 크게 어긋남.

## 2. 남은 문제

1. HWP export-pdf 결과가 `793.707 x 1122.48 pt`로 생성되어 한컴 2024의
   `595 x 841 pt` A4 PDF와 page size/scaling이 다르다.
2. HWP 본문 순수 텍스트 문단의 줄 진행이 `line_height + spacing`으로 계산되어,
   한컴 HWP PDF보다 하단 filler 문단이 약 22pt 아래로 누적된다.
3. HWPX page 1 표 셀 2행 텍스트가 한컴보다 약 4pt 아래에 놓이는 잔차가 남아 있다.

## 3. 적용

- `svg_to_pdf()` 단일 페이지 fast path를 제거하고, 단일/다중 페이지 모두
  `svgs_to_pdf()`의 96dpi SVG px → 72pt PDF 환산 경로를 타게 했다.
  - HWP 1쪽 PDF: `793.707 x 1122.48 pt` → `595.28 x 841.86 pt`.
- 본문 순수 텍스트 문단의 줄 진행에서 저장된 `text_height + spacing` 보정을 HWP에도 적용했다.
  - 표 셀 HWP 경로는 제외했다. HWP 표 셀 텍스트는 현재 한컴 대비 이미 위쪽에 가까워,
    같은 보정을 적용하면 더 틀어진다.
- 중복 계산 방지를 위해 `is_hwpx_source` 판정값을 `paginate_pass` 내부 지역 변수로 공유했다.

## 4. 기각한 접근

- HWPX 표 셀 측정/렌더 경로 전체에 `text_height` 보정을 전파하면
  `filler paragraph 30`이 다시 1쪽에 남아 `issue_1510_hwpx_unsigned_negative_offset...`
  테스트가 실패했다.
- 결론: HWPX 표 셀 2행 잔차는 표 전체/exclusion 높이를 줄이는 방식으로 풀면 안 된다.
  다음 단계가 필요하면 표 전체 예약 높이는 유지하면서 내부 row/baseline 배치만 조정해야 한다.

## 5. 검증 계획

- `cargo build --bin rhwp`
- `cargo test --test issue_1510 -- --nocapture`
- `cargo test --test issue_986 -- --nocapture`
- `cargo test --test issue_712 -- --nocapture`
- `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwpx`
- `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwp`
- `pdftoppm`/`pdftotext -bbox`로 한컴 2024 PDF와 비교

## 6. 중간 비교 결과

출력 디렉터리: `output/pdf/issue1510_stage4_pdf_scale/`

- HWP page size: rhwp `595.28 x 841.86 pt`, Hancom `595 x 841 pt`.
- HWP bbox:
  - `filler29`: `dy=+0.9pt` (Stage 3 재비교 시 `dy=+21.9pt`)
  - `filler30`: `dy=+1.0pt` (Stage 3 재비교 시 `dy=+23.0pt`)
- HWP PNG diff:
  - `diff>30=2.6297%`, `diff>80=1.7344%`, `rms=24.133`
- HWPX 기존 결과 유지:
  - page 1 `diff>30=2.2805%`, `rms=21.614`
  - page 2 `diff>30=0.1450%`, `rms=4.388`

## 7. 검증 결과

- `cargo build --bin rhwp`: 통과
- `cargo test --test issue_1510 -- --nocapture`: 통과
- `cargo test --test issue_986 -- --nocapture`: 통과
- `cargo test --test issue_712 -- --nocapture`: 통과
  - 기존 `LAYOUT_OVERFLOW ... overflow=2.8px` 로그는 유지되고 테스트는 통과.
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
