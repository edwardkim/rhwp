# Task #1510 Stage 2 — 시각 정합 개선

## 1. 출발점

Stage 1 커밋: `fd6ca17a task 1510: co-anchored float 표 순서 보정`

Stage 1은 visible text host 문단의 co-anchored para-relative `TopAndBottom` 표를
`vertical_offset`으로 재정렬하지 않고, 큰 양수 offset이 페이지를 2쪽으로 밀어내는
흐름 누적을 차단했다.

## 2. 남은 차이

`output/pdf/issue1510_compare/`의 PNG 비교 기준:

- HWP: 한컴 2024와 rhwp 모두 1페이지지만, A 표 주변의 본문 회피가 다르다.
  - 한컴 2024: filler 01~07 뒤에 A 표가 들어가고 filler 08부터 표 아래로 이어진다.
  - rhwp: A 표가 filler 06~09와 겹친다.
- HWPX: 한컴 2024는 2페이지, rhwp는 1페이지라 페이지네이션 자체가 다르다.
- PDF page size도 한컴 2024(`595x841pt`)와 rhwp(`793x1122pt`)가 다르므로,
  위치 비교는 정규화 PNG를 함께 본다.

## 3. 개선 방향

1. visible text host 의 para-relative `TopAndBottom` 표 중 양수 offset 표를
   본문 flow에서 완전히 무시하지 않는다.
2. 다만 Stage 1에서 막은 "anchor 직후 전체 공백 누적"으로 돌아가지 않고,
   후속 본문이 표의 실제 y 구간에 닿을 때만 필요한 만큼 cursor를 이동한다.
3. 빈 host 문단의 lane 경로(`#986`)와 음수 offset 침범 가드(`#712`)는 계속 보존한다.
4. HWPX 페이지 차이는 HWP 개선 후 별도 원인으로 분리해서 확인한다.

## 4. 검증 계획

- `target/debug/rhwp export-pdf samples/issue1510_coanchored_float_tables.hwp`
- `pdftoppm` PNG 추출 후 `pdf/issue1510_coanchored_float_tables-hwp-2024.pdf`와 비교
- `cargo test --test issue_1510 -- --nocapture`
- `cargo test --test issue_986 -- --nocapture`
- `cargo test --test issue_712 -- --nocapture`

## 5. 구현 결과

- HWP visible text host 문단의 co-anchored `TopAndBottom` 표에 대해:
  - 음수 `vertical_offset` 표는 visible host 경로에서만 body top clamp를 완화해
    선언된 음수 위치를 반영한다.
  - 양수 `vertical_offset` 표는 Stage 1처럼 anchor 직후 flow를 즉시 밀지 않고,
    렌더된 표의 y 구간을 active exclusion zone으로 등록한다.
  - 후속 일반 문단의 시작 y가 zone 내부에 들어올 때만 cursor를 표 하단으로 이동한다.
- `tests/issue_1510.rs`에 geometry 회귀를 추가했다.
  - B 표(음수 offset)가 C 표(0 offset)보다 위에 렌더되는지 확인.
  - A 표(양수 offset) 도달 전 filler 07은 위쪽에 남고, filler 08은 A 표 아래에서
    재개되는지 확인.
- HWPX는 Stage 2 적용 대상에서 제외했다.
  - `ir-diff` 결과 HWPX 샘플의 B 표 `vertical_offset`이 이미 `0`으로 파싱되어,
    HWP 샘플의 음수 offset 조건과 다르다.
  - 한컴 2024 HWPX PDF는 2페이지, rhwp HWPX는 아직 1페이지이므로 별도 Stage에서
    HWPX offset/페이지네이션 원인을 분리한다.

## 6. 검증 결과

- `cargo test --test issue_1510 -- --nocapture` 통과
- `cargo test --test issue_986 -- --nocapture` 통과
- `cargo test --test issue_712 -- --nocapture` 통과
  - 기존 `pi=546` 2.8px overflow 진단 로그는 유지되지만 테스트는 통과.
- PDF 갱신:
  - `pdf/issue1510_coanchored_float_tables.hwp.pdf`: 1페이지
  - `pdf/issue1510_coanchored_float_tables.hwpx.pdf`: 1페이지
- PNG 비교 산출물:
  - `output/pdf/issue1510_compare/final_hwp_side_by_side.png`
  - `output/pdf/issue1510_compare/final_hwpx_side_by_side.png`
- 정규화 PNG diff:
  - HWP: 한컴 2024 1페이지 vs rhwp 1페이지, `diff>30 = 3.4936%`
  - HWPX: 한컴 2024 2페이지 vs rhwp 1페이지, 1쪽 기준 `diff>30 = 3.5394%`
