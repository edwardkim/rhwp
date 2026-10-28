# Task M100 #2277 단계별 완료보고서 — 4단계: 범례 스와치 글리프 (SwatchKind)

- 이슈: #2277 (C2a stock HLC 렌더 + 2D fidelity 정합)
- 브랜치: `local/task2277`
- 단계: 4/5
- 작성일: 2026-07-15

## 구현 내용 (`src/ooxml_chart/renderer.rs`)

C1d 시각판정(2026-07-10)에서 발견된 "범례 스와치에 마커 글리프 없음" 갭 해소.
근거는 stage3 전수 실측에서 전 파일 재확인됨: 표식 라인 = 선분+플롯 마커와 동일
글리프(—◆—), 표식만 분산형 = 글리프만, stock 시/고/저 = 스와치 없음·종가만 글리프.

- **`SwatchKind` enum** {Square / LineOnly / LineGlyph(si) / GlyphOnly(si) / Blank} —
  글리프 인덱스는 **원 계열 인덱스**(역순 나열과 무관하게 플롯 마커·팔레트와 동일
  형상/색 유지, stage3 역순 나열 이전에 매핑).
- **`swatch_kind(chart, series, i)` 결정 함수**:
  - Line: 순수 라인 차트 && `line_markers` → LineGlyph / 그 외(콤보 라인 포함 —
    render_combo는 마커 미렌더) → LineOnly (현행 유지)
  - Scatter: 스타일 flags로 (선∧표식)=LineGlyph / 표식만=GlyphOnly / 선만=LineOnly
  - Stock: `marker_symbol ∈ {Auto, Named}`(종가) → GlyphOnly / 그 외(시/고/저
    `c:symbol val="none"`) → **Blank**
  - Column/Bar/Pie: Square
- **`legend_items`** 반환 `(라벨, 색, OoxmlChartType)` → `(라벨, 색, SwatchKind)`.
- **`push_legend_swatch`** match 재작성 — Square/LineOnly는 **종전 출력 바이트
  그대로**(issue_1882 `width="10" height="10"` 필터 보호), LineGlyph = 14px 선분 +
  중앙 글리프(r=3.0), GlyphOnly = 글리프만(r=3.5), Blank = 무출력(텍스트 x=+18
  오프셋 불변 — 정답지의 시/고/저/종 라벨 좌정렬 유지).
- 글리프는 **별도 클래스 `hwp-legend-glyph`** — 플롯 마커(`hwp-chart-marker`) 카운트
  오염 차단(issue_2129 12개 핀 보호). Stage 1의 `push_marker` class 파라미터 활용.

## 테스트 (RED→GREEN)

| 테스트 | 단언 | 종별 |
|--------|------|------|
| `test_legend_swatch_marker_line_has_glyph` | 표식 라인: 글리프 3+선분 3, 플롯 마커 12 불변 | 신규(유닛) |
| `test_legend_swatch_plain_line_no_glyph` | 무표식 라인: 선분만 (현행 핀) | 신규(유닛) |
| `test_legend_swatch_scatter_marker_only_glyph_only` | 표식만 분산형: 글리프만·선분 0 | 신규(유닛) |
| `test_legend_swatch_scatter_line_marker_line_glyph` | 선+표식 분산형: 선분+글리프 | 신규(유닛) |
| `test_legend_swatch_stock_blank_except_close` | stock: rect 0·선분 0·글리프 1(종가)·라벨 4 유지 | 신규(유닛) |
| `test_legend_swatch_square_unchanged_for_bars` | 막대: 10×10 사각형 3·글리프 0 (issue_1882 보호) | 신규(유닛) |
| `issue_2277_stock::stock_legend_swatches_blank_except_close_glyph` | 실 코퍼스 stock 4파일: 글리프 1 | 신규(통합) |
| `issue_2277_legend_order::marker_line_and_scatter_legend_swatches_have_glyphs` | 표식 라인 글리프 3 / 표식만 분산형 글리프 2 (hwp/hwpx) | 신규(통합) |

## TDD 절차 준수

RED (구현 전) — 4건이 정확한 사유(글리프 미방출·stock은 사각형 스와치)로 실패,
핀 2건(무표식 라인·막대 사각형)은 현행 단언으로 선통과:

```
test_legend_swatch_marker_line_has_glyph            FAILED
test_legend_swatch_scatter_marker_only_glyph_only   FAILED
test_legend_swatch_scatter_line_marker_line_glyph   FAILED
test_legend_swatch_stock_blank_except_close         FAILED
→ 89 passed; 4 failed
```

GREEN: `cargo test --lib ooxml_chart` **93/93**, 차트 통합 6스위트(신규 통합 2건 포함)
전부 통과.

## 검증 결과

- [x] 유닛 93/93 + 차트 통합 6스위트 (stock 3·legend_order 3·2129 6·scatter 1·1882 4·1453 2)
- [x] issue_2129 마커 카운트 무오염 (별도 클래스 검증 — 유닛 `플롯 마커 12 불변` 단언)
- [x] issue_1882 Square 10×10 문자열 불변 (유닛 단언 + 통합 통과)
- [x] **전체 `cargo test` 전수 (클린 로그 실행): 265스위트 전부 ok — 3,186 passed /
  0 failed, cargo exit 0** + `cargo clippy --all-targets -- -D warnings` 무경고 (exit 0)
- [x] fmt: 수정 파일 rustfmt, `cargo fmt --check` 지적 0건

## 다음 단계

5단계 — 특이케이스 0.5축 게이트(가로&&1카테고리 → step/2) + `line3DChart` 방어
라우팅 + 코퍼스 28종 export-svg 총괄 + 정답지 대조표 → 작업지시자 시각판정.
