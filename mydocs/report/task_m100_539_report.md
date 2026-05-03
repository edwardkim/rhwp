# Task #539 최종 보고서

**제목**: 21_언어_기출.hwp 글박스 안 paragraph 줄간격 좁음 (그룹 B 처리)
**마일스톤**: M100 (v1.0.0)
**브랜치**: `local/task539`
**이슈**: https://github.com/edwardkim/rhwp/issues/539
**선행 task**: #537 (PR #538)

---

## 1. 문제 요약

`samples/21_언어_기출_편집가능본.hwp` 의 두 그룹 결함 보고:
- 그룹 A: 지문 시작 표시 [X~Y] → 다음 문단 사이 줄간격 (9곳)
- 그룹 B: 글박스(InFrontOfText Shape) 호스트 paragraph 직후 다음 paragraph 줄간격 (2곳)

본 task 는 **그룹 B 우선 처리**.

## 2. 그룹 B 정량 (수정 전)

| 위치 | gap (px) | IR 기대 (px) | drift |
|------|---------|-------------|-------|
| 7p pi=145 → pi=146 "르포르는..." | 14.67 | 24.21 | -9.55 (1 ls 부족) |
| 9p pi=181 → pi=182 "더불어 수피즘..." | 14.67 | 24.21 | -9.55 (1 ls 부족) |

## 3. 근본 원인

`src/renderer/layout.rs:1443-1462` 의 `prev_has_overlay_shape` 가드:
- `Control::Shape` 분기에서 `treat_as_char` 무관하게 `InFrontOfText/BehindText` 면 true 반환
- pi=145 controls=[Shape ci=0 wrap=InFrontOfText **tac=true**] (글박스)
- pi=181 controls=[Shape ci=0 wrap=InFrontOfText **tac=true**] (글박스)

→ `prev_has_overlay_shape = true` → 직후 paragraph 의 vpos correction 분기 자체가 skipped (RHWP_VPOS_DEBUG 에 entry 누락) → trailing-ls drift 716 HU 잔존.

## 4. 수정 내용

### 4.1 코드 변경 (`src/renderer/layout.rs:1443-1471`, +9 LOC)

```rust
Control::Shape(s) => {
    let cm = s.common();
    // [Task #539] tac=true Shape 는 paragraph 의 LINE_SEG vpos 에
    // 통합되어 누적되므로, overlay 가 vpos 에 별도 영향을 주지 않는다.
    if cm.treat_as_char {
        return false;
    }
    matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
        || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
            && matches!(cm.vert_rel_to, VertRelTo::Para)
            && !cm.treat_as_char)
}
```

### 4.2 단위 테스트

`integration_tests.rs` 에 두 통합 테스트 추가:
- `test_539_paragraph_after_overlay_shape_host` (7p 르포르)
- `test_539_partial_paragraph_after_overlay_shape` (9p 수피즘)

TDD: Red(Stage 1) → **Green**(Stage 2).

## 5. 검증 결과

### 5.1 자동 테스트
```
test result: ok. 1119 passed; 0 failed; 1 ignored
```
1117 (Task #537 까지) + 2 (Task #539 신규) = 1119. 회귀 0건.

### 5.2 그룹 B 정량 (수정 전/후)

| 위치 | 수정 전 (px) | 수정 후 (px) | IR (px) |
|------|-------------|-------------|---------|
| 7p pi=146 '르' baseline | 569.81 | **579.36** | 579.36 |
| 9p pi=182 '더' baseline | 333.75 | **343.29** | 343.30 |
| gap pi=145→146 | 14.67 | **24.21** ✓ | 24.21 |
| gap pi=181→182 | 14.67 | **24.21** ✓ | 24.21 |

### 5.3 광범위 회귀 검증 (8 샘플)

| 샘플 | 변경 페이지 | 시프트 |
|------|-----------|-------|
| `21_언어_기출_편집가능본.hwp` | 10/15 | 직접 대상 fix + 동일 본질 추가 fix |
| `synam-001.hwp` | 0/35 | 변경 없음 |
| `복학원서.hwp` | 0/1 | 변경 없음 |
| `exam_math.hwp` | 0/20 | 변경 없음 |
| `exam_kor.hwp` | 0/20 | 변경 없음 |
| `exam_eng.hwp` | 1/8 | 151 paragraph +7.68 px (정합성 개선) |
| `exam_science.hwp` | 0/6 | 변경 없음 |
| `2010-01-06.hwp` | 0/6 | 변경 없음 |

**모든 샘플 음의 시프트 0건** = 회귀 없음. 변경은 모두 IR 정합 방향.

### 5.4 Task #537 회귀 없음
페이지 2 q3 ①→② gap = 72.64 px (IR 정확) 유지. test_537 PASS.

### 5.5 VPOS_CORR debug 변화

수정 전:
```
                                    ← pi=146 / pi=182 entry 누락
VPOS_CORR pi=147 base=716 ...
```

수정 후:
```
VPOS_CORR pi=146 base=0 applied=true   ← 신규 entry, drift 없음
VPOS_CORR pi=147 base=0 applied=true   ← Task #537 fix 도 함께 효과
VPOS_CORR pi=182 base=0 applied=true   ← 신규 entry
```

## 6. 메모리 룰 적용

- **본질 정정 회귀 위험**: 광범위 검증에서 모든 회귀 샘플 0건 — 매우 국소적 수정.
- **PDF 비교 결과는 절대 기준이 아님**: 그룹 B 는 IR vpos 자체가 명확히 1 ls 부족이었음 → IR 정합 fix.
- **룰과 휴리스틱 구분**: prev_has_overlay_shape 가드의 본래 의도(InFrontOfText/BehindText 별도 vpos 누적 방지) 는 `tac=true` 와 무관 → 단일 룰로 정정.

## 7. 잔존 / 후속 사항

### 7.1 그룹 A — 별도 issue 분리 권고

페이지 2 [4~6] → 지문 gap = 33.01 px = IR 정확. rhwp 는 IR 따름.
한컴 PDF 와 차이 시 의심 본질:
1. 한컴 60% line spacing 명세 동작 차이
2. 빈 paragraph (cc=0) + 음수 ls 처리

→ 한컴 2010/2020 환경 검증 후 별도 task 등록.

### 7.2 base=716 잔존 케이스 (Task #537 잔존)
본 task 의 fix 로 일부 해소됐을 가능성 있음. 별도 검증 후 처리.

## 8. 산출물 일람

| 파일 | 종류 | 변경 |
|------|-----|------|
| `src/renderer/layout.rs` | 코드 | +9 / -0 (prev_has_overlay_shape 가드 완화) |
| `src/renderer/layout/integration_tests.rs` | 테스트 | +155 (TDD 통합 테스트 2건) |
| `mydocs/plans/task_m100_539.md` | 수행계획서 | 신규 |
| `mydocs/plans/task_m100_539_impl.md` | 구현계획서 | 신규 |
| `mydocs/working/task_m100_539_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_539_report.md` | 본 최종 보고서 | 신규 |

## 9. 커밋

| 커밋 | Stage | 내용 |
|------|-------|------|
| (Stage 1) | Stage 1 | TDD 테스트 + 수행/구현 계획서 |
| `0db709bb` | Stage 2 | layout.rs prev_has_overlay_shape 가드 완화 |
| (Stage 3) | Stage 3 | 광범위 회귀 검증 + 최종 보고서 |

## 10. 작업지시자 결정 요청

1. **그룹 B 직접 대상 (검증 완료)**: merge 진행
2. **그룹 A 별도 issue 등록**: 한컴 환경 검증 후 진행 권고
3. **시각 비교**: 한컴 PDF 와의 시각 비교는 작업지시자 검토에 위임
