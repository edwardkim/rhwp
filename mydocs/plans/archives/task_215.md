# Task 215 수행계획서: 단일 패스 레이아웃 엔진 Phase 2 — 표 조판

## 목표

TypesetEngine에 **Break Token 기반 표 조판**을 구현하여, 기존 3단계 파이프라인의 측정-배치 불일치 버그를 구조적으로 해결한다.

### 해결 대상 버그

| 문서 | 현상 | 근본 원인 |
|------|------|----------|
| k-water-rfp.hwp | page 14~15에서 14px overflow, 문단 크롭 | pagination의 host_spacing 계산과 layout의 실제 spacing 불일치 |
| k-water-rfp.hwp | TYPESET_VERIFY sec1: 25→27페이지 차이 | TypesetEngine 표 분할 로직 미비 |
| kps-ai.hwp | TYPESET_VERIFY sec0: 79→75 차이 | 동일 |
| hwpp-001.hwp | TYPESET_VERIFY sec3: 57→55 차이 | 동일 |

### 설계 원칙 (선구자 엔진에서 학습)

1. **Chromium LayoutNG**: Break Token 패턴 — `layout(node, constraint) → (fragment, Option<break_token>)`
2. **LibreOffice Writer**: Master/Follow Chain — 분할 시 재개 정보 보존
3. **MS Word/OOXML**: cantSplit, tblHeader — 행 분할 규칙과 머리행 반복
4. **공통**: 측정과 배치를 분리하지 않고, **배치하면서 측정**한다

상세: `mydocs/tech/investigations/issue-397/layout_engine_research.md`

---

## 구현 계획 (4단계)

### 1단계: BreakToken 자료구조 및 format_table() 구현

**목표**: 표의 높이를 단일 패스로 계산하는 format_table() 구현

**작업 내용**:

1. `TypesetBreakToken` enum 정의
   ```rust
   enum TypesetBreakToken {
       Paragraph { para_index: usize, start_line: usize, suppress_spacing_before: bool },
       Table { para_index: usize, control_index: usize, start_row: usize,
               header_row_count: usize, cell_breaks: Option<Vec<CellBreakToken>> },
   }
   struct CellBreakToken { cell_index: usize, content_offset: f64 }
   ```

2. `FormattedTable` 구조체 — 표의 format() 결과
   ```rust
   struct FormattedTable {
       row_heights: Vec<f64>,      // 각 행의 실제 높이
       cell_spacing: f64,           // 행간 간격
       header_row_count: usize,     // 머리행 수
       host_spacing: HostSpacing,   // spacing_before/after + outer_margin + host_line_spacing
       total_height: f64,           // 전체 높이 (host_spacing 포함)
   }
   struct HostSpacing { before: f64, after: f64 }
   ```

3. `format_table()` 구현
   - 기존 height_measurer의 MeasuredTable 데이터를 활용
   - host_spacing을 **layout과 동일한 규칙**으로 계산
   - spacing_before/after, outer_margin, host_line_spacing 통합

**검증**: 기존 MeasuredTable과 FormattedTable의 total_height 비교 (차이 리포트)

---

### 2단계: typeset_table() — fits → place / split 구현

**목표**: 표의 페이지 분할을 Break Token 패턴으로 구현

**작업 내용**:

1. `typeset_table()` — 핵심 로직
   ```
   fn typeset_table(table, formatted, constraint, break_token)
       → (items: Vec<PageItem>, consumed_height: f64, Option<TypesetBreakToken>)
   ```

   흐름:
   ```
   1. 머리행 배치 (break_token.header_row_count 참조)
   2. 시작 행 결정 (break_token.start_row 또는 0)
   3. 각 행에 대해:
      a. 행 높이 = formatted.row_heights[ri] + cell_spacing
      b. cumulative + row_h ≤ available?
         - YES: 행 배치, cumulative += row_h
         - NO + 이전 행 있음: BreakToken(start_row=ri) 반환
         - NO + 첫 행: 인트라-로우 분할 또는 강제 배치 (monolithic)
   4. 모든 행 완료: (items, consumed, None)
   ```

2. `typeset_table_paragraph()` 리팩터링
   - 기존 MeasuredTable 의존 → FormattedTable + typeset_table() 호출
   - host_spacing.before: 단 상단이면 0, 아니면 spacing_before
   - host_spacing.after: 항상 적용

3. 인트라-로우 분할 (Phase 2.5로 분리 가능)
   - 셀 내 문단을 줄 단위로 분할하는 것은 복잡도가 높음
   - 우선 행 단위 분할에 집중, 인트라-로우는 기존 로직 유지

**검증**: k-water-rfp.hwp, kps-ai.hwp에서 overflow 0건 목표

---

### 3단계: 머리행 반복, 각주, 다중 TAC 표 처리

**목표**: 기존 Paginator의 모든 표 분할 기능을 TypesetEngine으로 이전

**작업 내용**:

1. **머리행 반복**
   - `table.header_row_count`에 따라 Break Token에 header 정보 저장
   - 분할 후 다음 페이지에서 머리행 재배치
   - PartialTable에 `header_rows` 정보 전달

2. **각주 수집**
   - 표 셀 내 각주(Footnote) 컨트롤 탐지
   - 각주 높이를 available_height에서 차감
   - 기존 paginator의 footnote 수집 로직 재현

3. **다중 TAC 표**
   - 한 문단에 여러 TAC 표가 있는 경우
   - LINE_SEG 기반 개별 표 높이 계산
   - 기존 paginator의 tac_table_count 로직 재현

4. **host_spacing 정밀 계산**
   - 비-TAC 표: spacing_before 조건부 제외 (text_wrap=1)
   - host_line_spacing: 비-TAC 표에서 호스트 문단의 trailing line_spacing 포함
   - 단 상단 spacing_before 억제

**검증**: TYPESET_VERIFY 표 구역 차이 0건 목표

---

### 4단계: 병렬 검증 강화 및 정리

**목표**: TypesetEngine을 Paginator와 완전 일치시키고, 전환 준비

**작업 내용**:

1. **TYPESET_VERIFY 강화**
   - 페이지 수뿐 아니라 각 페이지의 항목(PageItem) 단위 비교
   - 불일치 시 상세 diff 출력 (어떤 문단/표가 다른 페이지에 배치되었는지)

2. **overflow 검증 연동**
   - TypesetEngine 결과로 layout 실행 시 LAYOUT_OVERFLOW 0건 확인
   - 기존 Paginator 결과와 TypesetEngine 결과 모두에서 overflow 검사

3. **코드 정리**
   - TypesetEngine Phase 1 호환 스텁(process_table_controls, split_table_into_pages) 제거
   - 불필요한 MeasuredTable 의존 경로 정리

4. **테스트 확충**
   - 표 분할 단위 테스트 (1행, 다행, 머리행 반복, 인트라-로우)
   - 실제 HWP 파일 비교 테스트 확충

**검증**:
- 전체 테스트 PASS
- WASM 빌드 성공
- TYPESET_VERIFY 모든 문서 차이 0건
- LAYOUT_OVERFLOW 기존 대비 감소 또는 0건

---

## 핵심 설계 포인트

### A. host_spacing 통일 — 가장 중요한 수정

현재 버그의 직접 원인은 **pagination과 layout에서 host_spacing 계산이 다른 것**이다.

기존 Paginator의 host_spacing 계산 (engine.rs:591-624):
```
host_spacing = before + sa + outer_bottom + host_line_spacing
```

이 계산이 layout의 실제 spacing과 불일치하여 overflow 발생.

TypesetEngine에서는 **layout과 동일한 규칙을 format_table()에서 한 번만 계산**하고, 이 값을 fits/place/split 모든 단계에서 사용한다.

### B. Break Token으로 상태 전달

기존: 분할 시 start_row/end_row를 직접 계산하여 PartialTable에 삽입
신규: Break Token이 "어디서 재개"할지를 명시적으로 전달

```
// 기존
PartialTable { start_row: 3, end_row: 7, ... }

// 신규
typeset_table(table, formatted, available, Some(BreakToken::Table { start_row: 3, ... }))
→ (items, consumed, Some(BreakToken::Table { start_row: 7, ... }))
```

### C. 측정-배치 일원화

format_table()이 반환하는 높이값은 typeset_table()이 배치에 사용하는 높이값과 **정확히 동일**하다. 별도의 height_measurer를 거치지 않으므로 불일치가 구조적으로 불가능하다.

---

## 예상 성과

| 항목 | Phase 1 (현재) | Phase 2 (완료 시) |
|------|---------------|------------------|
| k-water-rfp OVERFLOW | 6건 (14px 포함) | 0건 |
| TYPESET_VERIFY 표 구역 차이 | 3개 문서 차이 | 0건 |
| 측정-배치 불일치 가능성 | 있음 (3단계 파이프라인) | 없음 (단일 패스) |
| 표 분할 정확도 | MeasuredTable 의존 | Break Token 기반 |
