# Task #412 구현 계획서

**제목**: 다단 우측 단 단행 문단 줄간격 누락 — vpos 보정 공식의 base 차감 무력화

**Issue**: #412 / **브랜치**: `local/task412`

---

## 1. 핵심 결정 사항

### 1.1 vpos_page_base vs vpos_lazy_base 분리 fix

수행 계획서에서 옵션 A(base 차감 제거)를 채택했으나, **두 경로의 의미가 다름**을 분석한 결과 다음과 같이 정밀화한다:

| 경로 | 의미 | 첫 항목의 실제 y | 올바른 보정 공식 |
|------|------|------|------|
| `vpos_page_base` | 첫 항목이 명확한 vpos 를 가짐 (FullParagraph/PartialParagraph/Table) | `col_area.y + base * scale` | `col_area.y + vpos_end * scale` (base 차감 제거) |
| `vpos_lazy_base` | 첫 항목 vpos 신뢰 불가 (PartialTable/Shape) → sequential y_offset 으로 역산 | 역산 시점의 y_offset | `col_area.y + (vpos_end - lazy_base) * scale` (기존 유지) |

**근거**:
- `vpos_page_base` 경로: 첫 PageItem 은 sequential 로 `col_area.y + base * scale` 위치에 배치된다 (HWP 절대 vpos 좌표). 그러므로 후속 paragraph 의 정확한 y = `col_area.y + vpos_end * scale`.
- `vpos_lazy_base` 경로: lazy_base 산출 자체가 `lazy_base = vpos_end - y_delta_hu` 형태로 sequential y_offset 와의 차이를 흡수하므로 base 차감이 의미 있음. 좌측 단(첫 PageItem 이 Shape) 등이 이 경로.

### 1.2 좌측 단 동작 재검증

좌측 단 첫 PageItem (pi=0 Shape) → `vpos_page_base = None` → lazy_base 경로 → 본 fix 영향 없음. 따라서 fix 는 **`vpos_page_base` 경로에서만** 동작 변경.

우측 단 첫 PageItem (pi=33 FullParagraph) → `vpos_page_base = Some(7060)` → 본 fix 적용.

## 2. 단계별 작업

### Stage 1 — 진단 보강 + 좌측 단 회귀 가설 검증

**목표**: `vpos_page_base` 경로 변경이 영향 미치는 케이스 사전 식별.

작업:
1. 임시 진단 코드로 페이지별 (vpos_page_base 경로 사용 여부, 첫 항목 종류, base 값, 적용 여부) 수집.
2. 회귀 후보 샘플(`exam_eng`, `exam_kor`, `k-water-rfp`, `2025년 기부·답례품 실적`, `aift`, `kps-ai`) 의 페이지별 base 분포 통계.
3. base 가 큰 page_base 경로 케이스 목록 → fix 후 visual 비교 대상 리스트.

산출물: `mydocs/working/task_m100_412_stage1.md` (진단 결과 표).

### Stage 2 — 코드 수정 + exam_eng p1 검증

**목표**: `layout.rs:1392-1430` 의 `vpos_page_base` 경로에서 base 차감 제거.

변경 (의사코드):

```rust
let (base, is_lazy) = if let Some(b) = vpos_page_base {
    (b, false)
} else if let Some(b) = vpos_lazy_base {
    (b, true)
} else {
    // ... 지연 산출 ...
};

let end_y = if is_lazy {
    col_area.y + hwpunit_to_px(vpos_end - base, self.dpi)
} else {
    col_area.y + hwpunit_to_px(vpos_end, self.dpi)
};
```

검증:
1. `exam_eng.hwp -p 0` 우측 단 7번~12번 선택지 y delta 측정 → 각각 22.55 px 근방 확인 (좌측 단과 동일 패턴).
2. 좌측 단 1번~6번 영향 없음 확인 (lazy_base 경로 사용).

산출물:
- 코드 변경 (1 hunk).
- `mydocs/working/task_m100_412_stage2.md` (전후 측정값 표).

### Stage 3 — 다중 샘플 회귀 검증

**목표**: 변경이 다른 샘플에 회귀를 유발하지 않음을 확인.

검증 대상:
- `exam_eng.hwp` 전체 8페이지 (다단 + 표 + 그림)
- `exam_kor.hwp` 전체 (다단 + 그림 + 폰트 다양)
- `exam_math.hwp` (다단 + 수식)
- `k-water-rfp.hwp` (단단 + 표 + 페이지 분할)
- `2025년 기부·답례품 실적 지자체 보고서_양식.hwp` (단단 양식)
- `aift.hwp` 일부 (단단 일반)
- `kps-ai.hwp` 일부

방법:
1. 변경 전후 SVG 산출 및 (page_count, total_text_y_avg, byte size diff) 자동 비교.
2. Stage 1 에서 식별한 page_base 경로 케이스 시각 비교 (qlmanage thumbnail).
3. `cargo test` 통과.

산출물: `mydocs/working/task_m100_412_stage3.md` (샘플별 결과 + 변동 페이지 분석).

### Stage 4 — 최종 보고서

산출물:
- `mydocs/report/task_m100_412_report.md`
- `mydocs/orders/{yyyymmdd}.md` 갱신 (해당 일 작업이면).

## 3. 회귀 시나리오 및 대응

| 시나리오 | 영향 | 대응 |
|---------|------|------|
| `vpos_page_base` 경로에서 첫 항목이 col_area.y 가 아닌 곳에 sequential 배치되는 케이스 | 보정값 잘못 산출 | Stage 1 진단으로 사전 식별. 만약 발견되면 `first_item_actual_y` 추적 변수 도입 |
| 페이지 분할(PartialParagraph) 시 page_base 의 vpos 가 page-relative 가 아닌 경우 | 보정 누적 오류 | PartialParagraph 분기 별도 검토 |
| 회귀 샘플에서 LAYOUT_OVERFLOW 발생 | 페이지 하단 클램프 | 회귀 시 base 차감 유지하는 옵션 분기 추가 검토 |

## 4. 롤백 기준

- Stage 3 회귀 검증에서 1개 이상 샘플의 페이지 수 변화 또는 LAYOUT_OVERFLOW 신규 발생 시 → 본 fix 보류, 더 좁은 분기로 재설계 (e.g. 다단 단(`col_index >= 1`) 에서만 적용).

## 5. 일정

- Stage 1: 30min
- Stage 2: 30min
- Stage 3: 45min
- Stage 4: 15min
- 총 ≈ 2h

## 6. 승인 요청 사항

- 단계 분할 (4단계) 승인
- vpos_page_base / vpos_lazy_base 분리 fix 방향 승인
- Stage 1 진행 승인
