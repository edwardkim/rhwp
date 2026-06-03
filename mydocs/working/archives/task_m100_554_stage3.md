# Stage 3 단계별 보고서 — Task #554

> **이슈**: [#554](https://github.com/edwardkim/issues/554)
> **구현계획서**: `mydocs/plans/task_m100_554_impl.md`
> **단계**: Stage 2-2 — HWP5 휴리스틱 + 조건부 -1600 보정 구현
> **상태**: 완료, 작업지시자 승인 대기
> **작성일**: 2026-05-03

---

## 1. 구현 내용

### 1.1 변경 파일

| 파일 | 변경 | 설명 |
|------|------|------|
| `src/parser/mod.rs` | +37 LOC | `apply_hwp3_origin_fixup()` 신규 함수 + 두 진입점(strict/lenient)에서 호출 |

### 1.2 핵심 로직

**HWP3 origin 식별 + 보정 함수** (`mod.rs`):

```rust
fn apply_hwp3_origin_fixup(doc: &mut Document) {
    let total_paragraphs: usize = doc.sections.iter()
        .map(|s| s.paragraphs.len())
        .sum();
    if total_paragraphs <= 50 {
        return;  // 짧은 문서는 비율 왜곡 회피
    }
    let ps_ratio = doc.doc_info.para_shapes.len() as f64 / total_paragraphs as f64;
    let cs_ratio = doc.doc_info.char_shapes.len() as f64 / total_paragraphs as f64;
    if ps_ratio < 0.05 && cs_ratio < 0.15 {
        for section in doc.sections.iter_mut() {
            section.section_def.page_def.margin_bottom =
                section.section_def.page_def.margin_bottom.saturating_sub(1600);
        }
    }
}
```

### 1.3 호출 위치

- `parse_hwp_with_cfb` (strict CFB 경로): `assign_auto_numbers` 직후
- `parse_hwp_with_lenient` (lenient CFB 경로): `assign_auto_numbers` 직후

### 1.4 설계 선택 근거

- **post-process**: paragraph 수가 필요하므로 `parse_page_def` 시점에는 휴리스틱 적용 불가. 모든 파싱 완료 후 fixup
- **`paragraph_count > 50` 가드**: Stage 1 진단에서 매우 짧은 문서 (1~5 para)는 `PS/Para` 비율이 왜곡됨 (예: `draw-group.hwp` PS/Para=21.0). 50개 임계값으로 안전 분리
- **strict + lenient 두 경로 모두 적용**: lenient 경로(FAT 검증 무시)에서도 변환본이 들어올 수 있음

## 2. 검증 결과

### 2.1 HWP5 변환본 (Task #554 핵심 대상)

| 파일 | 한컴 정답 | 변경 전 | 변경 후 | 평가 |
|------|----------|---------|---------|------|
| hwp3-sample-hwp5.hwp | 16 | 17 (+1) | **15** (-1) | ⚠️ over-correct (Stage 1 예상) |
| hwp3-sample4-hwp5.hwp | 36 | 38 (+2) | **36** ✓ | ✅ 정답 |
| hwp3-sample5-hwp5.hwp | 64 | 68 (+4) | **64** ✓ | ✅ 정답 |

### 2.2 HWP5 일반 fixture 회귀 0

Stage 1 baseline 대비:

| 파일 | Baseline | 변경 후 | 평가 |
|------|----------|---------|------|
| exam_kor.hwp | 20 | 20 | ✅ |
| exam_eng.hwp | 8 | 8 | ✅ |
| exam_math.hwp | 20 | 20 | ✅ |
| exam_science.hwp | 4 | 4 | ✅ Task #546 정합 |
| aift.hwp | 77 | 77 | ✅ |
| biz_plan.hwp | 6 | 6 | ✅ |
| 2010-01-06.hwp | 6 | 6 | ✅ |
| 21_언어_기출_편집가능본.hwp | 15 | 15 | ✅ |
| **2022년 국립국어원 업무계획.hwp** | 40 | **40** | ✅ (방안 A에서 -5 회귀였던 케이스, 휴리스틱으로 정확 회피) |

→ **9 fixture 모두 변화 없음**. 휴리스틱이 정확히 작동.

### 2.3 자동 회귀 검증

| 검사 | 결과 |
|------|------|
| `cargo test --lib` | **1113 passed** (회귀 0) |
| `cargo test --test svg_snapshot` | 6/6 passed |
| `cargo test --test issue_546` | passed (Task #546 정합) |
| `cargo test --test issue_418/501` | passed |
| `cargo clippy --lib -- -D warnings` | **0건** |

## 3. 종합 — Task #554 핵심 진전

| 파일 | 한컴 정답 | rhwp 결과 (Stage 2-2 적용 후) |
|------|----------|------------------------------|
| hwp3-sample.hwp (HWP3) | 16 | 16 ✓ |
| hwp3-sample-hwp5.hwp (HWP5) | 16 | **15** ⚠️ |
| hwp3-sample-hwpx.hwpx (HWPX) | 16 | **15** ⚠️ |
| hwp3-sample4.hwp (HWP3) | 36 | 39 ⚠️ (Task #554 범위 밖, 별도 issue) |
| hwp3-sample4-hwp5.hwp (HWP5) | 36 | **36** ✓ |
| hwp3-sample5.hwp (HWP3) | 64 | 64 ✓ |
| hwp3-sample5-hwp5.hwp (HWP5) | 64 | **64** ✓ |
| hwp3-sample5-hwpx.hwpx (HWPX) | 64 | **64** ✓ |

**개선**: 4개 변환본 (sample4-hwp5, sample5-hwp5, sample5-hwpx, sample5-hwpx) 정답 정합. 1개 변환본 (sample-hwp5/-hwpx) -1 over-correct 잔존.

## 4. 잔존 사항

### 4.1 hwp3-sample 변환본 -1 over-correct (의도됨)

- 정답 16 vs 결과 15 (HWP5/HWPX 모두)
- Stage 1 진단으로 이미 식별. 단일 -1600 값으로는 sample/sample5 모두 정합 불가
- 해결 옵션 (별도 task): 가변 tolerance, 본질적 한글97 모델 재구현

### 4.2 hwp3-sample4.hwp HWP3 자체 회귀 (Task #554 범위 밖)

- HWP3 39 페이지 vs 정답 36 페이지 (+3 회귀)
- HWP3 파서 또는 페이지네이션 별도 결함. 본 task 와는 무관
- 별도 issue 등록 권고

## 5. 작업지시자 승인 요청

본 Stage 2-2 보고서 검토 후:

1. **Stage 2-2 구현 승인**:
   - `apply_hwp3_origin_fixup()` 신규 함수 (PS/CS 비율 휴리스틱)
   - strict + lenient 두 경로에 적용
2. **잔존 사항 (-1 over-correct, hwp3-sample4 HWP3 회귀) 의도된 trade-off 승인**
3. **Stage 2-3 (회귀 검증 + tests/issue_554.rs) 진행 승인**

승인 후 Stage 2-3로 진행한다.
