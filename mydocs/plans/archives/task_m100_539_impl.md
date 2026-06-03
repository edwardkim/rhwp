# Task #539 구현 계획서

**제목**: 21_언어_기출.hwp 글박스 안 paragraph 줄간격 좁음 (그룹 B 우선 처리)
**브랜치**: `local/task539`
**이슈**: https://github.com/edwardkim/rhwp/issues/539
**수행계획서**: `mydocs/plans/task_m100_539.md`
**선행 task**: #537 (fix 의 page_path 확장)

---

## 1. 변경 대상 (그룹 B 한정)

| 파일 | 변경 내용 | 예상 LOC |
|------|----------|---------|
| `src/renderer/layout.rs` | page_path 의 vpos correction 또는 lazy_base 산출 보정 (Stage 1 진단 결과로 확정) | +5 ~ +20 |
| `src/renderer/layout/integration_tests.rs` | 그룹 B 케이스 단위 테스트 (페이지 9 pi=182, 페이지 7 pi=146) | +60 (신규) |
| `mydocs/working/task_m100_539_stage{1,2,3}.md` | 단계별 보고서 | 신규 |
| `mydocs/report/task_m100_539_report.md` | 최종 보고서 | 신규 |

**그룹 A 는 본 task 에서 처리하지 않음** — Stage 3 에서 별도 task 분리 결정.

## 2. Stage 1 정밀 진단 계획

Task #537 의 fix 가 그룹 B 케이스에서 발동하지 않는 이유 식별 필요. 가능한 원인:

### 2.1 가설 (Stage 1 에서 검증)

**가설 A**: `vpos_page_base` 가 Some(...) 이라서 page_path 사용. 그러나 `col_anchor_y` 와 `col_area.y` 가 차이 나서 end_y 가 잘못 계산.

**가설 B**: `prev_has_overlay_shape` 또는 다른 가드가 발동하여 vpos correction 자체가 skipped.

**가설 C**: end_y 가 sequential y_offset 보다 작거나 col 영역 밖이라 `applied=false`.

**가설 D**: 페이지 7 pi=146 (르포르) 케이스: pi=145 가 InFrontOfText Shape 를 controls 에 포함 → `prev_has_overlay_shape = true` → vpos correction skipped → drift 잔존.

### 2.2 진단 절차

```bash
# 1. RHWP_VPOS_DEBUG 로 pi=146 / pi=182 출력 확인 (없으면 가드에 의해 스킵)
RHWP_VPOS_DEBUG=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -o /tmp/diag539 -p 6 2>&1 | grep -E "VPOS_CORR.*pi=14[5-9]"

RHWP_VPOS_DEBUG=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -o /tmp/diag539 -p 8 2>&1 | grep -E "VPOS_CORR.*pi=18[12]"

# 2. dump 로 pi=145 / pi=181 의 controls 중 InFrontOfText/BehindText 확인
./target/release/rhwp dump samples/21_언어_기출_편집가능본.hwp -s 0 -p 145
./target/release/rhwp dump samples/21_언어_기출_편집가능본.hwp -s 0 -p 181
```

`pi=145 ci=0 InFrontOfText tac=true` (이미 dump-pages 출력으로 확인됨) → **가설 D 강력 후보**.

## 3. 수정안 (Stage 2, Stage 1 진단 결과 기반)

### 3.1 가설 D 가 맞는 경우 (예상 시나리오)

`layout.rs:1443-1462` 의 `prev_has_overlay_shape` 검사가 `InFrontOfText/BehindText` 또는 `TopAndBottom + Para-relative + !tac` 인 경우 true. pi=145/pi=181 의 InFrontOfText shape 가 이에 해당.

**수정안 A'-1**: `treat_as_char=true` 인 InFrontOfText/BehindText shape 는 `prev_has_overlay_shape` 가드에서 제외.

```rust
let prev_has_overlay_shape = paragraphs.get(prev_pi).map(|p| {
    use crate::model::shape::{TextWrap, VertRelTo};
    p.controls.iter().any(|c| match c {
        Control::Shape(s) => {
            let cm = s.common();
            // [Task #539] tac=true 인 InFrontOfText/BehindText 는 vpos 에 영향 안 줌 → 가드 제외
            if cm.treat_as_char {
                return false;
            }
            matches!(cm.text_wrap, TextWrap::InFrontOfText | TextWrap::BehindText)
                || (matches!(cm.text_wrap, TextWrap::TopAndBottom)
                    && matches!(cm.vert_rel_to, VertRelTo::Para)
                    && !cm.treat_as_char)
        }
        Control::Picture(pic) => { /* 동일 로직 */ }
        _ => false,
    })
}).unwrap_or(false);
```

근거: `treat_as_char=true` 의 Shape 는 paragraph 의 line_seg vpos 에 통합되어 처리됨. vpos 누적은 정상이므로 overlay 에 의한 과대 누적이 발생하지 않음.

### 3.2 가설 A 또는 C 가 맞는 경우

추가 분석 후 별도 안 (`A'-2`, `A'-3`) 도출. Stage 1 진단 결과로 확정.

## 4. 단계 분할

### Stage 1 — 정밀 진단 + TDD
1. RHWP_VPOS_DEBUG / dump 출력 수집
2. 가설 검증 (D 우선)
3. 그룹 B 두 케이스 (페이지 7 pi=146, 페이지 9 pi=182) 의 SVG y / IR vpos 비교 표 작성
4. TDD 통합 테스트 추가 (`test_539_partial_paragraph_after_overlay_shape` 등)
5. **현재 상태 실패** 확인
6. Stage 1 보고서 + 커밋

### Stage 2 — Fix 적용
1. 진단 결과에 따라 layout.rs 수정 (예상: A'-1)
2. TDD 테스트 통과 확인
3. Task #537 의 11 곳 정량 재확인 (회귀 없음)
4. 그룹 B 두 케이스 정량 검증 (gap = IR delta 24.21 px)
5. Stage 2 보고서 + 커밋

### Stage 3 — 회귀 검증 + 그룹 A 결정
1. `cargo test` 전체 통과
2. 광범위 샘플 회귀 검증 (Task #537 의 7 샘플 동일)
3. 그룹 A 의 한컴 PDF 비교 측정 결과 정리
4. 그룹 A 별도 task 분리 결정 (작업지시자 승인 요청)
5. 최종 보고서 + 커밋

## 5. 검증 명령

```bash
# 빌드
cargo build --release

# 본 task 대상 페이지 SVG
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag539 -p 6
./target/release/rhwp export-svg samples/21_언어_기출_편집가능본.hwp -o /tmp/diag539 -p 8

# vpos 디버그
RHWP_VPOS_DEBUG=1 ./target/release/rhwp export-svg \
  samples/21_언어_기출_편집가능본.hwp -o /tmp/diag539 2>&1 | grep VPOS_CORR

# 자동 테스트
cargo test --release --lib

# 회귀 광범위 검증
./target/release/rhwp export-svg samples/synam-001.hwp -o /tmp/diag539/synam
./target/release/rhwp export-svg samples/복학원서.hwp -o /tmp/diag539/bokhak
./target/release/rhwp export-svg samples/exam_math.hwp -o /tmp/diag539/exam_math
```

## 6. 위험 및 완화

| 위험 | 완화 |
|------|------|
| `prev_has_overlay_shape` 가드 완화로 다른 케이스 회귀 (BehindText 가 실제 vpos 영향) | `treat_as_char=true` 로 한정. 광범위 샘플 검증. |
| Stage 1 진단 결과가 가설 D 와 다르면 수정안 재설계 필요 | Stage 1 종료 시 수정안 확정 후 Stage 2 진행 (TDD 통과 가능 시점 확인) |
| 그룹 A 본 task 미처리로 작업지시자 보고 미해결 | Stage 3 에서 그룹 A 별도 task 분리 요청. 한컴 환경 검증 시점 통합 처리 |

## 7. 커밋 단위

- Stage 1: "Task #539 Stage 1: 그룹 B 정밀 진단 + TDD 단위테스트"
- Stage 2: "Task #539 Stage 2: prev_has_overlay_shape 가드 완화 (treat_as_char 제외)"
- Stage 3: "Task #539 Stage 3: 광범위 회귀 검증 + 최종 보고서"

`closes #539` 는 Stage 3 마지막 또는 merge 커밋.

---

**작업지시자 승인 요청 사항**:

1. **A'-1 수정안** (treat_as_char Shape 의 overlay 가드 제외) 채택 — Stage 1 가설 D 확정 후
2. **단계 분할 3단계** 적정성
3. **그룹 A 처리 시점** — Stage 3 에서 별도 task 분리 vs 본 task 통합
4. **TDD 테스트 케이스** — 두 그룹 B 위치 (페이지 7 pi=146 + 페이지 9 pi=182) 모두 포함

승인 후 Stage 1 부터 시작합니다.
