# Task #547 Stage 0 완료 보고서

**제목**: 사전 분석 + paragraph border 텍스트 inset 산식 진단 + 한컴 환경 검증 입력 대기
**브랜치**: `local/task547`
**이슈**: https://github.com/edwardkim/rhwp/issues/547

---

## 1. 본질 진단

### 1.1 [13~15] passage 박스 구조

`[13~15]-다음 글을 읽고 물음에 답하시오.` 박스는 paragraph border merge group 으로
2개 이상 문단 결합:

| pi | text | ps_id | margin_left (HU) | border_fill_id | border_spacing |
|----|------|-------|-----------------|----------------|----------------|
| 143 | "[13~15]-다음 글을 읽고 물음에 답하시오." | 10 | **0** | 1 | 0/0/0/0 |
| 144 | " " (passage 본문 시작) | 25 | **1700** | 7 | 0/0/0/0 |
| 145+ | passage 본문 ... | 25 | 1700 | 7 | 0/0/0/0 |

pi=143 은 margin_left=0 → text inset 문제 없음.
pi=144+ (passage 본문) 은 margin_left=1700 HU → **text inset 부작용**.

### 1.2 현재 SVG 텍스트 x 산출 (Task #544 적용 후)

`paragraph_layout.rs:709-716`:
```rust
let bs_left_px = para_style.map(|s| s.border_spacing[0]).unwrap_or(0.0);
let bs_right_px = para_style.map(|s| s.border_spacing[1]).unwrap_or(0.0);
let (inner_pad_left, inner_pad_right) = if has_visible_stroke && bs_left_px == 0.0 && bs_right_px == 0.0 {
    (box_margin_left, box_margin_right)  // ← 여기서 margin 한번 더 더함
} else {
    (0.0, 0.0)
};
let margin_left = box_margin_left + inner_pad_left;  // = 11.33 + 11.33 = 22.66 px
```

ParaShape margin_left=1700 HU → style_resolver 에서 `/2 = 850 HU = 11.33 px`.

| 좌표 | 산식 | 값 |
|------|------|-----|
| col_area.x | (page margin) | 117.17 px |
| Box outline x | col_area.x (Task #544) | **117.17 px** |
| Text x (현재) | col_area.x + 11.33 + 11.33 | **139.83 px** |
| Box 안 좌측 여백 | 22.66 px | |

### 1.3 PDF (한컴 2010) 기대 좌표

| 좌표 | 측정/기대 | 값 |
|------|----------|-----|
| Box outline x | (Task #544 검증) | 117.0 px |
| Text x | col_area.x + box_margin_left | ≈ 128.5 px |
| Box 안 좌측 여백 | box_margin_left | ≈ 11.33 px |

**차이**: 현재 SVG 가 PDF 보다 텍스트 x 가 **+11.33 px 우측** (= box_margin_left 한 번 더 적용된 양).

## 2. 원인 본질

`inner_pad_left = box_margin_left` 로직은 **Task #544 이전**에 도입되었음. 그 시점:

- 박스 outline x = col_area.x + box_margin_left (margin 적용)
- Text x = col_area.x + 2 * box_margin_left (margin 한 번 더)
- 박스 안 좌측 여백 = box_margin_left (한 번 만큼)

Task #544 후:

- 박스 outline x = col_area.x (margin 미적용)
- Text x = col_area.x + 2 * box_margin_left (변경 없음)
- 박스 안 좌측 여백 = **2 * box_margin_left** ← 두 배 inset (부작용)

→ Task #544 의 fix 가 박스 outline 만 옮기고 inner_pad logic 을 그대로 둔 결과.

## 3. fix 방향

### 3.1 본질 정정안 — inner_pad_left logic 제거

```rust
// Task #547: Task #544 이후 박스 outline 은 col_area, 텍스트 inset 은 box_margin_left
// 한 번만 적용. 기존 inner_pad_left = box_margin_left 분기는 Task #544 전에 박스도
// margin 을 적용했을 때만 의미가 있었음.
let margin_left = box_margin_left;
let margin_right = box_margin_right;
```

→ 텍스트 x = col_area.x + 11.33 = **128.50 px** ≈ PDF 기대값 일치.

### 3.2 회귀 위험

`inner_pad_left` 분기가 적용되는 케이스:
- has_visible_stroke (paragraph border 가진 stroke 있는 문단)
- border_spacing[0]=[1]=0

이 조건 만족하는 케이스가 다른 샘플에 있으면 영향. Stage 1 에서 광범위 사전 평가:

| 샘플 | 영향 가능성 | 검증 |
|------|-------------|------|
| 21_언어_기출 | passage 박스 본문 (pi=144 등) | 핵심 fix |
| exam_kor | paragraph border 가진 본문 케이스 | Stage 1 측정 |
| exam_math | paragraph border 보유 | Stage 1 측정 |
| synam-001 | paragraph border 보유 가능성 | Stage 1 측정 |
| 기타 | margin_left=0 → 영향 없음 | - |

## 4. HWP 환경 비교 (작업지시자 입력 요청)

[feedback_pdf_not_authoritative] 메모리 룰 적용. 한컴 2010 PDF 외:

1. 한컴 2020 (samples/21_언어_기출_편집가능본-2020.pdf) 의 [13~15] 박스 안 좌측 여백 측정값
2. 한컴독스 환경 (가능 시)

위 결과로 PDF 가 정합 ref 인지 재검증.

## 5. 셀 내부 / wrap=Square 호스트 영향

| 케이스 | 영향 | 대응 |
|--------|------|------|
| 셀 내부 paragraph border | inner_pad logic 동일 적용 → 동일 부작용 | fix 와 함께 정정 |
| wrap=Square 호스트 (border_box_override) | box_x override 따로 → inner_pad 영향 동일 | fix 와 함께 정정 |
| paragraph border 없음 | has_visible_stroke=false → 변경 없음 | 영향 없음 |

## 6. 검증 데이터 산출

| 항목 | 현재 | fix 후 기대 |
|------|------|------------|
| pi=144 text x | 139.83 px | 128.50 px |
| Box outline x | 117.17 px | 117.17 px (변경 없음) |
| Box 안 좌측 여백 | 22.66 px | 11.33 px |
| PDF 일치 (±2 px) | -11.33 (불일치) | -0.5 (일치) |

## 7. 산출물

| 파일 | 변경 |
|------|------|
| `mydocs/plans/task_m100_547.md` | 수행 계획서 |
| `mydocs/working/task_m100_547_stage0.md` | 본 보고서 |

## 8. 다음 단계 (Stage 1)

1. TDD 통합 테스트 추가: pi=144 text x = 128.5 ±2 px (RED)
2. 광범위 사전 평가: 6 샘플 paragraph border + border_spacing=0 케이스 분포
3. fix 위치 정밀 진단: paragraph_layout.rs:709-716 변경 범위 확정
4. 셀 내부 / wrap=Square 호스트 케이스 영향 평가

## 9. 승인 요청

Stage 0 완료. 본질 진단:
- **원인**: Task #544 가 박스 outline 만 옮기고 inner_pad logic 그대로 → 텍스트 inset 두 배
- **fix 방향**: `inner_pad_left = box_margin_left` 분기 제거 (margin 한 번만 적용)

Stage 1 (TDD 테스트 + 광범위 사전 평가) 진행 승인 요청.

작업지시자 입력 (선택):
- 한컴 2020 / 한컴독스 환경의 [13~15] 박스 안 좌측 여백 측정값
- 위 입력 없이도 한컴 2010 PDF 정합 기준으로 fix 진행 가능 (메모리 룰 권고만 기록)
