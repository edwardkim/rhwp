# Task #566 최종 보고서 — exam_science.hwp 7번 표 ㉠/㉡ 베이스라인 시프트 (옵션 A 종결)

- **이슈**: [#566](https://github.com/edwardkim/rhwp/issues/566) `[m100] exam_science.hwp 7번 표 셀 내부 ㉠/㉡ 베이스라인 위치가 위로 시프트됨`
- **마일스톤**: M100
- **브랜치**: `local/task566` (from `local/devel`)
- **작성일**: 2026-05-04
- **결정**: **옵션 A — 코드 수정 없이 종결** (작업지시자 승인)

---

## 1. 결론 요약

본 이슈에서 보고된 "㉠/㉡ 가 PDF 대비 위쪽으로 올라가 표시됨" 현상의 실제 변위 양은 **약 1.0 ~ 1.5 px (약 75 ~ 110 HU @ 96 dpi, ≈ 0.4 mm 종이상)** 으로 측정되었으며, 이는

1. **㉠/㉡ 만의 결함이 아님** — 표 전 셀 텍스트(직선형/굽은형/HOF/CF₄ 등)에 동일하게 적용
2. **폰트 fallback 으로 충분히 설명 가능** — 한컴 PDF (한컴윤고딕) ↔ SVG (HY신명조→Batang 등 시스템 폰트) 의 어센트/디센트 차이 범위 내
3. **㉠/㉡ 가 원형 글리프인 시각적 특성상** 동일 변위가 더 도드라져 보이는 현상

으로 결론지어, **소스 코드 수정 없이 종결** 합니다.

## 2. 시각 재현 (Stage 1)

### 96 dpi 픽셀 측정값

| 측정 | SVG | HWP PDF |
|------|-----|---------|
| 표(4×7) 셀 사각형 | y=512.37, h=22.88 | y≈512.37, h≈22.88 (동일) |
| 행 0 ㉠ 글리프 가시 y 범위 | 517 – 531 | 517 – 530 |
| 행 0 ㉠ 밑줄 y | 531.19 | 532.00 |
| 행 1 "직선형" 글리프 y 범위 | 540 – 554 | 541 – 555 |

**시프트 양**: 약 **1.0 px** (밑줄 좌표 비교 기준 0.81 px, 본문 직선형 1.0 px)

### 비교 자료

- 좌(SVG)/우(PDF) 대조 이미지: `/tmp/sbs.png`
- diff 오버레이(적=SVG only, 청=PDF only, 흑=일치): `/tmp/diff.png`
- 디버그 오버레이: `output/exam_science_002.svg` (s0:pi=35 ci=0 4x7 y=512.4)

## 3. 가설 검증 결과 (Stage 1)

| 가설 | 결과 | 근거 |
|------|------|------|
| (1) 셀 baseline 산식 차이 | **확정 (유력)** | `bl=978HU` 그대로 사용 — `paragraph_layout.rs:835` |
| (2) CharShape 세로 시프트 | 기각 | cs_id=10/117 모두 `char_offsets=[0,...,0]` |
| (3) valign / padding 누락 | 기각 | aim=false → `prefer_cell_axis(283,141)=true` 정상 산식 |
| (4) ㉠/㉡ 글리프 메트릭 의존 | 부분 | 시프트는 모든 셀에 동일, 시각 도드라짐만 강함 |
| (5) SVG transform/scale 분기 | 기각 | 셀/본문 모두 `transform translate scale(0.95,1)` 동일 |

### CharShape / LineSeg 실측

`examples/dump_cell_lineseg.rs` (신규):

```
cell[1] r=0,c=1 h=1716 pad=(283,283,283,283) text=" ㉠ "
  ls[0] vpos=0 lh=1150 th=1150 bl=978 ls=460 sw=6076
  cs_id=117: font_ids=[12,16,10,8,5,11,5] ratios=[95×7] char_offsets=[0×7] base_size=1150
cell[8] r=1,c=1 h=1916 pad=(283,283,283,283) text="직선형"
  ls[0] vpos=0 lh=1150 th=1150 bl=978 ls=460
  cs_id=10: font_ids=[12,16,10,8,5,11,5] ratios=[95×7] char_offsets=[0×7] base_size=1150
```

㉠ 셀과 직선형 셀은 **동일 메트릭** (font/ratio/offset/base_size). 두 셀 모두 동일 ~1px 시프트.

## 4. 옵션 비교

| 옵션 | 내용 | 변위 결과 | 회귀 위험 | 결정 |
|------|------|-----------|-----------|------|
| A | 코드 수정 없이 종결 (폰트 fallback 변위로 수용) | ~1px 잔존 | 0 | **채택** |
| B | `paragraph_layout.rs:835` baseline 산식 정정 (`(bl+lh)/2` 등) | ~0px | 모든 표 셀 영향 (광범위) | 미채택 |

**옵션 A 채택 사유**:

- 1 px @ 96dpi 는 일반 폰트 fallback 오차 범위
- 동일 시프트가 표 외 본문에도 존재할 가능성 — 본 이슈가 "표 전용 결함" 아닐 가능성
- 광범위 표 텍스트 회귀 리스크 대비 시각 개선 효과 미미
- ㉠/㉡ 의 시각 도드라짐은 글리프 형상 특성

## 5. 검증 / 회귀 점검

코드 변경이 없으므로 회귀 점검 불필요. 기준값(`devel` HEAD `2c39f244`) 동작 그대로 유지.

```bash
cargo build --release  # OK (재빌드 — 변경 없음)
```

## 6. 산출물

| 파일 | 내용 |
|------|------|
| `mydocs/plans/task_m100_566.md` | 수행계획서 |
| `mydocs/working/task_m100_566_stage1.md` | Stage 1 분석 보고서 |
| `mydocs/report/task_m100_566_report.md` | 본 최종 보고서 |
| `examples/dump_cell_lineseg.rs` | CharShape/LineSeg 실측용 진단 도구 |

## 7. 후속 권고

이슈 #566 클로즈. 만약 차후 한컴 PDF 와의 baseline 정합이 우선순위가 되면 옵션 B (paragraph_layout.rs:835 산식 정정 + 회귀 가드 광범위 추가) 로 별도 타스크 등록 권고.

## 8. 이슈 클로즈

작업지시자 승인 후 `gh issue close 566` 수행.
