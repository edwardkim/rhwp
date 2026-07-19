---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-257/README.md
last_verified: 2026-07-16
---

# text-align-2.hwp SVG ↔ 한컴 PDF 비교 보고

- **작성일**: 2026-04-23
- **샘플**: `samples/text-align-2.hwp` (작업지시자 제공, Task #257 로 samples 편입), `samples/text-align-2.pdf` (한컴 PDF 150dpi)
- **목적**: Task #146 v1~v4 수정 후 유사 샘플 회귀 검증 + 잔존 차이 식별
- **산출물**:
  - `output/svg/text-align-2/text-align-2.svg`
  - `output/compare/text-align-2/pdf-1.png` (한컴 PDF 150dpi)
  - `output/compare/text-align-2/svg-chrome150.png` (rhwp 150dpi)

## 1. 문서 구조

총 12 문단, 2 섹션 (dump 기준).

| 문단 | 내용 | 비고 |
|------|------|------|
| 0.0 | 빈 문단 + 구역나누기 | |
| 0.1 | `□ 국어 변화 관측 및 다양성 보존 체계 마련` | text-align.hwp 동일 |
| 0.2 | ` ㅇ 세대별·지역별 언어 변이 조사 항목 선정, …` | 동일 |
| 0.3 | 공백 4개 + TAC 표 (3×2) | 동일 |
| 0.4 | `     * 3세대: 20대, 50대, 80대` | 동일 |
| 0.5 | `    ** 10지점: 고양시, 춘천시, …` | 동일 |
| 0.6 | 빈 문단 | 섹션 구분 |
| 0.7 | `□ 특수언어 진흥 기반 마련으로 언어 소외 계층의 언어권 확대` | **신규 제목** |
| 0.8 | ` ㅇ 시·청각장애인의 의사소통 환경 개선을 위한 연구, 제도, 지원 확대` | **신규 본문** |
| 0.9 | `   * 수어사전 집필(934개), 수어교원 자격심의위원회 운영(3회), …` | **신규 주석** |
| 0.10, 0.11 | 빈 문단 | trailing |

## 2. 비교 결과

### 2.1 일치 (Task #146 v1~v4 수정 반영 확인)

| 항목 | 판정 근거 |
|------|---------|
| 두 제목 모두 bold 렌더 | `□ 국어…` / `□ 특수언어…` 시각적으로 굵게 (v4 heavy face → visual bold) |
| 제목 `□` 후 공백 | 제목 글자 간격 PDF 일치 (v2 Geometric Shapes 전각) |
| 표 x 좌표 | SVG 첫 셀 clip-rect x=109.59 px (v3 TAC 선행 공백 반영) |
| 표 셀 헤더 `구분`/`내용` bold | PDF 와 일치 |
| 섹션 2 제목 bold | HY헤드라인M 계열 heavy face 인식 |
| 섹션 2 본문 줄바꿈 위치 | `수어사전 집필(934개), … 지원(564회)` 뒤에서 줄바꿈, `및 품질 관리…` 부터 둘째 줄 — PDF 와 일치 |
| 섹션 2 `ㅇ` / `*` 들여쓰기 | 각 문단 indent 정상 |
| 2개 섹션 간 수직 간격 | `spacing_before` 반영 정상 |

**Task #146 에서 고친 3대 버그(Geometric Shapes / TAC 표 x / heavy face bold) 는 text-align-2 에서도 회귀 없이 동작.**

### 2.2 잔존 차이 (새 버그 후보)

| 항목 | PDF | rhwp SVG | 성격 |
|------|-----|---------|------|
| 표 셀 숫자 콤마 뒤 공백 | `1,000항목` | `1, 000항목` | **실제 버그 후보 (narrow glyph 간격)** |
| 표 셀 숫자 콤마 뒤 공백 | `30,000항목` | `30, 000항목` | 동일 |
| 표 셀 헤더 중점 | `어휘·표현` | `어휘· 표현` | 동일 (중점 narrow glyph) |
| 제목/본문 글리프 획 두께 | HY헤드라인M 본연 | Malgun Gothic Bold fallback | 폰트 치환 한계(별도 범위) |

## 3. 새 버그 후보 분석: narrow glyph 간격 과다

### 증상

콤마(`,`) · 중점(`·`) 등 **실제 글리프 폭이 반각(em/2) 보다 훨씬 좁은 narrow glyph** 뒤에 오는 글자가 PDF 보다 ~3-5 px 더 오른쪽에 배치.

### 원인 가설

`src/renderer/layout/text_measurement.rs:280-290` `compute_char_positions` 의 음수 자간 min-clamp 로직:

```rust
if style.letter_spacing + style.extra_char_spacing < 0.0 {
    let min_w = base_w * ratio * 0.5;
    w = w.max(min_w);
}
```

- 콤마 `base_w` = `font_size × 0.5` (반각 휴리스틱)
- 자간 -8% 시 `w = 반각 + (-8% × font_size)`
- `min_w = 반각 × 0.5 = font_size × 0.25`
- 실제 콤마 글리프는 `font_size × 0.15` 수준일 수 있음 → `min_w` 가 오히려 **과도한 최저 advance** 로 작용, 콤마 뒤 공백 확장

폰트 메트릭 DB (`measure_char_width_embedded`) 에 콤마/중점 글리프의 실제 advance 값이 있으면 해당 경로로 처리되지만, 미등록 폰트(HY헤드라인M 등) 에서는 반각 휴리스틱으로 떨어지는 것이 원인.

### 영향 범위

- 표 셀 내 숫자 (`1,000`, `30,000` 등)
- 일반 본문 내 콤마·중점이 다수 쓰이는 문서
- 콤마·중점 빈도가 높은 시험지·보고서·사양서 등

### 수정 접근 후보

A. `base_w` 휴리스틱 개선: 유니코드 범주별(`Punctuation`) narrow glyph 에 대해 `font_size × 0.3` 수준의 작은 값 반환
B. `min_w` 하한값 폐지: per-char clamp 를 narrow glyph 에 적용하지 않음
C. 폰트 메트릭 DB 에 공통 narrow glyph (`,` `.` `·` `:` `;`) 실제 값 추가

가장 낮은 리스크: **A + B 조합** (narrow glyph 감지 후 clamp 우회).

## 4. 권장 후속 조치

1. 본 문서를 근거로 별도 이슈(예: `#147` "narrow glyph 뒤 char advance 과다") 등록
2. 본 타스크(#146) 는 현재 상태로 종결 (devel push 완료 · PR 대기)
3. 별도 이슈는 [#257](https://github.com/edwardkim/rhwp/issues/257) 로 등록되어 `local/task257` 브랜치에서 수행 중

## 5. 참조 명령

```bash
# 재생성
cargo run --bin rhwp -- export-svg samples/text-align-2.hwp -o output/svg/text-align-2/
mutool convert -O resolution=150 -o output/compare/text-align-2/pdf-%d.png samples/text-align-2.pdf

CHROME="/c/Program Files/Google/Chrome/Application/chrome.exe"
WINSVG=$(cygpath -w "$PWD/output/svg/text-align-2/text-align-2.svg")
WINOUT=$(cygpath -w "$PWD/output/compare/text-align-2/svg-chrome150.png")
"$CHROME" --headless --disable-gpu --no-sandbox \
  --force-device-scale-factor=1.5625 \
  --window-size=794,1123 --hide-scrollbars \
  --default-background-color=ffffffff \
  --screenshot="$WINOUT" "file:///${WINSVG//\\//}"
```
