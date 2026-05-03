# Task #540 Stage 3 완료 보고서

**제목**: 광범위 회귀 검증 + 가드/lazy_base 정밀화
**브랜치**: `local/task540`
**이슈**: https://github.com/edwardkim/rhwp/issues/540

---

## 1. 검증 절차

`local/task540` Stage 2 (커밋 `8d27ad44`) 와 그 직전 (`7c8509a9`, Stage 1) 의
SVG 출력을 비교하여 회귀 영향 검출.

### 1.1 검증 샘플

| 샘플 | 페이지 수 | 음수 ls 분포 |
|------|----------|--------------|
| `synam-001.hwp` | 35 | 57건 (대부분 일반 paragraph/셀 내부) |
| `21_언어_기출_편집가능본.hwp` | 15 | 9건 빈 paragraph 음수 ls (#540 target) |
| `exam_math.hwp` | 20 | 9건 빈 paragraph 음수 ls (s0/s1) |
| `exam_eng.hwp` | 8 | 0건 |
| `exam_kor.hwp` | 20 | 0건 |
| `exam_science.hwp` | 6 | 1건 (s0.p0 cc=57 controls=7 ls=-1348 — section-setup) |

### 1.2 검증 방법

각 페이지의 SVG 의 `<text translate(x,y)>...</text>` 좌표를 추출하여
페이지/단어 단위로 BEFORE↔AFTER 비교. shift 분포 확인.

## 2. 1차 검증: 회귀 발견

Stage 2 직후 회귀 2건 발견:

### 2.1 exam_math 페이지 7-16: 음수 시프트 (-7.63 ~ -11.47 px)

**원인**: `vpos_neg_ls_floor_total` 누적 후 sequential y_offset 이 IR vpos 보다
앞서나가, 후속 paragraph 의 `lazy_base` 산출 시 `prev_vpos_end - y_delta_hu`
가 음수로 흘러 fallback 경로 (`(prev_vpos_end, false)`) 진입 → vpos correction
미적용 → 시프트 미반영 → 차분이 음수로 보임.

**수정**: `lazy_base` 산출 시 `vpos_neg_ls_floor_total` 만큼 보정.

```rust
let lazy_base = prev_vpos_end - y_delta_hu + vpos_neg_ls_floor_total;
```

근거: lazy_base 는 IR 절대 vpos 좌표 기준이고, y_offset 은 누적 floor 만큼
shifted 된 상태이므로 보정 후 IR 좌표를 정확히 가리킨다.

### 2.2 exam_science 페이지 1: 본문 +17.97 px 잘못 시프트

**원인**: `s0.p0` 가 cc=57 controls=7 (구역나누기/단정의/감추기/머리말/새번호/표) 의
section-setup paragraph. ls=-1348, lh=1350 → advance ≈ 0 HU 의도. 본 paragraph
는 빈 paragraph 가 아니라 페이지 설정용 컨테이너이므로 floor 하면 본문이
1348 HU 만큼 잘못 시프트됨.

**수정**: 가드를 `text.is_empty() && controls.is_empty()` 로 강화.

```rust
if !p.text.is_empty() || !p.controls.is_empty() { return 0; }
```

근거: 진짜 빈 paragraph 는 controls=0 (paragraph terminator 만 존재). controls
가 있으면 구역/머리말/표/도형 등 page-layout 역할을 가지므로 floor 대상이 아님.

## 3. 2차 검증: 모든 회귀 해소

```
=== synam: 0 pages w/ diffs, NEGATIVE: 0 ===
=== 21: 3 pages w/ diffs, NEGATIVE: 0 ===
  21_언어_기출_편집가능본_001.svg: {0.75: 931}
  21_언어_기출_편집가능본_002.svg: {5.87: 1240}
  21_언어_기출_편집가능본_014.svg: {0.75: 906}
=== math: 6 pages w/ diffs, NEGATIVE: 0 ===
  exam_math_006.svg: {3.04: 28}
  exam_math_007.svg: {3.84: 26}
  exam_math_008.svg: {3.84: 46}
  exam_math_011.svg: {3.84: 20}
  exam_math_012.svg: {3.84: 123}
  exam_math_016.svg: {3.84: 106}
=== eng: 0 pages w/ diffs, NEGATIVE: 0 ===
=== kor: 0 pages w/ diffs, NEGATIVE: 0 ===
=== sci: 0 pages w/ diffs, NEGATIVE: 0 ===
```

### 3.1 분석

- **synam-001 (음수 ls 57건)**: 0 페이지 차이 → `text + controls` 가드가 일반
  paragraph 의 음수 ls 를 모두 보존. 본질 정정 회귀 위험 [feedback_essential_fix_regression_risk]
  완전 회피.
- **21_언어_기출 (target)**: 3 페이지 정정. ls=-56 (95%) → +0.75 px, ls=-440
  (60%) → +5.87 px. 수치가 기대값 (`-ls × 96 / 7200`) 정확히 일치.
- **exam_math**: 6 페이지 정정. ls=-228 (84%) → +3.04 px, ls=-288 (79%) → +3.84 px.
- **exam_eng / exam_kor / exam_science**: 0 차이. 음수 ls 가 없거나 (eng/kor)
  controls 가 있는 section-setup paragraph 만 (sci) 존재하여 가드에 의해 보존.

### 3.2 단위 테스트

```
test result: ok. 1120 passed; 0 failed; 1 ignored
test_540_empty_paragraph_negative_ls_floor ... ok (gap 38.88 px ✓)
```

Task #537/#539 trailing-ls 보정, 글박스 vpos 보정 모두 무회귀.

## 4. 산출물

| 파일 | 변경 |
|------|------|
| `src/renderer/layout.rs` | 가드 강화 (`controls.is_empty()`) + lazy_base 보정 (+10 LOC) |
| `mydocs/working/task_m100_540_stage3.md` | 본 보고서 |
| `mydocs/report/task_m100_540_report.md` | 최종 결과 보고서 |

## 5. 위험/완화 회고

| 위험 | 발견 여부 | 완화 |
|------|----------|------|
| synam-001 일반 paragraph 회귀 | 미발견 | text+controls 가드 |
| Task #537/#539 회귀 | 미발견 | 1120 단위 테스트 통과 |
| section-setup paragraph (ls=-1348) 잘못 floor | **발견** | controls.is_empty() 가드 추가 |
| lazy_base fallback 회귀 (음수 시프트) | **발견** | lazy_base 에 누적 보정 가산 |

## 6. 승인 요청

Stage 3 완료. 최종 결과 보고서 작성 + PR/merge 진행 승인 요청.
