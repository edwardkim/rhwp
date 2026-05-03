# Task #540 Stage 0 — 사전 분석 보고서

**제목**: 한컴 환경 검증 입력 대기를 위한 사전 분석
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/regex/issues/540

---

## 1. 9곳 IR 구조 통일성 (확정)

모든 9곳이 동일 구조: [X~Y] paragraph (165% line, ls=716) → 빈/공백 paragraph (음수 ls) → 지문 첫 paragraph.

| 페이지 | [X~Y] pi | 빈 pi | 빈 paragraph PS line | 빈 ls | 다음 지문 pi |
|--------|---------|-------|---------------------|-------|-------------|
| 2 | 44 | 45 | 60% | **−440** | 46 |
| 4 | 80 | 81 | 95% | **−56** | 82 |
| 5 | 110 | 111 | 65% | **−384** | 112 |
| 7 | 143 | 144 | 95% | **−56** | 145 |
| 8 | 174 | 175 | 95% | **−56** | 176 |
| 10 | 206 | 207 | 60% | **−440** | 208 |
| 11 | 232 | 233 | 95% | **−56** | 234 |
| 13 | 264 | 265 | 60% | **−440** | 266 |
| 14 | 294 | 295 | 95% | **−56** | 296 |

## 2. rhwp 현재 출력 (수정 후) 정량

| 페이지 | gap (px) | 빈 paragraph ls | 추정 advance |
|--------|---------|----------------|---------------|
| 2p | 33.01 | -440 (60%) | 1816 + 660 = 2476 HU |
| 5p | 33.76 | -384 (65%) | 1816 + 716 = 2532 HU |
| 7p | 38.13 | -56 (95%) | 1816 + 1044 = 2860 HU |
| 13p | 33.01 | -440 (60%) | 1816 + 660 = 2476 HU |

다른 페이지(4/8/10/11/14)는 측정 기준 line 식별 정확도 낮아 추정 (gap 38.13 등은 95% 95% advance 에 부합).

## 3. 코드 분석

### 3.1 paragraph_layout.rs (Percent line spacing 처리)

`src/renderer/composer/line_breaking.rs:818` 의 Percent advance 계산:
```rust
LineSpacingType::Percent => {
    // 전체 줄 피치 = line_height * percent / 100
    // line_spacing = 전체 줄 피치 - line_height
    (line_height_hwp as f64 * (ls_value - 100.0) / 100.0).max(0.0) as i32
}
```

→ **percent < 100% 인 경우 ls = 0 으로 floor 적용**.

### 3.2 그러나 SVG 출력은 IR vpos 따름

paragraph_layout.rs 의 line advance:
```rust
let line_spacing_px = hwpunit_to_px(comp_line.line_spacing, self.dpi);
y += line_height + line_spacing_px;
```

여기 `comp_line.line_spacing` 은 composer 계산값(0 으로 floor 됨). 그러나 vpos correction (Task #537 / #539) 이 IR 의 실제 vpos 위치로 paragraph 를 강제 이동.

→ 결과적으로 SVG 출력은 IR vpos 의 음수 ls 영향 그대로 반영 (33.01 px = 2476 HU).

### 3.3 가설 (한컴 동작)

가설 H1: 한컴은 Percent ls 를 ls = `lh × percent / 100` 로 해석 (advance, not "extra").
- 60% 면 advance = 660 HU
- 95% 면 advance = 1045 HU (≈ 1044)
- rhwp 와 동일 동작

가설 H2: 한컴은 Percent < 100% 의 빈 paragraph 의 visual height 를 lh 그대로 사용 (음수 ls 무시).
- 60% 빈 paragraph advance = 1100 HU
- 95% 빈 paragraph advance = 1100 HU
- 페이지 2 [4~6] gap = 1816 + 1100 = 2916 HU = **38.88 px**

가설 H3: 한컴은 paragraph 사이 minimum spacing 적용.
- 한컴 명세 자문 필요

## 4. 광범위 영향 평가

| 샘플 | 음수 ls 횟수 | 영향 평가 |
|------|------------|----------|
| `synam-001.hwp` | **57** | 가장 큼. 음수 ls 처리 변경 시 회귀 위험 매우 큼 |
| `21_언어_기출` | 15 | 직접 대상 |
| `exam_math.hwp` | 13 | 중간 |
| `exam_science.hwp` | 2 | 작음 |
| `exam_eng.hwp` | 1 | 미미 |
| `exam_kor.hwp` | 0 | 영향 없음 |

## 5. 작업지시자 입력 요청 (Stage 1 진입 조건)

### 5.1 한컴 환경 측정값
한컴 2010 / 한컴 2020 / 한컴독스 PDF 200dpi 에서 다음 측정:

1. **페이지 2 [4~6] → 지문 첫 줄 gap (px)**: 한컴이 33.01 px 인지 38.88 px 인지 또는 다른 값인지
2. (선택) 동일 측정 4p, 5p, 13p — line spacing % 별 동작 차이 확인
3. (선택) `synam-001.hwp` 의 음수 ls paragraph 사례 1~2 곳 — 한컴이 다르게 처리하는지

### 5.2 한컴 명세 자문
가능 시 다음 질문에 대한 답:
- HWP 파일 포맷의 LINE_SEG.line_spacing 은 "advance" 인지 "extra" 인지?
- Percent line spacing 의 한컴 정확 해석?
- 음수 ls 의 한컴 동작 (floor vs raw)?

### 5.3 fix 결정 기준
- 한컴 측정값이 33.01 (= rhwp 현재) → 본 task 종료, 작업지시자 보고가 잘못된 인식
- 한컴 측정값이 38.88 (= 가설 H2) → 빈 paragraph 음수 ls floor fix
- 다른 값 → 추가 분석 후 결정

## 6. 다음 단계

작업지시자 한컴 측정값 입력 → Stage 1 진입:
1. 측정값 기반 가설 (H1/H2/H3) 확정
2. fix 안 (A/B/C) 결정
3. TDD 테스트 작성 (현재 실패 확인)
4. Stage 2 fix 적용
5. Stage 3 광범위 회귀 검증

본 task 진행 정지 — 한컴 입력 대기.

## 7. 산출물 (Stage 0)

| 파일 | 변경 |
|------|------|
| `mydocs/plans/task_m100_540.md` | 수행계획서 |
| `mydocs/working/task_m100_540_stage0.md` | 본 보고서 |

코드 변경 없음 — 사전 분석만.
