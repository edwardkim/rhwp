# 구현 계획서 — Task M100 #1916

## 단계

1. **판별** — 서베이 12건 매직 바이트 전수(12/12 ZIP 실체) + 어댑터 경로
   flowWithText 보존 실측 + plain 경로 소실 지점 특정
   (`serialize_table` CTRL_HEADER 빈 데이터 폴백).
2. **수정** — `serialize_table`: raw_ctrl_data 부재 시
   `serialize_common_obj_attr(&table.common)` 합성 방출.
   - attr=0 → `pack_common_attr_bits` (flow_with_text bit 13, tac bit 0,
     wrap bits 21-23 등 포함)
   - raw 보존 경로(HWP5 파스본)·어댑터 경로(Stage 2 합성 raw)는 분기 불변
3. **핀 + 게이트** — `tests/issue_1916.rs`: raw 없는 표 IR 의 plain 왕복에서
   flowWithText/treat_as_char/wrap/크기 보존. 전체 스위트
   (hwp5_roundtrip_baseline 포함).

## 비수정 범위

- 게이트 유입 분류(.hwp 명명 HWPX): #1914 FORMAT_SKIP (PR #1922)
- HWPX 직렬화의 표 flowWithText: #1637 에서 기수정 (`hwpx/table.rs`)
