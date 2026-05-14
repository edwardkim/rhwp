# Task #894 Stage 1 진단 보고서 — HWPX 변환본 페이지 수 정합 (72 → 62)

**Stage**: 1 / 3 (항목 C')
**상태**: 1차 진단 완료 — 작업 방향 결정 요청

## 1. 진단 결과 요약

### 1.1 현재 페이지 수

| 파일 | rhwp | 한컴 viewer | 차이 |
|------|------|-----------|------|
| `hwp3-sample16.hwp` (원본 HWP3) | 64 | 64 | 0 ✅ |
| `hwp3-sample16-hwp5.hwp` (HWP5 변환본) | 62 | 62 | 0 ✅ |
| `hwp3-sample16-hwp5.hwpx` (HWPX 변환본) | **72** | **62** | **+10 ❌** |

### 1.2 핵심 관찰

- Paragraph count 동일 (HWPX 1058 = HWP5 1058)
- 페이지 1 비교:
  - HWPX: items=28, used=962.8px
  - HWP5: items=27, used=911.6px
  - **diff=+51.2px** (HWPX 가 첫 페이지에 paragraph 1개 더 들어감)
- `dump-pages` 의 hwp_used 진단: HWPX 페이지 1 표시 = `used=962.8px hwp_used≈943.6px diff=+19.2px`

### 1.3 ir-diff 차이 패턴 (HWPX vs HWP5)

| 항목 | 건수 | 패턴 |
|------|------|------|
| char_shapes count | 604 | 빈 paragraph: HWPX=0, HWP5=1 (HWPX 가 default char_shape 미생성) |
| line_segs count | 59 | PUA U+F03C5 글머리 paragraph: HWPX=1, HWP5=0 |
| cc (char count) | 26 | paragraph 길이 |
| text | 13 | paragraph 텍스트 |

### 1.4 page-level 누적 차이

- diff +19.2px/page × 60 page ≈ **+1152px ≈ 약 10 페이지 inflate** — 실측 (62 → 72) 와 일치
- 즉 HWPX 의 page 당 정확히 균일하게 19.2px 더 사용. 각 paragraph 의 height 자체는 동일하지만 **누적 위치 (vpos) 또는 paragraph 간 spacing** 이 다를 가능성 높음

## 2. 추가 측정 (다른 HWPX 샘플)

다른 HWPX 샘플의 페이지 수 (한컴 viewer 정답지 미비 — 비교 보조용):

| 샘플 | rhwp 페이지 |
|------|-----------|
| `표-텍스트.hwpx` | 1 |
| `2025년 기부·답례품…` | 30 |
| `hwp3-sample-hwpx.hwpx` | 15 |
| `hwp3-sample10-hwpx.hwpx` | 767 |
| `hwp3-sample13-hwp5.hwpx` | 3 |
| `hwp3-sample14-hwp5.hwpx` | 11 |
| **`hwp3-sample16-hwp5.hwpx`** | **72 (한컴 62)** |
| `hwp3-sample5-hwpx.hwpx` | 64 |
| `table-vpos-01.hwpx` | 5 |
| `tac-img-02.hwpx` | 70 |

→ 다른 HWPX 샘플들은 한컴 viewer 정답지 없어 정합 점검 어려움. **sample16-hwp5.hwpx 만 대조 가능**.

## 3. 가설 후보 vs 검증 비용

| 가설 | 검증 비용 | 비고 |
|------|----------|------|
| H1: HWPX 빈 paragraph 의 char_shape 미생성 → line height 영향 | 중 | `section.rs:144` char_shape_changes 가 비어있을 때 default 추가 시도 |
| H2: HWPX paragraph 의 spacing_before/after 단위 변환 차이 | 중 | ParaShape spacing 값 비교 |
| H3: HWPX char metric (폰트 크기 / line height) 계산 차이 | 높 | layout 코드 trace 필요 |
| H4: HWPX의 페이지 break 누적 계산 알고리즘 차이 | 높 | pagination 영역 |
| **H5: 한컴 HWPX 변환기 자체가 line height 다르게 설정** | — | rhwp 의 문제가 아닐 가능성. 한컴 viewer 가 HWPX 정답지 아님 |

## 4. 위험 평가

- HWPX 파서 / layout 변경은 **모든 HWPX 샘플에 영향**
- 그런데 sample16-hwp5.hwpx 외에는 한컴 viewer 정답지가 없어 회귀 점검 어려움
- 잘못된 fix 는 다른 HWPX 샘플의 페이지 수 회귀 야기 가능
- ★ **H5 가설**: 한컴이 HWP3 → HWPX 로 변환할 때 의도적으로 다른 metric 사용 가능성. 이 경우 rhwp 는 한컴 viewer 정합이 아닌 HWPX spec 정합을 따라야 함

## 5. 작업 방향 옵션

| 옵션 | 처리 | 비고 |
|------|------|------|
| (a) | H1 (빈 paragraph default char_shape) 만 fix 시도 → 효과 측정 | 작은 변경, 회귀 위험 낮음. 효과는 미지수 |
| (b) | 깊이 진단 (layout / pagination 코드 trace) 후 root cause fix | 시간 많이 소요. 회귀 위험 높음. sample16 만 검증 가능 |
| (c) | Stage 1 별도 task 로 분리 → #894 는 B + A + D 만 진행 | 본 Stage 의 분량이 단일 task 수준. HWPX 회귀 점검 자료 부족 우려 |
| (d) | H5 가설 검증 우선 — HWPX spec 의 metric 정의 확인 후 결정 | 한컴 viewer 정합이 정답인지부터 확정 |

## 6. 작업지시자 결정 요청

본 1차 진단으로는 정확한 root cause 가 안 잡힘. 작업 방향 결정 필요.

**기본 추천**: 옵션 **(c) Stage 1 별도 task 분리**.
- Stage 2 / 3 / D (CLAUDE.md) 는 sample16 의 시각 정합에 더 직접적인 영향
- Stage 1 은 HWPX 변환본의 페이지 수 inflate — 분량 큼, 회귀 점검 자료 부족, 별도 깊은 작업 필요
- 별도 task 분리하여 HWPX 전반의 정합성 점검을 종합적으로 다룸

## 7. 산출물

- 진단 결과: 본 문서
- ir-diff 원본: `/tmp/ir_diff_hwpx_vs_hwp5.txt` (2076 lines, 753 건 차이)
