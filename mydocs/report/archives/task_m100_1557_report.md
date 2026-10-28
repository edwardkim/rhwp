# Task #1557 최종 결과보고서 — HWPX 저장본 한글 페이지 붕괴 해소 (secCnt)

- 이슈: #1557 (M100)
- 브랜치: `local/task1557` (from devel)
- 일자: 2026-06-26

## 1. 문제
`serialize_hwpx` 저장본을 한글 2024 가 열면 다중 페이지 문서가 **1쪽으로 붕괴**.
rhwp 자신은 원본·저장본을 동일 페이지수로 보고(자기 일관), **IR diff 게이트도
rhwp 페이지수도 검출 불가** — 한글 오라클 전용. PASS(IR diff=0) 문서도 붕괴.
fidelity v2(실문서 1135건) 표본 40건 중 **4건(10%)**.

## 2. 근본원인 (Stage 1)
`src/serializer/hwpx/header.rs:37` 가 `secCnt` 를 stale 가능한
`doc.doc_properties.section_count`(=1)에서 가져옴. 섹션 **파일**은 `doc.sections`
(=3) 기준으로 방출되어 **`secCnt`(1) < 실제 섹션 수(3)** 불일치 → 한글이 구역 1·2 를
로드하지 않고 1쪽으로 붕괴.

격리 실험으로 확정: 저장본의 `secCnt="1"`→`"3"` 치환만으로 36382669 가 한글 8→8 완전 복원.

## 3. 수정 (Stage 2)
```rust
- let sec_cnt = doc.doc_properties.section_count.max(1).to_string();
+ let sec_cnt = doc.sections.len().max(1).to_string();
```
`secCnt` 를 실제 직렬화 섹션 수와 항상 일치. IR 의미 변경 없음(메타데이터 교정).

## 4. 검증 (Stage 2·3)
- 36382669: 한글 **8→8 완전 복원**(IR diff=0 불변), 36388145 **5→5 복원**, 36384160 1→3 개선.
- 회귀 가드 단위 테스트(`write_header_seccnt_matches_section_count`) 통과.
- `hwpx_roundtrip_baseline` 4 passed(samples/hwpx 전건 회귀 없음).
- 광역: 표본 붕괴율 **10%(4/40) → 5%(2/40)** — 다중구역 붕괴 전부 복구.

## 5. 잔여 / 후속
- 잔여 붕괴(secCnt 무관): 36388284·36388429 **2→1**(단일구역 2쪽 → 1쪽 손실). 별도 원인 → 후속 이슈 후보.
- 표 셀 pic 드롭(V2-B, 545건), char_shape 8유닛 시프트(F3 #1556), header.xml 기타 차이
  (imgBrush/strikeout/shadow/tabDef switch)는 본 타스크 범위 외.

## 6. 변경 파일
- `src/serializer/hwpx/header.rs` (secCnt 산출 1행 + 회귀 가드 테스트)
- 계획/보고: `mydocs/plans/task_m100_1557{,_impl}.md`, `mydocs/working/task_m100_1557_stage{1..3}.md`

## 7. 한계
- 한글 의존 검증(자동화 한계)은 코드 레벨 가드로 보완.
- 게이트(`hwpx-roundtrip`)는 여전히 페이지 붕괴를 직접 검출 못 함(IR 비교라) — secCnt
  불일치는 코드 가드로 봉인했으나, 일반 한글 페이지 오라클 연동은 후속 개선 후보.
