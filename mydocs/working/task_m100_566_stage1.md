# Task #566 Stage 1 — 원인 분석 보고서

- **이슈**: [#566](https://github.com/edwardkim/rhwp/issues/566) `[m100] exam_science.hwp 7번 표 셀 내부 ㉠/㉡ 베이스라인 위치가 위로 시프트됨`
- **브랜치**: `local/task566`
- **작성일**: 2026-05-04

---

## 1. 시각적 재현 (96 dpi 픽셀 레벨)

| 측정 | SVG (`output/exam_science_002.svg`) | HWP PDF (`samples/exam_science.pdf` p.2) |
|------|----------|----------|
| 표(4×7) 셀 사각형 | y=512.37, h=22.88 | y≈512.37, h≈22.88 (동일) |
| 행 0 ㉠ 글리프 가시 y 범위 | 517 – 531 | 517 – 530 |
| 행 0 ㉠ 밑줄 y | **531.19** | **532.00** |
| 행 1 "직선형" 글리프 y 범위 | 540 – 554 | 541 – 555 |
| 행 0 ㉠ SVG `<text y>` (baseline) | **529.19** | (PDF 상응 baseline ≈ 530.0~) |

**결론**: SVG 가 PDF 대비 **약 1.0–1.5 px (75–110 HU) 위로 시프트**. 시프트는 표 전 셀에 동일하게 적용되며 ㉠/㉡ 만의 문제는 아님. 다만 원형 글리프(㉠/㉡)의 시각적 특성상 변위가 더 도드라진다.

비교 이미지: `/tmp/sbs.png` (좌SVG/우PDF), `/tmp/diff.png` (적=SVG only, 청=PDF only).

## 2. 가설별 검증

| 가설 | 결과 | 근거 |
|------|------|------|
| 5. SVG transform/scale 분기 차이 | **기각** | 셀 내 ㉠/㉡ 와 본문 ㉠/㉡ 모두 `transform="translate(...) scale(0.95,1)" font-family="HY신명조,...,serif" font-size="15.333"` 동일 출력. (line 382 vs line 367 of svg) |
| 2. CharShape 세로 시프트(`char_offsets`) | **기각** | cs_id=117 (㉠/㉡ 셀) 와 cs_id=10 (직선형 셀) 모두 `char_offsets=[0,0,0,0,0,0,0]`. |
| 4. ㉠/㉡ 글리프 메트릭 의존 | **부분 관련** | 시프트는 모든 셀 텍스트에 동일 (직선형/HOF/굽은형 모두 ~1px 위). ㉠/㉡ 글리프의 원형 윤곽 때문에 시각적 도드라짐만 강함. |
| 3. valign / padding 처리 누락 | **기각** | aim=false, cell.padding=(283,283,283,283) > table.padding=(141,141,510,510)의 top — `prefer_cell_axis` 결과 cell.padding=283 HU 사용 (cell_top + pad_top = 512.37+3.78 = 516.15 px). 산식 자체 정상. |
| 1. 셀 baseline 산식 차이 | **확정 (유력)** | 아래 §3 |

## 3. Baseline 산식 분석

### 현재 SVG 산식 (`paragraph_layout.rs:835`)

```rust
let (line_height, baseline) = (line_height, ensure_min_baseline(
    hwpunit_to_px(comp_line.baseline_distance, self.dpi), max_fs));
// → text 의 baseline = text_y + baseline
//   = cell_top + pad_top + bl
//   = 512.37 + 3.78 + (978 HU = 13.04 px)
//   = 529.19 px ✓ (실제 SVG 출력과 일치)
```

여기서:
- `comp_line.baseline_distance = 978 HU = 13.04 px` (LINE_SEG.bl 그대로)
- `comp_line.line_height = 1150 HU = 15.33 px`

### PDF 추정 산식

PDF baseline ≈ 530.0~530.5 px → `cell_top + pad_top + bl_pdf` 에서 `bl_pdf ≈ 1.0~1.4 px` 만큼 더 큼.

차이 ~1.0–1.4 px (75–105 HU). 가능한 PDF 산식 후보:

| 후보 | 산식 | 결과 (px) | 일치도 |
|------|------|-----------|--------|
| A | `bl` 그대로 | 529.19 | 현재 (오차 1.0~) |
| B | `lh` (= 1150 HU) | 531.48 | 약간 초과 |
| C | `bl + ls/2` (978 + 230) | 532.26 | 초과 |
| D | `lh - (lh-bl)/2` ≈ `(bl+lh)/2` (1064) | 530.34 | **거의 일치** |
| E | 폰트 어센트 기반 (max_fs * 0.85 = 13.03) | 529.19 | 동일 |

후보 D가 가장 근사. 즉 한컴 PDF 는 `(bl + lh) / 2` 또는 비슷한 "lh 와 bl 의 중간값" 으로 baseline 을 잡는 듯.

> 다만 1 px 차이는 폰트 substitute (HWP=한컴윤고딕 vs SVG=HY신명조→Batang) 의 폰트 어센트 차이로도 설명 가능. 표 셀이 아닌 본문에서 동일 ㉠ 도 같은 ~1px 차이가 있는지 추가 검증 필요.

## 4. 영향 범위 / 리스크

- **변위 크기**: 1.0–1.5 px @ 96 dpi (실제 종이상 ~0.4 mm).
- **적용 범위**: 모든 표 셀 텍스트 baseline (회귀 위험 큼).
- **유사 이슈 가능성**: PR #551/Task #544 v2/v3 에서 paragraph border / inset 산식을 정정한 직후 — 그 영향이 잠재적으로 셀 baseline 에도 미쳤을 수 있음 (확인 필요).

## 5. 권고

다음 두 가지 중 하나로 진행을 권고합니다.

### 옵션 A — 최소 변경: 폰트 메트릭 정합성 점검만

- 한컴윤고딕 의 ascent/descent 와 SVG fallback (Batang/HY신명조) 의 차이로 ~1px 변위가 나는 것이라면 코드 수정 없이 폰트 폴백 매핑만 점검.
- ㉠/㉡ 의 시각적 도드라짐은 글리프 자체의 메트릭 차이이므로 추가 수정 없음.
- **장점**: 회귀 0, **단점**: 이슈가 "수용 가능 변위" 로 종결 (실제 1px 미만 ~ 1.5px 변위는 일반적 폰트 fallback 오차 범위).

### 옵션 B — 셀 baseline 산식 정정 (후보 D)

- `paragraph_layout.rs:835` 셀 cas 에서 `bl` 대신 `(bl + lh) / 2` 사용.
- **장점**: PDF 와의 변위 ~0 으로 수렴 가능. **단점**: 모든 표 셀 회귀 영향, 본문 (셀 외) baseline 과의 분기 필요.

## 6. 의견

본 이슈는 사실상 **폰트 fallback 으로 인한 ~1px 시각 변위** 로 보이며, ㉠/㉡ 가 원형 글리프라 더 도드라져 보이는 케이스로 판단합니다. 옵션 A 권고. 단, 작업지시자가 PDF 와의 정확 일치를 원하시면 옵션 B 로 계획서를 갱신해 진행합니다.

## 승인 요청

- (1) 본 분석 결과 수용 여부
- (2) 옵션 A (코드 수정 없이 종결) 또는 옵션 B (산식 정정 진행) 선택

승인 후 절차:
- 옵션 A → 즉시 최종보고서 작성 후 이슈 클로즈 요청
- 옵션 B → 구현 계획서(`task_m100_566_impl.md`) 작성·승인 → Stage 2 시작
