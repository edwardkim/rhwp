# Stage 5 완료보고서 — Task #987

## 단계 목표

쪽 번호를 한컴 정답지대로 **쪽 테두리 하단 경계선 아래쪽** 여백에 배치
(sample16, 본문 내 PageNumberPos pos=5).

## 변경 내용

### 1. `page_border_bottom_y` 헬퍼 추가 (`layout.rs:932`)

쪽 테두리 하단 변 y 산출 (build_page_borders 와 동일 공식).
**body 기준 테두리일 때만 Some** 반환:

- body 기준 테두리 = 본문을 감쌈 → 쪽 번호는 테두리 *바깥(아래)* (sample16)
- paper 기준 테두리 = 종이 전체를 감쌈 → 쪽 번호는 테두리 *안쪽* (aift.hwp)
  → paper 기준은 None 반환, 기존 로직 유지

### 2. `build_page_number` 시그니처 + y 보정 (`layout.rs:1497,1583`)

- `page_border_fill: Option<&PageBorderFill>` 인자 추가 (호출부 `:628`)
- 꼬리말 위치 + body 기준 테두리 + **이 페이지에 테두리 실제 그려질 때**
  (`!hide_border`): `y = border_bottom + gap + font_size`
  - gap = `font_size * 0.6` (시각 판정으로 확정 예정)
- 그 외(테두리 없음/paper 기준/hide_border): 기존 footer 중앙 로직 유지

## 회귀 추적 (2회 수정)

| 회귀 | 원인 | 수정 |
|------|------|------|
| 1차: aift p1 실패 | `page_border_fill` 이 섹션 레벨이라 hide_border 페이지에도 Some | `!hide_border` 조건 추가 (build_page_borders 와 동일 판정) |
| 2차: aift p6/p7 실패 | aift 는 **paper 기준** 테두리 — 쪽 번호가 테두리 안쪽이 정답(Task #634) | `page_border_bottom_y` 를 **body 기준만** Some 으로 한정 |

→ body 기준(sample16)만 보정, paper 기준(aift)은 기존 동작.
   포맷이 아닌 **테두리 기준(paper/body)** 으로 분기 — 논리적 일관.

## 측정 결과 (sample16 page 3 = index 2)

| 항목 | y 좌표 (px) |
|------|-------------|
| 쪽 테두리 하변 이중선 | 1086.1 / 1088.2 |
| **쪽 번호 "- 1 -" baseline** | **1113.1** (테두리 하변 아래) |

쪽 번호가 쪽 테두리 하단 경계선 아래 footer 여백에 배치 — 한컴 동작 방향 일치.

## 검증

| 항목 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo test --lib test_634` (aift 쪽번호) | ✅ 8 passed, 0 failed (회귀 해소) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| sample16 쪽 번호 위치 | ✅ 테두리 하변 아래 (y=1113.1) |
| aift.hwp 쪽 번호 위치 | ✅ 기존 y=1079.16 복원 (paper 기준) |

산출물: `output/poc/task987_stage5/hwp3-sample16_003.svg`

## 설계 정정 (작업지시자 피드백 반영)

초기 구현은 "쪽 테두리 하변 아래 + gap" 으로 baseline 산출 → 쪽 번호가
용지 하단으로 과도하게 내려감 (작업지시자: "너무 아래로 내려갔다",
"한컴은 꼬리말 영역에 출력, 우리는 꼬리말 다음에 출력").

→ 최종 설계: `page_number_baseline_y` 를 **꼬리말 영역(footer_area)
세로 중앙** 으로 정정 (기존 footer_center 와 동일 공식, footer_area 기준).
body 기준 테두리 + 테두리 실제 그려질 때만 적용, 그 외는 기존 footer_center.

## 측정 결과 (현재, sample16 page 3)

| 항목 | y (px) |
|------|--------|
| footer_area (꼬리말 영역) | 1068.21 ~ 1084.69 (h=16.48) |
| 쪽 번호 "- 1 -" SVG y | 1089.79 (footer 중앙 공식 + SVG baseline 변환) |
| aift.hwp 쪽 번호 (paper 기준 대조) | 1079.16 (Task #634 정합 유지) |

## 검증

| 항목 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo test --lib test_634` | ✅ 8 passed (aift 회귀 없음) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |

## 정답지 정밀 측정 (한글 2022 PDF)

`pdf/hwp3-sample16-hwp5-2022.pdf` 3페이지 `pdftotext -bbox` 추출:

- PDF 페이지 595×841pt (A4), 쪽 번호 "- 1 -" bbox:
  yMin=803.68 yMax=814.63 pt
- rhwp 좌표 환산 (×1122.51/841): **yMin=1072.7 ~ yMax=1087.3 px**

| 항목 | y (px) |
|------|--------|
| **정답지 쪽 번호** | **1072.7 ~ 1087.3** |
| rhwp 쪽 테두리 하변 (body 기준) | 1086.1 ~ 1088.2 |
| rhwp 현재 출력 (footer 중앙) | 1089.8 |

**결론**: 한컴 정답지 자체가 쪽 번호를 쪽 테두리 하변(1086~1088)에
**걸치게** 출력한다. rhwp 현재 구현(1089.8)과 정답지(1087.3)는
**2.5px(0.7mm) 차이로 사실상 일치**.

작업지시자 판정: **현재 구현 수용 (정답지 일치)**.

## 최종 검증

| 항목 | 결과 |
|------|------|
| `cargo test` | ✅ 1476 passed, 0 failed |
| `cargo test --lib test_634` | ✅ 8 passed (aift 회귀 없음) |
| `cargo clippy -- -D warnings` | ✅ 0 warnings |
| sample16 쪽 번호 vs 정답지 | ✅ 1089.8 vs 1087.3 (0.7mm, 정답지 일치) |
| aift.hwp 쪽 번호 | ✅ 1079.16 (Task #634 정합 유지) |

Stage 5 완료. 작업지시자 정답지 대조 판정 통과.

## 다음 단계

최종 보고서 작성 (Stage 1~5 통합) + PR #971 stash 복귀.
