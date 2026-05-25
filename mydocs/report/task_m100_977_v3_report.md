# 최종 결과보고서 — Task #977 v3: WASM 미등록 폰트 한글 폭 native 동기화

- 이슈: edwardkim/rhwp#977 (Skia replay 경로 — 선두 공백 CharShape가 다른 목차 문단의 개요번호 x 우측 밀림)
- 브랜치: `local/task977-v2` (stream/devel `fbfcf682` 기준)
- 이전 시도: #980 (자진 close) · #1045 (메인테이너 close — PR #1026 본질 흡수 판단)
- Stage: 1 (진단 + 설계) · 2 (구현 + 검증) · 3 (잔존 진단 + 최종 수정)

## 본질

**한 문장**: WASM `measure_hangul_width_hwp` 가 미등록 폰트의 한글 폭을 JS `cached_js_measure('가')` (브라우저 fallback 폰트 실측) 로 폴백 → native `EmbeddedTextMeasurer` 의 `font_size` (1.0 em CJK 휴리스틱) 와 다른 폭 산출 → 한컴이 `tab_extended[0]` 에 "tab_pos − 한컴_선행텍스트폭" 으로 저장한 행별 tab 폭과 합산 시, 행별 한글 개수 차이 × 한글 폭 차 만큼 디지트 x 가 누적 어긋남.

## 진단 흐름 (Stage 1 → 3)

### Stage 1 — 1차 진단

- PR #1026 (narrow_punct 분기 + native/WASM 동기, 5/21 머지) 흡수 후 잔존 본질을 "미등록 폰트 공백 폭" 으로 가설.
- `measure_char_width_hwp` (WASM) 폴백 체인 분석: embedded → 한글 음절 → narrow_punct → JS Canvas.

### Stage 2 — 1차 수정 (불완전)

`measure_char_width_hwp` 의 일반 폴백을 native heuristic 으로 통일 (JS Canvas 폴백 제거):
```rust
if super::is_cjk_char(c) || super::is_fullwidth_symbol(c) { return font_size; }
font_size * 0.5
```

- KTX 회귀 1/1 + golden 8/8 통과, native 무회귀.
- **단, 한글 음절 경로 (`measure_hangul_width_hwp`) 는 미손**.

### Stage 3 — 잔존 본질 확정 + 최종 수정

작업지시자 시각 검증 (1.png, `samples/2. 인공지능(AI)…HWPX` page 2 Ⅰ.사업안내 · page 3 Ⅴ.제안안내) — Stage 2 빌드에서도 잔존 보고. 추가 진단:

1. **문서 분석** — TOC 모든 행 ParaShape ps_id=498, LEFT tab (type=0) pos=602.88 px, fill=3 점선 leader. CS 946/947/948/950 모두 **나눔바른고딕** 15pt.
2. **네이티브 estimate 실측** (`RHWP_DBG_TASK977=1` + export-svg): bbox.x + ext[0] = 686.59 (page 2) / 696.59 (page 3) 모든 행 동일 → 정렬 OK.
3. **`tab_extended[0]` 정밀 조사**: 한컴이 행별로 다른 ext[0] 저장 (pi=22=31164 HU=415.52 px, pi=23=25916 HU=345.55 px, …) — "선행텍스트폭 + ext[0] = tab_pos" 정합 의도.
4. **FONT_METRICS 검색**: NanumGothic·NanumMyeongjo 있음, **NanumBarunGothic (나눔바른고딕) 없음**.
5. **WASM 한글 폭 폴백 식별**: `measure_hangul_width_hwp` (text_measurement.rs:842~) 가 미등록 폰트에 `cached_js_measure('\u{AC00}')` 사용 → 브라우저 fallback 폰트 (보통 시스템 sans-serif) '가' 실측 폭 채택.
6. **누적 시뮬레이션**: 한글 1자당 ~1px 차이만 나도, 한글 7~10자 선행 시 5~10px 디지트 어긋남 → 사용자 시각 보고와 정합.

## 수정 (`src/renderer/layout/text_measurement.rs`)

```rust
pub(super) fn measure_hangul_width_hwp(
    _measure_font: &str,
    font_family: &str,
    bold: bool,
    italic: bool,
    font_size: f64,
) -> i32 {
    if let Some(w) =
        super::measure_char_width_embedded(font_family, bold, italic, '\u{AC00}', font_size)
    {
        return (w * 75.0).round() as i32;
    }
    // native EmbeddedTextMeasurer 동기화: 미등록 폰트의 한글(CJK)은 font_size (1.0 em).
    (font_size * 75.0).round() as i32
}
```

추가로 Stage 2 의 `measure_char_width_hwp` 적극 fix (JS 폴백 제거) 도 그대로 유지 — 디지트(라틴 문자) 폭 동기화 보존.

## 검증

| 항목 | 결과 |
|------|------|
| `issue_874_ktx_toc_page_number_right_align` (PR #1026 회귀 가드) | **1/1 PASS** |
| `svg_snapshot` (golden SVG 8 종) | **8/8 PASS** |
| `cargo check --lib --target wasm32-unknown-unknown` | OK |
| Docker WASM 빌드 → pkg/rhwp_bg.wasm | md5 ad1444a9..., 4.95 MB |
| pkg/ → rhwp-studio/public/ 동기화 | md5 일치 |
| `cargo fmt` (변경 파일 한정) | clean |
| **작업지시자 시각 재현 (rhwp-studio, samples/AI 재정통합시스템)** | **해결 확인** ✓ |

## 변경 파일

- `src/renderer/layout/text_measurement.rs` (`measure_hangul_width_hwp` 폴백 + Stage 2 `measure_char_width_hwp` 폴백 통일)
- `rhwp-studio/public/rhwp.js` · `rhwp_bg.wasm` (WASM 재빌드 동기화)
- `mydocs/plans/task_m100_977_v3.md`
- `mydocs/working/task_m100_977_v3_stage1.md`, `_stage2.md`, `_stage3.md`
- `mydocs/report/task_m100_977_v3_report.md` (본 문서)

## 비회귀 분석

- **등록 폰트**: `measure_char_width_embedded` 가 `Some` 반환 → 본 분기 미진입. 무회귀.
- **미등록 폰트 한글**: 종전 JS '가' 실측 → 현재 `font_size` (1.0 em). native 와 동일 → native SVG 결과로 무회귀 검증 (KTX + golden 8/8).
- **줄바꿈·표 너비 등 파생 측정**: 동일 폭 사용으로 native 와 일관 → 무회귀.
- **`measure_font` 매개변수**: 한글 폭 산출에서 더 이상 쓰지 않음. 서명 보존 (`_measure_font` 로 silent) — 호출자 무영향.

## 교훈

- "동일 본질" 의 fix 라도 폴백 경로가 **함수별로 분기** 되어 있으면 한 곳만 통일해도 본질이 다른 함수에서 잔존할 수 있다. Stage 2 적극 fix 가 `measure_char_width_hwp` 만 손대고 `measure_hangul_width_hwp` 를 미손한 것이 그 사례.
- 한컴은 LEFT tab + tab_extended[0] 조합으로 "한컴이 측정한 선행텍스트 폭 + ext[0] = tab_pos" 가 되도록 미리 계산해 저장한다. 우리 metric 이 한컴과 다른 폭을 산출하면 ext[0] 더해도 일관 정렬 깨짐.
- 시각 보고 (1.png) 만으로 단정 어려운 root cause 는 (a) native SVG 동일 케이스 비교, (b) `paragraph_layout` 의 estimate 실측, (c) `tab_extended` 원본 덤프 의 3 단계 진단으로 좁힐 수 있다.

## 차기

- 본 PR 머지 후 1.png 같은 사례가 다시 보고되면 `measure_*_width_hwp` 계열 함수 전체를 native EmbeddedTextMeasurer 와 cross-check 하는 자동 회귀 테스트 추가 검토.
- 나눔바른고딕 등 자주 쓰이는 미등록 한글 폰트를 `FONT_METRICS` 에 추가하는 별도 이슈 가치 (정확도 향상 + JS 호출 추가 절감).
