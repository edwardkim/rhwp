# Task #1058 최종 보고서 — 글상자 LIST_HEADER 13 byte 한컴 contract 정합 (Task #1050 후속)

- 이슈: [#1058](https://github.com/edwardkim/rhwp/issues/1058) (closes)
- 마일스톤: M100 (v1.0.0)
- 브랜치: `local/task1058`
- assignee: @edwardkim
- 일시: 2026-05-21
- 수행 계획서: [task_m100_1058.md](../plans/archives/task_m100_1058.md)
- 구현 계획서: [task_m100_1058_impl.md](../plans/archives/task_m100_1058_impl.md)
- 단계별: [stage1](../working/task_m100_1058_stage1.md) / [stage2](../working/task_m100_1058_stage2.md) / [stage3](../working/task_m100_1058_stage3.md)

## 1. 이슈 본질

Task #1050 (HWPX 각주 한컴 호환) 후속 결함:
- 한컴편집기에서 rhwp 저장 HWP 의 기존 각주 사이에 신규 각주 추가 시 본문 다단계 목록 번호 **"1.1.1.1.1.1."** 자동 부여
- rhwp-studio 정상 동작 (한컴편집기에서만 발생)

## 2. 본질 (Stage 0~1 진단)

작업지시자 지시 "정답지와 저장 버전과 차이를 통해 추론" — HWP 정답지 vs rhwp 저장본 hwp5-inventory-diff 잔여 차이:

| 위치 | 정답지 | rhwp | 의미 |
|------|--------|------|------|
| **#18 LIST_HEADER tuple=0** | **size=33** | **size=20** | **글상자 (TextBox) LIST_HEADER 13 byte 누락** |

`hwplib::ForTextBox::listHeader` 권위 자료로 13 byte contract 완전 규명:

| Offset | Size | Field | Type | Default |
|--------|------|-------|------|---------|
| 20-27 | 8 | zero padding | bytes | 0 |
| 28-31 | 4 | editableAtFormMode | SInt4 | 0 (false) |
| 32 | 1 | fieldName flag | UInt1 | 0 (no fieldName) |

→ rhwp 의 누락 13 byte = `zero(8) + editable(4) + flag(1)`.

**가설 (확인됨)**: 글상자 LIST_HEADER 가 한컴 contract 33 byte 보다 짧으면 한컴이 글상자 안 paragraph 를 본문 list 의 일부로 잘못 해석 → 신규 paragraph 추가 시 본문 다단계 목록 번호 자동 부여.

## 3. 변경 사항

### 3.1 `src/serializer/control.rs::serialize_text_box_if_present` (+10 라인)

```rust
// [Task #1058] hwplib::ForTextBox::listHeader 정합 — TextBox LIST_HEADER 의
// 마지막 13 byte 필드 contract:
//   sw.writeZero(8);                 // 8 byte zero padding
//   sw.writeSInt4(editableAtFormMode); // 4 byte (0 = false)
//   sw.writeUInt1(fieldNameFlag);     // 1 byte (0 = no fieldName)
if !text_box.raw_list_header_extra.is_empty() {
    w.write_bytes(&text_box.raw_list_header_extra).unwrap();
} else {
    // HWPX 출처: 한컴 default 13 byte (zero 8 + editable 0 + fieldName flag 0)
    w.write_bytes(&[0u8; 13]).unwrap();
}
```

### 3.2 `tests/issue_1058_textbox_list_header.rs` (148 라인, 신규)

회귀 가드 4 tests:
- `issue_1058_textbox_list_header_size_33` — HWPX → HWP 글상자 LIST_HEADER size=33
- `issue_1058_hwp_textbox_roundtrip` — HWP 출처 회귀 부재
- `issue_1058_footnote_list_header_size_16_preserved` — Task #1050 양립
- `issue_1058_textbox_list_header_byte_contract` — byte-by-byte 정합

## 4. 검증 결과

### 4.1 자동 검증

| 항목 | 결과 |
|------|------|
| cargo build --release --bin rhwp | OK |
| cargo build --lib | OK |
| cargo test --release --lib | **1323 passed** (Task #1050 1319 + 본 task 회귀 4) |
| cargo test --release --tests | FAILED 0 (전체 통합) |
| **회귀 가드** `tests/issue_1058_textbox_list_header.rs` | **4/4 passed** |
| Task #1050 회귀 가드 양립 | 7/7 passed |
| cargo clippy --release --lib -D warnings | clean |
| cargo fmt --all --check | clean |
| WASM Docker 빌드 | OK (4.91 MB) |
| rhwp-studio 동기화 | OK |

### 4.2 광범위 sweep (14 fixtures, 1143 SVG)

작업지시자 선택 B (변환본 포함):

| Fixture 카테고리 | 페이지 수 | BEFORE/AFTER diff |
|------|----------|------|
| footnote-tbox-01 (HWPX+HWP) | 1+1 | 0 |
| footnote-01 (HWPX+HWP) | 6+6 | 0 |
| 2010-01-06 + table-in-tbox | 6+2 | 0 |
| 변환본 4종 (hwp3-sample-hwp5) | 16+763+151+64 | 0 |
| 일반 4종 (aift/KTX/biz_plan/exam_kor) | 74+27+6+20 | 0 |

```
diff -rq output/poc/issue_1058/sweep-before/ output/poc/issue_1058/sweep-after/ = 0
```

→ **14 fixture 1143 SVG 완전 동일** (직접 export 회귀 부재 정량 입증).

### 4.3 hwp5-inventory-diff TextBox LIST_HEADER

```
LIST_HEADER 차이 0건 (size=33 + raw byte 완전 동일)
```

### 4.4 작업지시자 한컴 한글 2020 시각 판정 — 통과 ✓

- 신규 각주 추가 시 본문 다단계 목록 "1.1.1.1.1.1." **부여 안 됨**
- Task #1050 통과 영역 (각주 영역 조판) 회귀 부재

## 5. 성공 기준 충족

| 기준 | 내용 | 결과 |
|------|------|------|
| C1 | 글상자 LIST_HEADER size=33 한컴 정합 | ✓ |
| C2 | rhwp 자기 라운드트립 회귀 부재 | ✓ |
| C3 | **한컴편집기 시각 판정 — 다단계 목록 부여 안 됨** | ✓ |
| C4 | 회귀 가드 영구화 | ✓ tests/issue_1058 (4) |
| C5 | 일반 fixture 회귀 부재 | ✓ sweep diff=0 |
| C6 | 자동 검증 통과 | ✓ |
| C7 | Task #1050 회귀 가드 7/7 유지 | ✓ |

## 6. 메모리 룰 정합

- ✅ `feedback_search_troubleshootings_first` — `hwpx2hwp-rule.md` + `task178_*` 사전 정독
- ✅ `feedback_diagnosis_layer_attribution` — Stage 0 정답지 vs 저장본 record-level 비교로 본질 (글상자 LIST_HEADER 13 byte 누락) 정확 식별
- ✅ `feedback_self_verification_not_hancom` — rhwp 자기 정합 통과 ≠ 한컴 호환. 작업지시자 한컴 직접 검증 게이트 통과
- ✅ `feedback_visual_judgment_authority` — 한컴 한글 2020 시각 판정 통과
- ✅ `feedback_hancom_compat_specific_over_general` — case-specific 한컴 contract (글상자 LIST_HEADER 13 byte default) 보존, raw_list_header_extra 우선 + HWPX default fallback
- ✅ `feedback_push_full_test_required` — lib + tests + clippy + fmt 모두 통과
- ✅ `project_hwpx_to_hwp_adapter_limit` — **HWP IR oracle 방식** (Task #1050 Stage 4-pivot 패턴) 적용 — 정답지 record 와 저장본 record 의 정확한 차이 추출로 contract unit 추가

## 7. 의의 — `hwpx2hwp-rule.md` contract unit 추가

본 task 는 Task #1050 의 Stage 4-pivot 통찰 (HWP IR oracle) 의 동일 패턴 적용. `hwpx2hwp-rule.md` 의 contract unit 추가:

- **TextBox LIST_HEADER 33 byte contract**:
  - 마지막 13 byte = `zero(8) + editableAtFormMode(SInt4=0) + fieldName flag(UInt1=0)`
  - 한컴 정합 default: HWPX 출처는 모두 0 으로 fallback
  - hwplib `ForTextBox::listHeader` 권위 참조

## 8. 잔여 / 후속

- 본문 PARA_HEADER #0 의 break_type/num_char_shapes 차이 (1 byte hash 변동) — 본 task 본질과 다른 영역
- FOOTNOTE_SHAPE tuple=2 (endnote shape) 1건 잔여 — endnote noteLine 매핑
- 미주 (Endnote) 의 한컴 시각 검증
- 광범위 HWPX → HWP 호환 (Task #178 영역)
