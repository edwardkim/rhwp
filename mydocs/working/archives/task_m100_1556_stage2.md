# 단계 2 완료보고서 — Task #1556

## 목표
직렬화기가 `orphan_field_ends` 를 `<hp:fieldEnd>` 로 복원(8유닛 슬롯 소비)하도록 확장.

## 변경 사항

### `src/serializer/hwpx/field.rs`
- `write_field_end_full(w, begin_id_ref, field_id)` 추가 — `beginIDRef`+`fieldid` 동시 방출
  (`field_id == 0` 이면 `fieldid` 생략).

### `src/serializer/hwpx/section.rs`
- `emit_orphan_field_end(out, ofe)` 헬퍼 추가 (`<hp:ctrl><hp:fieldEnd .../></hp:ctrl>`).
- **`inferred_control_slot_count`**: 반환식에 `orphan_field_ends.len()` `saturating_sub`
  추가 → 고아 fieldEnd(컨트롤 없는 8유닛 슬롯)를 슬롯 추정에서 제외, 메인 경로 진입.
  (핵심 정합 포인트 — 누락 시 mismatch 경로로 빠져 무효과.)
- **fast-path** 조건에 `&& para.orphan_field_ends.is_empty()` 추가.
- **메인 루프**:
  - pre-char: `char_idx == idx` 고아를 문자 push 전 방출(+8).
  - post-loop: `char_idx == text_char_count`(텍스트 끝) 고아 방출 — para 0.16 케이스.
  - 빈 문단(text=="") 의 `char_idx == 0` 고아도 별도 처리.
- **mismatch 경로**: 위치 추정 불가 시에도 말미 일괄 방출로 char_count 보존(안전망).

## 검증

### 단위 테스트 (`cargo test --lib task1556`, 4건 통과)
- (Stage1) 파서 2건 + (Stage2) 직렬화 2건:
  - `task1556_orphan_field_end_emitted_at_text_end`: 텍스트 뒤 `<hp:fieldEnd
    beginIDRef="1878228493" fieldid="627272811"/>` 방출·순서 검증.
  - `task1556_orphan_field_end_zero_fieldid_omits_attr`: `field_id=0` → `fieldid` 생략.

### 실문서 roundtrip (`rhwp hwpx-roundtrip`)
- 대표 4건(dt2854/dt2906/dt2952/dt3004) **diff=0 PASS** (수정 전 diff=1).
- 2023-01-05 폴더 60건 재스캔: **IR_DIFF 24 → 1**.
  - 잔여 1건(dt3001)은 `ir-diff` 0건 / roundtrip diff=1 = **그림 BinData 누락**
    (`Picture 직렬화 실패: BinDataContent 누락`) — #1552 F1 계열 별개 결함, #1556 범위 밖.

### 회귀 가드
- `cargo test --test hwpx_roundtrip_baseline` (큐레이션 전수) **무회귀 통과**.

## 다음 단계
단계 3 — 합성 parse→serialize→parse roundtrip 테스트 + 실문서 `samples/hwpx/` 추가.
