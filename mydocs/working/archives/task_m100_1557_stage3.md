# Task #1557 Stage 3 완료보고서 — 회귀 가드 + 광역 검증

## 변경
- `src/serializer/hwpx/header.rs` 테스트 추가: `write_header_seccnt_matches_section_count`
  — 섹션 3개·`doc_properties.section_count=1`(stale 모사)에서 `secCnt="3"` 방출 단언.
  한글 미접근 환경에서도 회귀 감지.

## 검증
- `cargo test --lib write_header_seccnt`: **1 passed** (가드).
- `cargo test --test hwpx_roundtrip_baseline`: **4 passed** (samples/hwpx 전건 — 회귀 없음).

## 광역 재측정 (hwpdocs 실문서, fixed 바이너리)
동일 T3 표본 40건 한글 페이지 붕괴율:

| | 붕괴 |
|---|---|
| 수정 전 | **4/40 (10%)** |
| 수정 후 | **2/40 (5%)** |

- **secCnt 수정으로 복구**(다중구역): 36382669 **8→8**, 36388145 **5→5**. 완전 복원.
- 36384160(29쪽) 1→3 개선(3구역 로드).
- **잔여 붕괴**(secCnt 무관, 별도 원인): 36388284 **2→1**, 36388429 **2→1**
  — 단일구역 2쪽 문서의 1쪽 손실. 본 타스크 범위 외 → 후속 이슈 후보.

> IR_DIFF 건수(892/1398)는 secCnt 수정과 무관(XML 메타라 IR 비교 대상 아님) — 변동 없음.
> pic 드롭(V2-B)·잔여 2→1 붕괴는 별도.

## 결론
HWPX 저장 시 **다중 구역 문서의 한글 페이지 붕괴(secCnt 불일치)를 해소**. 회귀를 코드
레벨(단위 테스트)로 봉인. 표본 붕괴율 10%→5%(다중구역 케이스 전부 복구).
