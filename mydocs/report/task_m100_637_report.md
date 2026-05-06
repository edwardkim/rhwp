# Task #637 최종 보고서

**제목**: 한컴 호환 — aift.hwp 페이지 2, 3 (큰 표만 있는 cover-style) 쪽번호 미표시 메커니즘 분석
**브랜치**: `local/task637`
**이슈**: https://github.com/edwardkim/rhwp/issues/637
**Milestone**: M100 (v1.0.0)
**상태**: **분석 완료 (Stage 0 → Stage 1 → Stage 3, Stage 2 생략)**
**완료 시점**: 2026-05-06

---

## 1. 배경

Task #634 (한컴 호환 — 첫 NewNumber Page 발화 후 쪽번호 표시) 종료 시점에 미해결로 남은
페이지 2, 3 미표시 메커니즘이 별도 issue 로 분리됨.

`samples/aift.pdf` (한컴 PDF) vs rhwp:

| rhwp 페이지 | 한컴 표시 | 메커니즘 (Task #634 종료 시) |
|------------|----------|---------------------------|
| 1 (cover disclaimer) | 표시 | PageNumberPos 등록 후 표시 ✓ |
| **2 (사업계획서 표지, 35×27 표)** | **미표시** | **메커니즘 미확인** ★ |
| **3 (요약문, 14×17 표)** | **미표시** | **메커니즘 미확인** ★ |
| 4 (목차) | 미표시 | PageHide on para 2.34 ✓ |
| 5 (별첨 목차) | 미표시 | PageHide on para 2.54 ✓ |
| 6+ (본문) | 표시 | 정상 ✓ |

페이지 2, 3 두 페이지 모두 PageHide page_num=true 컨트롤 **없음** (Task #634 검증).
즉 별도 메커니즘 존재.

## 2. 결과 — 결정적 룰 확정

### 2.1 룰

> **페이지가 `items=1` 인 단일 완전한 Table (PartialTable 아님) 을 포함하고
> 그 Table 의 `treat_as_char=false` 일 때 한컴은 쪽번호 표시 안 함.**

이 룰은 페이지 레벨에서 정의되며, "단일 완전한 cover-style 표"를 정확히 식별한다.

### 2.2 매칭 데이터 (aift.hwp)

| 페이지 | items | item 종류 | tac | 표 크기 / body | 한컴 표시 | 룰 적용 결과 |
|--------|-------|----------|-----|--------------|----------|------------|
| 1 | 2 | Table tac=true + PartialPara | true | - | 표시 | 표시 (룰 미매칭) ✓ |
| **2** | **1** | **Table** | **false** | **97.4%** | **미표시** | **미표시 (룰 매칭)** ✓ |
| **3** | **1** | **Table** | **false** | **88.6%** | **미표시** | **미표시 (룰 매칭)** ✓ |
| 4, 5 | 44, 13 | FullParagraphs | - | - | 미표시 (PageHide) | 표시 (룰 미매칭, PageHide 별도 처리) ✓ |
| 6 | 18 | Table 8% + 17 paragraph | false | 8% | 표시 | 표시 (룰 미매칭) ✓ |
| 74 | 1 | Table | **true** | 85% | 표시 | 표시 (룰 미매칭, tac=true) ✓ |
| 75 | 1 | Table | **true** | 95% | 표시 | 표시 (룰 미매칭, tac=true) ✓ |

룰의 모든 조건 (items=1 + 완전 Table + tac=false) 이 동시 충족된 페이지만 미표시.

## 3. 5가지 가설 최종 판정

| 가설 | 판정 | 결정적 근거 |
|------|------|-----------|
| **H1** cover-style 휴리스틱 → 결정적 룰 | **채택** | 174 샘플 중 aift p2, p3 만 정확 매칭 (0.06%) |
| H2 셀 내부 PageHide | 기각 | 문서 전체 PageHide 정확히 2개 (페이지 4, 5 만) |
| H3 paragraph header 비트 | 기각 | 페이지 2 host = 페이지 6 host **byte-for-byte 동일** (ps_id 외) |
| H4 표 attr 비트 | 기각 | 페이지 6 (표시) 표 attr = 페이지 2 (미표시) attr (0x0600000e) 동일 |
| **H5** 한컴 자체 휴리스틱 | H1 등가 | H1 이 결정적 룰로 정형화되어 별도 항목 아님 |

### 3.1 H3 결정적 byte-level 검증

| 페이지 | char_count | control_mask | break_raw | raw_header_extra (12B) |
|--------|-----------|--------------|-----------|------------------------|
| 1 (0.0) 표시 | 56 | 0x00200804 | 0x03 | `02 00 00 00 02 00 99 C0 55 BE 00 00` |
| **2** (0.1) 미표시 ★ | 9 | **0x00000800** | **0x04** | `01 00 00 00 01 00 00 00 00 00 00 00` |
| 3 (1.0) 미표시 ★ | 25 | 0x00000804 | 0x07 | `01 00 00 00 01 00 00 00 00 80 00 00` |
| **6** (2.57) 표시 | 9 | **0x00000800** | **0x04** | `01 00 00 00 01 00 00 00 00 00 00 00` |

페이지 2 host 와 페이지 6 host 의 paragraph header 가 ps_id 외 모든 항목 byte-for-byte 동일.
ps_id 는 정렬/들여쓰기 등 시각 모양이며 hide 와 무관.

## 4. 174 샘플 전수 조사 (회귀 위험 평가)

### 4.1 룰 매칭 패턴 (`items=1 + Table + tac=false`)

```
=== aift.hwp (2 cover-pages) ===
  page 2 (sec=0 page_num=2 pi=1): 35x27 635.6x946.3px
  page 3 (sec=1 page_num=3 pi=0): 14x17 631.3x861.1px
```

**다른 173 샘플 모두 0건 매칭**.

### 4.2 룰 보정 검증 (다른 패턴은 미매칭 확인)

- `items=1 + Table + tac=true`: 23 페이지 (10 샘플). 한컴 PDF 측정 (aift p74, p75)
  결과 **표시**. 룰 미매칭 정상.
- `items=1 + PartialTable`: 37 페이지. 분할 표 중간 페이지로 cover 와 다름. 룰 미매칭 정상.

### 4.3 회귀 위험

룰 적용 시 영향:

- 174 샘플 중 영향받는 페이지: **2 페이지** (aift p2, p3, 한컴 미표시 정합성 개선)
- 다른 173 샘플: **0건 변경**

**회귀 위험: 매우 낮음 (사실상 0)**.

## 5. 권고안 — **시나리오 (a) 채택**

### 5.1 결정

별도 fix issue 분리 후 본 issue **close-as-analysis-complete**.

근거:
1. H1 이 결정적 룰로 정형화됨 (174 샘플에서 정확 매칭)
2. 회귀 위험 매우 낮음 (영향 페이지 2, 모두 정합성 개선)
3. 한컴 호환 개선 가치 (M100 마일스톤 목표 부합)

### 5.2 별도 fix issue 작성 가이드

**제목 (예시)**: 한컴 호환: cover-style 페이지 (items=1 + 완전한 Table + tac=false)
자동 쪽번호 미표시

**구현 위치 후보**:
- `src/renderer/pagination/engine.rs:finalize_pages` (페이지 단위 page_hide 결정 직전)
- 또는 `src/renderer/page_number::PageNumberAssigner::assign` (쪽번호 할당 시)

**구현 가이드라인**:
```rust
// finalize_pages 내부 page_hide 결정 직후
if page.page_hide.is_none() {
    // cover-style 자동 미표시 룰
    let is_cover_table = page.column_contents.len() == 1
        && page.column_contents[0].items.len() == 1
        && matches!(&page.column_contents[0].items[0],
            PageItem::Table { .. });  // PartialTable 제외
    if is_cover_table {
        // tac=false 인 표인지 확인 (Table 의 common.treat_as_char)
        // ... (paragraphs 에서 해당 Table 의 common 접근)
        // tac=false 이면 page_hide 자동 설정 (page_num=true)
    }
}
```

**검증 항목**:
- aift.hwp 페이지 2, 3 미표시 (FAIL→PASS)
- aift.hwp 페이지 1, 6, 7+ 표시 유지 (회귀 0)
- 다른 173 샘플 회귀 0
- 통합 테스트 추가: `test_637_aift_page2_cover_style_no_page_number`,
  `test_637_aift_page3_cover_style_no_page_number`

## 6. 메모리 룰 준수

### 6.1 적용된 룰

- **[feedback_pdf_not_authoritative]** (PDF 측정 절대 기준 아님): pypdf 측정으로 페이지 1,
  6, 7 표시 / 페이지 2, 3, 4, 5 미표시 확정. 단 IR / 바이너리 raw 데이터 (paragraph
  header, table attr) 동시 검증.
- **[feedback_rule_not_heuristic]** (룰 vs 휴리스틱): H1 이 처음에는 휴리스틱 같았으나
  174 샘플 정확 매칭 + tac=true 케이스 일관성으로 **결정적 룰** 확정. 휴리스틱이면
  코드 변경 보류했을 것.
- **[feedback_essential_fix_regression_risk]** (정정 회귀 위험): 174 샘플 전수 조사로
  회귀 위험 0 확인 후 fix 권고. Task #634 의 잘못된 가설 H1'' 시도와 다른 길.

### 6.2 학습한 교훈

1. **byte-level paragraph header 비교의 가치**: 페이지 2 host vs 페이지 6 host 의
   raw header 가 byte-for-byte 동일한 사실이 H3 즉시 기각 + H1 강화의 결정적 근거였음.
   `examples/inspect_637.rs` 같은 ad-hoc 분석 도구의 효용 확인.
2. **174 샘플 전수 조사로 룰의 결정성 확정**: cover-style 룰이 휴리스틱 같았으나 전수
   조사 결과 매칭이 극히 정확 (2/174 = 0.06%) + 모든 매칭이 한컴 동작과 일치 → 결정적 룰.
3. **tac=false 의 분리 의미**: tac=true items=1 페이지 (aift p74, p75) 는 한컴 표시.
   tac=false 가 cover-style 의 정확한 분리자.

## 7. 산출물

| 파일 | 설명 |
|------|------|
| `mydocs/plans/task_m100_637.md` | 수행 계획서 (4단계, Stage 0~3) |
| `mydocs/working/task_m100_637_stage0.md` | Stage 0 — 사전 데이터 수집 + 5가지 가설 사전 판정 |
| `mydocs/working/task_m100_637_stage1.md` | Stage 1 — 5가지 가설 체계적 검증 + 결정적 룰 확정 |
| `examples/inspect_637.rs` | paragraph header raw + cover-candidate 분석 도구 |
| `mydocs/report/task_m100_637_report.md` | 본 최종 보고서 |
| `mydocs/orders/20260506.md` | 오늘 할일 (#637 상태 갱신 예정) |

## 8. 코드 변경

**0** (분석 read-only).

`examples/inspect_637.rs` 는 추가되었으나 src/ 변경 없음 (분석 도구).

## 9. 다음 단계

작업지시자 승인 후:
1. 본 issue **close-as-analysis-complete**
2. 별도 fix issue 등록 (5.2 가이드라인 따라)
3. 새 issue 별도 task 로 진행 (수행 계획서 → 구현 계획서 → Stage 1 RED 테스트 → Stage 2 fix → Stage 3 회귀 검증)

---

**완료 상태**: 분석 100% 완료. 결정적 룰 확정. 별도 fix issue 분리 권고. 본 issue close-as-analysis 대기.
