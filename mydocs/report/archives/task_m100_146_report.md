# 최종 결과보고서: text-align.hwp SVG ↔ 한컴 PDF 렌더링 차이 보정

- **타스크**: [#146](https://github.com/edwardkim/rhwp/issues/146)
- **마일스톤**: M100
- **브랜치**: `local/task146`
- **작성일**: 2026-04-23
- **상위 문서**:
  - 수행계획서: `mydocs/plans/task_m100_146_v2.md`
  - 구현계획서: `mydocs/plans/task_m100_146_v2_impl.md`
  - 단계 보고서: `task_m100_146_stage1.md`, `task_m100_146_stage2.md`

## 1. 문제 요약

작업지시자가 제공한 `text-align.hwp` 1페이지 문서에 대해 `rhwp export-svg` 출력과 한컴 오피스 PDF 출력 `text-align.pdf` 를 나란히 렌더링한 결과, 제목 "□ 국어 변화 관측 및 다양성 보존 체계 마련" 에서 □ 와 국 사이 공백이 사라진 것처럼 보이며 제목 전체가 좌측으로 약 11px 치우쳐 나타났다.

## 2. 원인 규명

### 2.1 초기 가설 (수행계획서 원본)

최초에는 3가지 가설을 세웠다:
- ① 자간(-8%) × 공백 문자 상호작용 버그
- ② `Alignment::Justify` 가 SVG 에 미반영
- ③ 음수 indent(hanging) 후속 줄 x 기준점 어긋남

### 2.2 PDF 좌표 정밀 비교로 가설 ②③ 기각

`mutool draw -F stext` 로 PDF 내 모든 문자의 절대 좌표를 추출해 SVG 좌표와 1:1 비교.

| 항목 | PDF (pt) | SVG (pt 환산) | 차이 | 판정 |
|------|---------|--------------|------|------|
| 본문 line1 끝 "범" | 538.15 | 538.27 | 0.12 pt | Justify 정상 |
| 본문 line2 시작 "조" | 85.99 | 86.03 | 0.04 pt | Hanging indent 정상 |
| 제목 "국" 시작 | 79.04 | 70.80 | **-8.24 pt** | 글자 폭 측정 오류 |

②③ 은 실제 버그가 아니었고 ① 의 원인도 자간×공백이 아니라 **"□" 자체의 폭 측정**이었다.

### 2.3 진짜 원인

`src/renderer/layout/text_measurement.rs:845-858` `is_fullwidth_symbol` 함수는 HWP 전각 심볼 판별용이나, **Unicode Geometric Shapes 블록 (U+25A0-U+25FF) 이 리스트에서 누락**.

`'□'` (U+25A1) 에 대해:
- `is_cjk_char('□')` = false (Hangul/CJK 범위 아님)
- `is_fullwidth_symbol('□')` = false (누락)
- 결과: `base_w = font_size * 0.5` (반각)
- 한컴 PDF 는 HY헤드라인M 폰트의 실 메트릭으로 15.95pt=21.27px 로 측정

→ □ advance 가 em/2 만큼 작아져 후속 글자 전체가 좌측으로 붕괴.

Geometric Shapes 는 HWP 문서에서 섹션 머리 기호(□/■/▲/◆/○ 등) 로 매우 빈번히 사용되므로 누락이 광범위한 잠재 버그로 이어질 소지가 있었다.

## 3. 수정 내역

### 3.1 소스 (1줄 추가)

`src/renderer/layout/text_measurement.rs`:

```rust
|| ('\u{2460}'..='\u{24FF}').contains(&c) // Enclosed Alphanumerics (①②③ 등)
|| ('\u{25A0}'..='\u{25FF}').contains(&c) // Geometric Shapes (□■▲◆○ 등, 섹션 머리 기호)  ← 신규
|| ('\u{2600}'..='\u{26FF}').contains(&c) // Miscellaneous Symbols (☆★ 등)
```

이 함수는 네이티브(`EmbeddedTextMeasurer`) 와 WASM(`WasmTextMeasurer`) 측정 경로 3곳에서 참조되므로 한 줄 추가가 양쪽에 모두 반영된다.

### 3.2 단위 테스트 (2건)

`src/renderer/layout/tests.rs`:
- `test_geometric_shapes_treated_as_fullwidth`: □ ■ ▲ ▼ ◆ ○ ● ◇ 8개 글자에 대해 `compute_char_positions` 결과 advance == font_size 확인.
- `test_square_bullet_with_space_preserves_layout`: "□ 가" + letter_spacing=-1.6 (자간 -8%) 조합에서 positions = [0, 18.4, 26.8, 45.2] 확인 — 수정 전에는 [0, 8.4, 16.8, 35.2] 로 붕괴.

### 3.3 svg_snapshot golden 업데이트

`tests/golden_svg/form-002/page-0.svg` 6줄 변경 (2 위치 × 3줄):

```
< 개 x=95.59    →   > 개 x=104.59   (+9.00 px)
< 념 x=113.69  →   > 념 x=122.69
< 발 x=113.69  →   > 발 x=122.69
< 내 x=131.80  →   > 내 x=140.80
< 용 x=149.91  →   > 용 x=158.91
```

변경 사유: form-002 본문의 "□ 개념 " / "□ 개발내용" 섹션 머리에서 □ 전각 인식으로 후속 글자가 정상 advance 된 결과. **PDF 기준으로 가까워지는 방향의 수정**이므로 golden 을 새 상태로 갱신.

## 4. 검증 결과

### 4.1 samples/text-align.hwp 좌표 수렴

| 글자 | 변경 전 (px) | 변경 후 (px) | PDF 환산 | 오차 |
|------|------------|------------|---------|------|
| □ 시작 | 75.58 | 75.58 | 75.48 | 0.10 |
| 국 시작 | 94.40 | **105.40** | 105.39 | **0.01** |
| 어 시작 | 114.88 | 125.88 | 125.85 | 0.03 |
| 변 시작 | 145.17 | 156.17 | 156.33 | 0.16 |
| 화 시작 | 165.65 | 176.65 | — | — |

제목 전체가 한컴 PDF 와 **1 px 이내**로 수렴.

### 4.2 자동 테스트

| 항목 | 결과 |
|------|------|
| `cargo test --lib` | 929 passed / 14 failed (14건은 `serializer::cfb_writer` / `wasm_api` 의 기존 실패, 본 타스크 무관 — 스태시 후 재측정으로 사전 확인) |
| `cargo test --test svg_snapshot` | 3 passed (form-002 golden 업데이트 후) |
| `cargo clippy --lib -- -D warnings` | 경고 없음 |

### 4.3 스모크 스위프

Geometric Shapes 사용 여부를 확인한 샘플 3건:
- `samples/biz_plan.hwp`: Geometric Shapes 미사용. 렌더 정상.
- `samples/exam_kor.hwp`: 5개 페이지에 □ 계열 사용. 렌더 정상. 페이지 10 시각 확인 통과.
- `samples/draw-group.hwp`: 렌더 정상.

### 4.4 시각 비교 (PDF ↔ SVG 150dpi)

`output/compare/text-align/` :
- `pdf-1.png`: 한컴 PDF 150dpi
- `svg-chrome150.png`: 수정 전 SVG 150dpi
- `svg-chrome150-after.png`: 수정 후 SVG 150dpi

수정 후 이미지에서 제목 "□ 국어..." 공백이 뚜렷이 표시되어 PDF 와 육안 동등. (제목 글자 굵기 차이는 HY헤드라인M 폰트 치환 한계로 본 타스크 범위 외.)

## 5. 커밋 이력

| 커밋 | 설명 |
|------|------|
| `bca5599` | Task #146 단계1: 샘플 편입 + 비교 파이프라인 고정 |
| `ffaf488` | Task #146: Geometric Shapes(U+25A0-U+25FF) 를 전각 심볼로 처리 |
| `8642159` | docs: Task #146 v2 계획서 + 단계2 보고서 |
| (본 커밋) | Task #146 단계3: golden 업데이트 + 최종 결과보고서 |

## 6. 잔여 / 별도 이슈

- **제목 HY헤드라인M 폰트 굵기**: PDF 는 한컴 번들 HY헤드라인M (heavy display) 로 굵고 크게 렌더되지만 SVG 는 Malgun Gothic fallback 으로 보통 굵기. `mydocs/tech/font_fallback_strategy.md` 범위로 분리.
- **다른 유니코드 심볼 블록 미반영 가능성**: 화살표(U+2190-U+21FF), 수학 연산자(U+2200-U+22FF) 등이 HWP 문서 섹션 머리로 쓰이는 사례가 관찰되면 동일 방식으로 추가. 현재는 실사례 없어 본 타스크에 포함하지 않음.

## 7. 회귀 방지

- 단위 테스트 2건이 범위 축소/삭제 시 즉시 실패한다.
- `svg_snapshot` form-002 golden 이 □ 포함 라인의 좌표를 감시한다.
