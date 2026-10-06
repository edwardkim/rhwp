# Task #1567 최종 결과보고서 — HWPX 표 셀 pic 드롭 해소

- 이슈: #1567 (M100)
- 브랜치: `local/task1567` (from devel)
- 일자: 2026-06-26

## 1. 문제
HWPX 저장 시 표 셀 내 `<hp:pic>` 드롭(roundtrip IR diff
`tbl.cell.p[0] controls: expected=[pic] actual=[]`). 실문서 최다 IR_DIFF 클래스(907건).

## 2. 근본원인
1. 원본 셀 pic 다수가 `binaryItemIDRef=""`(빈 ref, placeholder) → 파서 `bin_data_id=0`.
2. `write_img`(`picture.rs:256`) `resolve_bin_id(0)→None`→`Err`.
3. 호출자 `section.rs:701` 가 에러를 로그만 찍고 **pic 드롭**(IR_DIFF).

## 3. 수정
`write_img`: `resolve_bin_id` 실패 + `bin_id==0` 이면 `binaryItemIDRef=""` verbatim 방출
(드롭 금지, `"" → 0 → ""` roundtrip 보존). 비-0 미해결은 `Err` 유지(손실 은폐 방지).

## 4. 검증
- 단위 테스트 `task1567_empty_binary_ref_pic_preserved` passed.
- 36385464: IR_DIFF/1 → **PASS/0**(pic 드롭 해소).
- `hwpx_roundtrip_baseline` 4 passed(회귀 없음).
- **광역(hwpdocs 2601건): 표셀 pic 드롭 907→0, PASS율 35%→71%, IR_DIFF율 65%→28%.**

## 5. 잔여 / 후속
- 남은 IR_DIFF(737)는 char_shape 8유닛 시프트(F3 #1556, 별개).
- opengov 스냅샷(#1564/PR #1566) 갱신 교차 후속: #1564+#1567 머지 시 36385464(→PASS)·36388571(→d1) 승격.
- 일부 가시 이미지 손실(비-0 미해결)은 본 수정 범위 밖(진단 유지) — 별도 조사.

## 6. 변경 파일
- `src/serializer/hwpx/picture.rs` (write_img 빈 ref 보존 + 단위 테스트)
- 계획/보고: `mydocs/plans/task_m100_1567{,_impl}.md`, `mydocs/working/task_m100_1567_stage{1..3}.md`
