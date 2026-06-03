---
타스크: #178 HWPX→HWP IR 매핑 어댑터
단계: Stage 4 — SectionDef 컨트롤 삽입 + typeset 버그픽스
브랜치: local/task178
작성일: 2026-04-19
선행: Stage 3 완료
---

# Stage 4 단계별 완료 보고서

## 1. 진행 과정 — 진단 + 발견 + 수정의 반복

Stage 4 는 본 타스크에서 가장 많은 통찰이 누적된 단계.

### 1.1 hwp2hwpx 라이브러리 리뷰 (작업지시자 지시)

`/home/edward/vsworks/hwp2hwpx` (Apache 2.0, hwplib/hwpxlib 저자) 리뷰:

- HWP↔HWPX 의 표/셀/문단/lineseg 가 enum/필드 단위 1:1 매핑 (Stage 2/3 명세 정확성 확인)
- `cell.hasMargin ↔ apply_inner_margin` 직접 매핑 (Stage 3 의미 확인)
- **lineseg vpos 별도 계산 영역 아님** (양 포맷 동일 의미, 1:1 매핑)
- 본 라이브러리는 변환만 다루고 렌더링 미고려 — 우리 페이지네이터 휴리스틱 문제는 못 잡음

### 1.2 페이지 폭주의 진짜 원인 추적

진단 도구로 측정:
- 어댑터 적용 IR vs 재로드 IR 의 vpos 분포 동일 (max=451660)
- 표 26개 / 문단 121개 / lineseg 142개 모두 라운드트립 보존
- **유일한 차이: 재로드 IR 의 PageDef 가 모두 0 (`width=0, height=0, margin=0`)**

→ 진짜 폭주는 **PageDef 손실**.

### 1.3 PageDef 손실의 근본 원인

HWPX 파서가 `<hp:secPr>` 정보를 `Section.section_def` 로 채우지만,
**`Control::SectionDef` 컨트롤을 첫 문단의 `controls` 에 삽입하지 않음**.

HWP 직렬화기 (`serializer/control.rs:40 + 171-241`) 는 `paragraph.controls` 를 순회하며
`Control::SectionDef` 를 만나야 PAGE_DEF / FOOTNOTE_SHAPE / PAGE_BORDER_FILL 출력.

→ HWPX 출처 IR 을 직렬화하면 PAGE_DEF 가 출력되지 않음 → 재로드 시 PageDef 가 0
→ paginator 가 본문 영역 0×0 으로 페이지 분할 → 모든 표가 1 페이지 차지 (169 페이지)

### 1.4 작업지시자 추가 통찰 — vpos 사전계산 불필요

초기 설계에서 `precompute_lineseg_lh_and_vpos` (`reflow_zero_height_paragraphs` 의 어댑터 이식)
도 추가했으나, 작업지시자 지적:

> "이미 hwpx 를 IR 에서 rhwp 렌더링 하면서 vpos는 계산되지 않나요? 그걸 업데이트 시키면
> 별도로 하지 않아도 되지 않나요?"

검증 결과:
- `DocumentCore::from_bytes` → `reflow_zero_height_paragraphs` 가 IR 의 `line_segs[].vertical_pos`
  를 in-place 로 갱신 (`document_core/commands/document.rs:278, 284`)
- 메모리상 IR 에 영구 반영 → 어댑터 시점에 이미 정확
- 직렬화 → 재로드 시 정수 필드 그대로 보존
- **`precompute_lineseg_lh_and_vpos` 제거 후에도 페이지 수 100% 회복 유지** (17개 테스트 그린 확인)

→ 어댑터의 핵심 결손은 **SectionDef 컨트롤 삽입 단 하나**. 코드 단순화 + 167줄 절감.

## 2. 산출물

### 2.1 코드 변경

| 파일 | 변경 |
|---|---|
| [src/document_core/converters/hwpx_to_hwp.rs](src/document_core/converters/hwpx_to_hwp.rs) | `insert_section_def_control(section)` 추가 (PAGE_DEF 살리기 — 핵심), 어댑터 호출 순서 정의 |
| [src/renderer/typeset.rs:1582-1588](src/renderer/typeset.rs#L1582-L1588) | `get_table_vertical_offset` 가 `raw_ctrl_data[0..4]` (잘못된 attr 비트) 대신 `table.common.vertical_offset` 사용 (paginator 와 일치) — 기존 버그픽스 |
| [tests/hwpx_to_hwp_adapter.rs](tests/hwpx_to_hwp_adapter.rs) | Stage 4 테스트 +6 |

### 2.2 HWP 직렬화기 변경

**0줄 수정** — 정체성 (구현계획서 §1.0.1) 준수.
typeset.rs 는 renderer 영역으로 직렬화기 변경 정책과 무관 (기존 버그픽스).

## 3. 검증 결과

### 3.1 단위 테스트 (16개)

기존 단위 테스트 16개 그린 유지 (`AdapterReport` 필드 정리에도 회귀 0).

### 3.2 통합 테스트 (17개, +6)

```
stage4_section_def_control_inserted ... ok                ← SectionDef 삽입 발생
stage4_section_def_idempotent ... ok                      ← 2차 호출 시 삽입 0
stage4_page_def_preserved_after_roundtrip ... ok          ← PageDef width/height/margins 보존
stage4_page_count_recovered_hwpx_h_01 ... ok              ← 페이지 수 9 회복
stage4_page_count_recovered_hwpx_h_02 ... ok              ← 페이지 수 9 회복
stage4_page_count_recovered_hwpx_h_03 ... ok              ← 페이지 수 9 회복
```

### 3.3 회귀 (전체 라이브러리)

```
test result: ok. 891 passed; 0 failed; 1 ignored; 0 measured
```

## 4. 페이지 폭주 회복 측정

| 샘플 | 원본 | Stage 1 | Stage 2 | **Stage 4** | 회복 |
|---|---:|---:|---:|---:|---:|
| hwpx-h-01 | 9 | 200 | 169 | **9** | 100% |
| hwpx-h-02 | 9 | 220 | (측정X) | **9** | 100% |
| hwpx-h-03 | 9 | 224 | (측정X) | **9** | 100% |

**3개 디버그 샘플 모두 페이지 수 완전 회복**.

## 5. 어댑터 호출 순서

```rust
pub fn convert_hwpx_to_hwp_ir(doc: &mut Document) -> AdapterReport {
    // 1. SectionDef 컨트롤 삽입 (HWPX 결손 영역 보강)
    for section in &mut doc.sections {
        insert_section_def_control(section, &mut report);
    }
    // 2. 표 ctrl_data + 셀 list_attr (raw_ctrl_data 합성)
    for section in &mut doc.sections {
        for para in &mut section.paragraphs {
            adapt_paragraph(para, &mut report);
        }
    }
    report
}
```

## 6. 핵심 발견 — Stage 4 가 발견한 두 가지 결손

### 6.1 HWPX 파서의 SectionDef 컨트롤 결손

`Section.section_def` 는 채우지만 `paragraph.controls` 에 `Control::SectionDef` 를 삽입하지 않음. 이는 **HWPX 출처 IR 을 HWP 직렬화기에 넣을 때 항상 PAGE_DEF 가 누락되는 원인**. 본 어댑터가 영구 보강.

### 6.2 typeset.rs 의 vertical_offset 추출 버그

`get_table_vertical_offset` 가 paginator 와 다르게 구현돼 있었음:
- `pagination/engine.rs:1786`: `table.common.vertical_offset` (정확)
- `typeset.rs:1582` (수정 전): `raw_ctrl_data[0..4]` (attr 비트를 vertical_offset 으로 잘못 해석)

HWPX 출처 (raw_ctrl_data 비어있음) 에서는 0 반환으로 영향 없었으나, 어댑터가 raw_ctrl_data 를 채우면서 attr 값을 vertical_offset 으로 오인 → typeset 결과 불일치 (TYPESET_VERIFY 경고). 본 단계에서 paginator 와 동일하게 통일.

## 7. 정체성 셀프 체크

- [x] HWP 직렬화기 0줄 수정 (`git diff src/serializer/` 출력 0줄)
- [x] 어댑터는 IR 만 만짐 (`&mut Document` 단일 진입점)
- [x] idempotent (`stage4_section_def_idempotent`, `stage4_*_recovered` 모두 idempotent 함수 호출)
- [x] HWP 출처 보호 (모든 함수가 `is_empty()` / `already_has` 가드)
- [x] renderer 버그픽스는 어댑터와 별개 영역 (직렬화기 비변경)

## 8. Stage 4 의 핵심 교훈 — 단순화의 가치

작업지시자의 vpos 통찰로 **167줄의 어댑터 코드를 제거**하면서도 동일한 결과 (페이지 회복 100%) 를 달성. "**중복을 제거하고 진짜 결손에 집중**" 한 결과:

- 코드 명확성 향상
- 유지보수 부담 감소
- 본 타스크 정체성 ("잘 작동하는 직렬화기 어깨 위에 서자") 더욱 명확화 — 어댑터의 핵심은 **결손 보강** 이지 **재계산 이식** 이 아님

## 9. 다음 단계

Stage 5: 통합 진입점 + 전 영역 결합 검증.

`DocumentCore::export_hwp_with_adapter()` 추가 + idempotent + HWP 출처 no-op 통합 테스트 + 작업지시자 수동 검증 준비.

## 10. 승인 요청

본 단계 완료 보고서 승인 후 Stage 5 착수.
