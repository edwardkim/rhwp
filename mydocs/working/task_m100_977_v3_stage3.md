# Stage 3 보고서 — Task #977 v3: WASM 미등록 폰트 한글 폭 native 동기화

- 브랜치: `local/task977-v2`
- 수정: `src/renderer/layout/text_measurement.rs` (`measure_hangul_width_hwp` 폴백 1 곳)
- 산출물: 본 문서 + 신규 WASM 빌드 + rhwp-studio/public 동기화

## 잔존 본질 진단 (1.png Stage 3 보고)

작업지시자가 hard-refresh 후에도 1.png 페이지 3 (Ⅴ. 제안안내 ~ Ⅵ. 제안서 작성기준) 의 페이지번호 (68, 69, 72, 74, 77, 80) 가 visible 다중 px 어긋남을 보고 — Stage 2 적극 fix (`measure_char_width_hwp` 의 JS 폴백 제거) 만으로는 해소 안 됨.

### 문서 분석 — `samples/2. 인공지능(AI)…HWPX`

TOC 모든 행 (sec 0, pi=22~25, 42~48) 의 ParaShape · TabDef · CharShape:
- ParaShape `ps_id=498`, **LEFT** tab (type=0), TabDef pos=90432 HU = **602.88 px** (text-relative), fill=3 (점선 leader).
- CS 946 (선행공백·번호·제목): **나눔바른고딕** 15pt
- CS 89 (탭): 맑은 고딕 13pt
- CS 947/948/950 (페이지번호 디지트): **나눔바른고딕** 15pt
- 한컴은 매 행 `tab_extended[0]` 에 "선행텍스트폭 + ext[0] = 602.88 (LEFT tab pos)" 가 되도록 **행별로 다른 tab 폭** 을 미리 저장 (예: pi=22 ext[0]=31164 HU=415.52 px, pi=23 ext[0]=25916 HU=345.55 px).
- 즉 한컴의 "선행텍스트폭" metrics 와 우리 metrics 가 정확히 일치해야 (= bbox.x_for_tab_run + ext[0] == 602.88) 디지트가 col 우측에서 일관 정렬.

### Native (export-svg) — 정렬 OK

`paragraph_layout.rs` 의 estimate pass 로 측정한 run 별 bbox.x · full_width 실측 (`RHWP_DBG_TASK977=1`):

```
[T977] pi=22 run_idx=2 text="\t1"  bbox.x=261.59 full_w=425.00 x_after=686.59
[T977] pi=23 run_idx=2 text="\t1"  bbox.x=331.59 full_w=355.00 x_after=686.59
[T977] pi=24 run_idx=2 text="\t2"  bbox.x=231.59 full_w=455.00 x_after=686.59
[T977] pi=25 run_idx=2 text="\t2"  bbox.x=231.59 full_w=455.00 x_after=686.59
```

bbox.x + ext[0] = 686.59 (page 2) 또는 696.59 (page 3) 로 **모든 행 동일** → 디지트 좌측 위치 일관 (시각 정렬).

### WASM (rhwp-studio) — 어긋남

`FONT_METRICS` 에 **나눔바른고딕** 없음 (NanumGothic / NanumMyeongjo 만 존재). 따라서 `measure_char_width_embedded("나눔바른고딕", '가', ...)` 가 `None` 반환 → 폴백:

- Native `EmbeddedTextMeasurer` (text_measurement.rs:233-249) — 미등록 폰트 CJK = `font_size` (1.0 em)
- WASM `WasmTextMeasurer` → `measure_hangul_width_hwp` (text_measurement.rs:842~) — 미등록 폰트면 **`cached_js_measure('가')`** (브라우저 fallback 폰트의 '가' 실측) 사용

→ WASM 한글 폭 = 브라우저 fallback 폰트의 '가' 폭 ≠ native font_size (1.0 em).

선행텍스트의 **한글 개수 차이** × (WASM_한글폭 − 한컴_한글폭) 만큼 bbox.x_run_tab 가 paragraph 별로 어긋남:

- pi=42 "  1. 입찰안내 사항 " (4 hangul), pi=43 "  2. 입찰서류 및 제안서 제출 안내 " (10 hangul) 등 행별 한글수 다름
- 한컴 ext[0] 는 "tab_pos - 한컴_선행텍스트폭" 로 저장 — WASM 의 어긋난 선행텍스트폭과 합산 시 (한글개수차 × 폭차) 만큼 디지트 x 가 어긋남

## 수정 (Stage 3 최종)

`measure_hangul_width_hwp` 의 JS 폴백을 native heuristic 으로 교체:

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

- 등록 폰트(맑은 고딕, HCR Batang 등): 기존 `measure_char_width_embedded` 경로(불변).
- 미등록 폰트(나눔바른고딕 등): **font_size (1.0 em) 폴백** — native EmbeddedTextMeasurer 와 동일.
- JS `cached_js_measure('가')` 폴백 제거 → 브라우저 fallback 폰트 변동 차단.

매개변수 `measure_font` 는 더 이상 한글 폭 산출에 쓰지 않으므로 `_measure_font` 로 silent (서명 보존 — 호출자 영향 없음).

## 검증

- **KTX 회귀 가드** (`issue_874_ktx_toc_page_number_right_align`): **1/1 PASS** (PR #1026 영역 유지)
- **golden SVG 8 종** (`svg_snapshot`): **8/8 PASS** (form_002, issue_147/157/267/617/677, table_text, determinism)
- `cargo check --lib --target wasm32-unknown-unknown`: **OK**
- Docker WASM 빌드 성공 (pkg/rhwp_bg.wasm md5=ad1444a9..., 4.95 MB)
- pkg/ → rhwp-studio/public/ 동기화 (md5 일치)

## 비회귀 분석

미등록 폰트 한글 폭 차이는 다음에 영향 가능:
- 줄바꿈 결정 (compose 단계의 width 누적): WASM 도 native 와 동일 폭 사용 → 줄바꿈도 동일.
- 표 셀 자동 너비 계산: 본 패치는 measure 결과를 native 와 동기 → native 와 동일 표 너비 산출.
- 라인 전체 폭 측정: 영향 없음 (동기).
- 등록 폰트(맑은 고딕 등): 분기 미진입 → 무회귀.

## 작업지시자 검증 요청

1. 브라우저 hard-refresh (Ctrl+Shift+R) — pkg/ 새 빌드 (md5 ad1444a9...) 로드 확인.
2. 같은 문서의 TOC 페이지 (page 2 Ⅰ. 사업안내, page 3 Ⅴ. 제안안내) 재캡처.
3. 결과:
   - **정렬 OK** → Stage 3 종결, 최종 결과보고서 작성 진행.
   - **잔존** → 추가 진단 (한글 외 다른 글자 폭, 또는 Canvas2D fillText 경로의 별도 측정).
