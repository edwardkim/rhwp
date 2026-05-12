# Stage 3 완료 보고서 — Task #842 (M100)

목표: 결함 #3 — 두 단 사이 가운데 구분선이 실선으로 렌더되는 문제(점선이어야 함).

## 원인
shortcut.hwp 2단 ColumnDef 의 `구분선 type=7`(HWP 선 종류 Circle/원형 점선). `src/renderer/layout.rs::build_column_separators` 의 `separator_type → StrokeDash` 매핑이 `2..=5` 만 처리하고 `6`(LongDash)·`7`(Circle) 누락 → `7` 이 `_ => Solid` 로 떨어짐. (파서/IR `ColumnDef.separator_type` 및 SVG `<line>` 출력은 정상.)

## 변경
`src/renderer/layout.rs::build_column_separators`:
- `separator_type` → `StrokeDash` 매핑에 `6 => Dash`(LongDash 근사), `7 => Dot`(Circle/원형 점선) 추가. `doc_info.rs:294` 의 `line_type` 의미와 정합. 8+ (이중선/물결 등) 은 종전대로 Solid 대체.

## 결과
- shortcut.hwp 2~8페이지 단 구분선 `<line ... stroke-dasharray="2 2"/>` 로 점선 렌더 (이전: `stroke-dasharray` 없음 = 실선).
- `cargo test` 전건 통과.

비고: HWP 의 "원형 점선"(type 7) 은 작은 둥근 점이나 `StrokeDash` enum 에 RoundDot 변형이 없어 `Dot`(사각 점선 `2 2`)로 근사. 시각상 충분히 점선으로 보이며 별도 enum 추가는 범위 외.

다음: Stage 4 — 섹션 헤더 바 좌측 위치 어긋남(결함 #2).
