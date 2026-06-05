# PR #1222 처리 보고서 — HWPX 본문·표 셀 문단 id 전역 유니크 (PR #1206 분리 재제출)

- **작성일**: 2026-06-01
- **PR**: #1222 → **MERGED** (devel, 로컬 `--no-ff` 머지 + 충돌 해소)
- **컨트리뷰터**: @Mireutale (#1206 분리 재제출 — 메인테이너 안내 반영)
- **연결 이슈**: #1134 → **CLOSED** (`closes #1134`)
- **판단**: **머지** ✅

## 결정 사유

#1206 close 시 요청한 "문단 id 충돌 수정만 분리 재제출"을 정확히 이행(fillBrush 제외 확인).
id 충돌 수정은 #1206 검토에서 이미 "적절·가치 있음"으로 평가. 전역 카운터 설계 깔끔, 통합
테스트로 실효 검증, 회귀 0(1921 passed), clippy clean. 시각 무관(XML id).

## 변경 요약 (4 파일, +159/−7)

| 파일 | 변경 |
|------|------|
| `serializer/hwpx/context.rs` | `para_id_counter` + `next_para_id()` 전역 단조 증가 카운터 |
| `serializer/hwpx/section.rs` | 본문 문단·각주 sublist 가 next_para_id() 사용(루프 인덱스 제거) |
| `serializer/hwpx/table.rs` | 셀 문단이 같은 ctx 카운터 공유 |
| `tests/hwpx_roundtrip_integration.rs` | basic-table-01 라운드트립 본문+셀 id 전역 유니크 |

## #1206 close 3사유 처리

| 사유 | 처리 |
|------|------|
| ② fillBrush 무관 변경 혼재 | ✅ 해소 — header.rs 제외(파일 4개) |
| ③ image_fill_mode 라운드트립 비대칭 | ✅ 해소 — fillBrush 자체 제외 |
| ① 미완성 HWPX 쓰기 모듈 | id 수정은 #1206 검토 "적절" 평가 + 분리 요청 합의의 이행. id 유니크는 cell_split 손상 방지 방향 |

## 충돌 해소 (CONFLICTING)

`table.rs` tests 모듈 단일 충돌 — 방금 머지한 #1213(textFlow) 테스트 4건과 #1222 id 테스트 4건이
같은 라인 영역에 추가된 텍스트 인접 충돌(본문 write_table 자동 머지). 양쪽 테스트 전부 보존.

## 검증 결과

| 항목 | 결과 |
|------|------|
| merge `--no-ff` | ✅ (텍스트 인접 충돌 해소, 마커 0) |
| fmt / build / clippy(lib) | ✅ CLEAN |
| 전체 테스트 `cargo test --tests` | ✅ **1921 passed, 0 failed** |
| id 충돌 단위 4건 | ✅ 셀 유니크/카운터 오프셋/연속 표 무충돌/셀당 복수 문단 |
| 통합 테스트 | ✅ para_ids_unique_across_body_and_table (basic-table-01 실제 parse→serialize_hwpx→XML id 유니크) |
| fillBrush 제외 | ✅ header.rs 미포함 |
| 시각 | 해당 없음 (XML id 변경, 렌더 무관) |

## 처리 절차

1. PR 정보 — CONFLICTING/DIRTY, CI 미실행, #1206 분리 재제출. #1206 report 정독.
2. CONFLICTING 원인(table.rs tests 텍스트 인접 충돌) + context/section/table diff 검토. fillBrush 제외 확인.
3. 로컬 `pr1222-verify`: 충돌 해소(테스트 8건 보존) / fmt/build/clippy(lib) / test 1921 / 통합 테스트.
4. 작업지시자 승인(첫 기여자 분리 재제출 감사 표명 요청).
5. devel `--no-ff` 머지(충돌 해소 포함) + push. 이슈 #1134 클로즈 + 감사 코멘트.

## 비고

- CI 미실행(serializer/hwpx 가 CI paths 밖) → 로컬 직접 검증.
- fillBrush 직렬화 + image_fill_mode 토큰 정합은 HWPX 쓰기 모듈 정비 시 별도(#1206 후속 (b)).
- @Mireutale 첫 기여 + 메인테이너 안내 따라 분리 재제출 — 정중한 감사 코멘트.
