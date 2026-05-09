# Task #713 Stage 2 (분석) 보고서 — 가설 폐기 + Root cause 재정의

**Issue**: [#713](https://github.com/edwardkim/rhwp/issues/713)
**Stage**: 2 — Root cause 재분석
**작성일**: 2026-05-08

---

## 1. Stage 0/1 가설 폐기

### 가설 H1 (RowBreak 인트라-로우 분할 가드)

`!matches!(mt.page_break, TablePageBreak::RowBreak)` 가드를 인트라-로우 분할 분기에 추가.

**결과**: tests/issue_713 RED→GREEN 전환됐으나, **181 샘플 광범위 회귀 검증에서 3 샘플 페이지 수 증가**:

| 샘플 | Before (정합) | After (회귀) | PDF 권위 |
|------|--------------|-------------|----------|
| `samples/inner-table-01.hwp` | 2 | 3 | 2 (한글 2022) |
| `samples/k-water-rfp.hwp` | 27 | 29 | 27 |
| `samples/synam-001.hwp` | 35 | 39 | 35 |

→ PDF 권위 자료가 RowBreak 표의 **인트라-로우 분할을 허용**하는 것을 확인. H1 가설 폐기.

### 가설 H3 (잔여 < 임계값 시 push)

`MIN_REMAINING_AT_TOP_PX = 20.0` 임계값 추가.

**결과**: typeset.rs + engine.rs 양쪽 가드 추가 후에도 본 결함의 17.6 px 분할 변하지 않음. 활성 분기가 다른 위치로 추정. 추가 트레이스 인스트루먼트로도 출력 0.

→ 본 결함의 페이지 분할 결정이 **다른 코드 경로**를 거치고 있음.

---

## 2. 신규 발견 — enum 매핑 결함

참조 코드 (`HLHwp-Old/Hwp50/Hwp/Include/HwpDef.h`) 의 정답 정의:

```c
HWPTABLE_BREAK_NONE,   // 0 — 나누지 않는다 (table atomic)
HWPTABLE_BREAK_TABLE,  // 1 — 테이블은 나누지만 셀은 나누지 않는다 (행 경계 분할)
HWPTABLE_BREAK_CELL    // 2 — 셀 내의 텍스트도 나눈다 (인트라-로우 분할)
```

HLHwp-Old 의 분할 처리 코드 (`HwpTable.cpp:1802`):
```cpp
if (limit != -1 && _GetPageBreak() == HWPTABLE_BREAK_CELL  // == 2
        && pcelo->size[HWPTABLE_ROW] > limit) {
    pcelo->size[HWPTABLE_ROW] = __max(limit, height);
    pcelo->flags |= HWPCELLLO::bottomSplitted;  // 인트라-로우 분할 적용
}
```

→ **값 2 = 셀 분할 허용 (인트라-로우 분할)**.

### Rust 현재 매핑 (`src/model/table.rs:58-66` + `src/parser/control.rs:255-259`)

```rust
table.page_break = match attr & 0x03 {
    1 | 3 => TablePageBreak::CellBreak,  // ❌ 정답: TABLE (행 경계만)
    2 => TablePageBreak::RowBreak,        // ❌ 정답: CELL (셀 분할)
    _ => TablePageBreak::None,
};

pub enum TablePageBreak {
    None,        // 0
    CellBreak,   // 1 — Rust 정의: 셀 단위 분할
    RowBreak,    // 2 — Rust 정의: 행 경계만 분할
}
```

| attr 값 | HLHwp-Old (정답) | Rust (현재) |
|---------|------------------|-------------|
| 0 | NONE — atomic | None — atomic ✅ |
| 1 | **TABLE — 행 경계 분할** | **CellBreak — 셀 분할** ❌ 거꾸로 |
| 2 | **CELL — 셀 분할** | **RowBreak — 행 경계 분할** ❌ 거꾸로 |

### 현행 동작 분석

본 결함 12x5 표: `attr & 0x03 = 2` → Rust `RowBreak` 로 분류.

Task #474 가 "RowBreak 표는 보호 블록 정책 비적용 (인트라-로우 분할 허용)" 처리:
- 한컴 의도 (값 2 = CELL = 셀 분할 허용) 와 결과적으로 일치
- 즉 **enum 매핑은 거꾸로지만 Task #474 가 그를 보정**해서 한컴 PDF 정합 → 광범위 회귀 0

값 1 표 (Rust `CellBreak`, 한컴 의도는 행 경계만):
- Rust 코드는 `is_row_splittable`/`can_intra_split` 으로만 판단, page_break 모드 무시
- 모든 분할 가능 행에 인트라-로우 분할 적용 → 한컴 의도 (TABLE = 행 경계만) 위반 가능성

값 0 (None) 표:
- Rust 코드 동일 처리 — 분할 명세상 atomic 이지만 실제 처리 미확인

### HWP 스펙 문서 (`mydocs/tech/한글문서파일형식_5.0_revision1.3.md:1568`) 기재

```
| bit 0-1 | 쪽 경계에서 나눔 | 0 | 나누지 않음 |
|         |                  | 1 | 셀 단위로 나눔 |
|         |                  | 2 | 나누지 않음 |
```

→ 스펙 문서의 표현이 모호 (값 0 과 2 모두 "나누지 않음"). HLHwp-Old 코드 동작이 실증적 정답.

---

## 3. 사용자 보고 결함 재해석

사용자: `31페이지 하단이 '국외 한국어교육 지원' 라인은 다음 페이지로 이동되어야 한다`

**해석 가능성**:

| 해석 | 검증 |
|------|------|
| (A) 행 8 전체가 다음 페이지로 (RowBreak 명세) | H1 가설 — 회귀 발생 → 폐기 |
| (B) 분할 잔여 < 임계값 시 push (orphan 휴리스틱) | H3 가설 — 효과 미발현 (코드 경로 미확인) |
| (C) 페이지 31 하단의 시각적 디스플레이 결함 (Task #712 잔재 또는 다른 본질) | 미검증 |
| (D) PDF 권위 자료가 행 8 의 17.6 px 분할을 표시 — 사용자 오해 | 미검증 (PDF 시각 직접 확인 필요) |

**현재 결론**: PDF 시각 직접 확인 없이는 (A)/(B)/(C)/(D) 구분 불가.

---

## 4. 권고

**Task #713 의 본질이 enum 매핑 결함으로 이동**. 다음 액션 후보:

### 옵션 1 — Task #713 enum 매핑 정정 + 광범위 회귀 검증

- `src/model/table.rs::TablePageBreak` enum 정의를 HLHwp-Old/HwpDef.h 정답에 맞춰 변경 (None=0, RowBreak=1, CellBreak=2)
- `src/parser/control.rs:255-259` 의 attr → enum 매핑 수정 (1→RowBreak, 2→CellBreak)
- `src/parser/hwpx/section.rs:588-591` HWPX 파서 동일 정정
- Task #474 의 `allows_row_break_split()` 의미 재검토 (현재 값 2=Rust RowBreak=한컴 CELL 처리 → 정정 후 값 2=Rust CellBreak 가 되어 동일 코드 경로 의미)
- 광범위 회귀 검증 (181 샘플) — 페이지 수 변경 0 기대 (이름만 바뀌고 동작 동일)

### 옵션 2 — Task #713 Close (False positive) + 행 8 분할 결함 재검증

- 사용자 보고 결함의 정확한 시각 검증 (PDF 페이지 31 vs SVG 페이지 36)
- PDF 가 분할 표시면 결함 없음 → close
- PDF 가 분할 표시 없으면 다른 root cause 조사

### 옵션 3 — 분석 보고만 정리하고 진행 보류

- Stage 2 보고서 + 인스트루먼트 revert 만 커밋
- 사용자 의사결정 대기

## 승인 요청

Stage 2 분석 결과:
- H1, H3 가설 모두 폐기
- enum 매핑 결함 발견 (별도 본질)
- 사용자 보고 결함의 root cause 재검증 필요

다음 액션 선택 (옵션 1/2/3) 승인 요청.
