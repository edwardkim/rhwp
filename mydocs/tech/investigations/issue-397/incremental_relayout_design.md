---
kind: investigation
status: historical
canonical: mydocs/tech/investigations/issue-397/README.md
last_verified: 2026-07-17
---

# rhwp 편집 시 재조판 문제 분석 및 개선 설계안

> 2026-03-28 | Task 397 후속 — 편집 시 자연스러운 재조판을 위한 정밀 분석

---

## 1. 현재 이미 구현된 증분 처리 메커니즘

rhwp는 이미 상당 수준의 증분 레이아웃 메커니즘을 보유하고 있다.

### 1.1 dirty 플래그 시스템

| 메커니즘 | 위치 | 역할 |
|----------|------|------|
| `dirty_sections: Vec<bool>` | document.rs | 섹션 수준 dirty 플래그 |
| `dirty_paragraphs: Vec<Option<Vec<bool>>>` | document.rs | 문단 수준 dirty 비트맵 |
| `mark_section_dirty()` | rendering.rs:566 | 섹션만 dirty 마킹 (composed 불변 시) |
| `mark_paragraph_dirty()` | rendering.rs:573 | 특정 문단만 dirty 마킹 |
| `mark_all_sections_dirty()` | rendering.rs:634 | 전체 dirty (초기화 등) |

### 1.2 증분 재조합/측정

| 메커니즘 | 위치 | 역할 |
|----------|------|------|
| `recompose_paragraph()` | rendering.rs:593 | 단일 문단만 재조합 (섹션 전체 아님) |
| `insert_composed_paragraph()` | rendering.rs:602 | 문단 분할 시 새 항목 삽입 |
| `remove_composed_paragraph()` | rendering.rs:617 | 문단 병합 시 항목 제거 |
| `measure_section_selective()` | height_measurer.rs:876 | dirty 문단만 재측정, 나머지 캐시 재사용 |

### 1.3 증분 페이지네이션

| 메커니즘 | 위치 | 역할 |
|----------|------|------|
| `paginate()` dirty 섹션 체크 | rendering.rs:716 | `dirty_sections[idx]`가 false이면 skip |
| `paginate_if_needed()` | rendering.rs:642 | batch_mode 시 페이지네이션 지연 |
| `batch_mode` | rendering.rs:643 | 대량 편집 시 최종 1회만 paginate |

### 1.4 수렴 루프

| 메커니즘 | 위치 | 역할 |
|----------|------|------|
| `para_column_map` 비교 | text_editing.rs:57-70 | 다단 컬럼 재배치 감지 및 반복 처리 |
| 최대 3회 반복 | text_editing.rs:57 | cascade 안정화 |

### 1.5 표 dirty 마킹

| 메커니즘 | 위치 | 역할 |
|----------|------|------|
| `table.dirty = true` | table_ops.rs (10+ 곳) | 표 구조 변경 시 dirty 마킹 |
| `reflow_cell_paragraph()` | table_ops.rs:1077 | 셀 내 문단 개별 reflow |

---

## 2. 실제 남은 갭 (핀셋 문제 목록)

### Gap 1: 수렴 루프가 편집 문단만 감시

**위치**: `text_editing.rs:57-70` (insert), `144-157` (delete)

**현상**: 편집된 문단의 컬럼 배치만 확인. 인접 문단이 밀려나는 cascade를 감지하지 못함.

**예시**: 문단 5가 col 0 → col 1로 이동 시, 기존 col 1에 있던 문단 6이 col 2로 밀리지만 감지 안 됨. 문단 6의 line_segs는 col 1 너비로 계산된 상태 유지.

**영향**: 다단 문서에서 편집 후 인접 문단 레이아웃 깨짐.

---

### Gap 2: split 시 new_para_idx 컬럼 미감시

**위치**: `text_editing.rs:715-727`

**현상**: 문단 분할 후 수렴 루프가 `para_idx`만 확인. 새로 생성된 `new_para_idx`의 컬럼 배치는 확인하지 않음.

**영향**: 분할된 새 문단이 다른 컬럼으로 밀렸을 때 레이아웃 불일치.

---

### Gap 3: merge 시 수렴 루프 부실

**위치**: `text_editing.rs:946-956`

**현상**: 병합 후 `prev_idx`만 확인. 병합으로 사라진 문단 뒤의 문단들이 밀려나는 것을 감지하지 못함.

**영향**: 병합 후 후속 문단 위치 어긋남.

---

### Gap 4: 원본 LINE_SEG 첫 줄만 보존

**위치**: `line_breaking.rs:587, 652`

**현상**: `reflow_line_segs()` 호출 시 첫 번째 LineSeg의 `line_height`, `text_height`, `baseline_distance`만 보존. 2번째 줄 이후는 새로 생성.

**결과**:
- 줄 수가 변할 때 새 줄의 baseline_distance가 원본과 달라짐
- 다중 편집 시 Y 좌표 누적 드리프트

---

### Gap 5: baseline_distance 0.85 휴리스틱

**위치**: `line_breaking.rs:621`

**현상**: `line_height`가 0인 경우 `baseline_distance = (line_height_hwp * 0.85)` 로 추정. 원본 HWP의 실제 baseline과 다를 수 있음.

**영향**: 편집 후 글자 수직 위치가 미세하게 달라짐.

---

### Gap 6: 셀 편집 시 composed 미갱신

**위치**: `text_editing.rs:1005-1007` (split_paragraph_in_cell_native)

**현상**: 셀 내 문단 분할/병합 시 `mark_section_dirty()`만 호출. `recompose_paragraph()`를 호출하지 않아 부모 문단의 composed 데이터가 즉시 갱신되지 않음.

**비교**: 본문 텍스트 편집은 `recompose_paragraph()`로 즉시 갱신.

**영향**: 셀 편집 후 커서 위치, 렌더링이 일시적으로 부정확.

---

### Gap 7: 페이지/컬럼 브레이크 삽입 시 불필요한 전체 재조합

**위치**: `text_editing.rs:777` (page break), `text_editing.rs:824` (column break)

**현상**: `recompose_section()` 호출 → 섹션 내 모든 문단 재조합. 실제로는 분할된 2개 문단만 재조합하면 충분.

**비교**: `split_paragraph_native()`는 `recompose_paragraph()` 2회로 처리.

**영향**: 200개 문단 섹션에서 페이지 브레이크 삽입 시 ~100배 불필요 작업.

---

### Gap 8: vpos cascade 후 인접 문단 reflow 미실행

**위치**: `line_breaking.rs:722-740`

**현상**: `recalculate_section_vpos()`가 후속 문단의 vpos를 일괄 조정하지만, 해당 문단의 line_segs는 갱신하지 않음. 다단 레이아웃에서 컬럼 경계를 넘는 경우 문제.

**영향**: 다단 문서에서 vpos 변경 후 컬럼 오버플로 미감지.

---

## 3. v1.0.0 전략: 한컴 동일 조판 구현

### 3.1 전략 비교

| | 전략 A: 원본 LINE_SEG 존중 (현행) | 전략 B: 자체 조판 (일부 상용 제품 접근) | **전략 C: 한컴 동일 구현 (v1.0.0 목표)** |
|---|---|---|---|
| 뷰어 정확도 | 높음 (원본 의존) | 차이 발생 | **한컴과 동일** |
| 편집 자연스러움 | 불연속 (첫 편집 시 점프) | 자연스러움 | **자연스러움** |
| 복잡도 | LINE_SEG 보존 + reflow 이중 관리 | 단일 파이프라인 | **단일 파이프라인** |
| 핵심 과제 | 원본/자체 불일치 관리 | 한컴과 다른 결과 수용 | **한컴 줄바꿈/배치 알고리즘 역공학** |

**전략 C 선택**: 한컴의 LINE_SEG 생성 로직 자체를 정확히 역공학하여, rhwp의 자체 조판 결과가 한컴 원본과 동일해지도록 한다.

### 3.2 전략 C가 해결하는 것

- 파일 로드 시 원본 LINE_SEG와 자체 계산 결과가 **일치**
- 편집 시 자체 reflow 결과도 한컴과 **동일**
- 원본 보존 vs 자체 조판의 이중 관리가 **불필요** (Gap 4 해소)
- baseline_distance 휴리스틱이 **불필요** (Gap 5 해소)
- 첫 편집 시 레이아웃 점프가 **사라짐**
- 뷰어 정확도와 편집 자연스러움을 **동시 달성**

### 3.3 역공학 대상

한컴의 조판 결과를 rhwp가 동일하게 재현하려면 다음 요소의 정확한 역공학이 필요하다:

| 역공학 대상 | 현재 상태 | 검증 방법 |
|------------|----------|----------|
| **줄바꿈 알고리즘** | 자체 `fill_lines()` (한컴과 차이 있음) | 원본 LINE_SEG의 text_start와 비교 |
| **텍스트 폭 측정** | 내장 폰트 메트릭 582개 | 원본 segment_width와 비교 |
| **baseline_distance 계산** | 0.85 휴리스틱 | 원본 baseline_distance와 비교 |
| **line_height / text_height** | 첫 줄 원본 복사 | 원본 전체 줄과 비교 |
| **line_spacing 계산** | ParaShape 기반 | 원본 line_spacing과 비교 |
| **문단 간격 (spacing_before/after)** | 자체 구현 | 원본 vpos 차이와 비교 |
| **탭 폭 계산** | 자체 구현 | 원본 segment_width와 비교 |

### 3.4 검증 체계

`dump` 명령으로 원본 LINE_SEG를 추출하고, rhwp 자체 reflow 결과와 1:1 비교하는 테스트 프레임워크 구축:

```
원본 LINE_SEG (HWP 파일)     rhwp reflow 결과
───────────────────────     ──────────────────
text_start: 0               text_start: 0        ✓ 일치
text_start: 28              text_start: 31       ✗ 줄바꿈 위치 다름
baseline_distance: 612      baseline_distance: 595  ✗ 차이 17
segment_width: 34200        segment_width: 34180    ✗ 차이 20 (폰트 메트릭)
```

샘플 HWP 파일 세트에 대해 LINE_SEG 일치율을 측정하고, 불일치 원인을 하나씩 역공학으로 해소한다.

### 3.5 우선순위

#### 1단계: 역공학 기반 조판 정합성 (근본 과제)

| 순서 | 대상 | 접근 |
|------|------|------|
| 1-1 | LINE_SEG 일치율 측정 인프라 | 원본 vs reflow 자동 비교 테스트 |
| 1-2 | 줄바꿈 알고리즘 정합성 | text_start 불일치 패턴 분석 → fill_lines() 개선 |
| 1-3 | 텍스트 폭 측정 정합성 | segment_width 불일치 → 폰트 메트릭 보정 |
| 1-4 | baseline/line_height 정합성 | 원본 값과 비교 → 계산식 역공학 |
| 1-5 | 문단 간격/vpos 정합성 | 원본 vpos와 비교 → spacing 계산 보정 |

#### 2단계: 파생 문제 수정 (조판 정합성 달성 후)

| 순서 | Gap | 개선 방향 |
|------|-----|----------|
| 2-1 | Gap 1,2,3 | 수렴 루프 감시 범위 확장 |
| 2-2 | Gap 6 | 셀 편집 composed 즉시 갱신 |
| 2-3 | Gap 7 | 페이지/컬럼 브레이크 증분 재조합 |
| 2-4 | Gap 8 | vpos cascade 컬럼 경계 감지 |

---

## 4. 참고

- [증분 레이아웃 아키텍처 조사](incremental_layout_research.md) — LibreOffice, Typst, Google Docs, xi-editor 패턴 비교
- [표 객체 처리 아키텍처 현황](../../../report/archives/table_architecture_review.md)
- [텍스트 레이아웃 기술 리뷰](text_layout_review.md)
