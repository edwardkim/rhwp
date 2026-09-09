# Task #1564 최종 결과보고서 — 고정 실문서 회귀 말뭉치

- 이슈: #1564 (M100)
- 브랜치: `local/task1564` (from devel)
- 일자: 2026-06-26

## 1. 목표 및 결과
hwpdocs(수집기로 건수가 변하는 비재현 대상) 대신, opengov 정보공개 결재문서 **클래스별
대표 8건**을 `samples/hwpx/opengov/` 로 동결하고 **재현 가능한 충실도 회귀 게이트**를 신설.

## 2. 말뭉치 (8건, 1.7MB)
| 클래스 | 파일 | IR status | 한글 verdict |
|--------|------|-----------|--------------|
| PASS 클린 | 36389298, 36384285 | PASS/0 | OK |
| 다중구역/secCnt 회귀가드(#1557) | 36382669 | PASS/0 | **OK 8→8** |
| 표셀 pic 드롭(V2-B) | 36388571, 36385464 | IR_DIFF/2,1 | OK |
| char_shape 시프트(F3 #1556) | 36383351, 36388853 | IR_DIFF/1 | 36383351 COLLAPSE |
| 잔여 2→1 붕괴 | 36387103 | IR_DIFF/1 | COLLAPSE |

PII 방침 **A(그대로 동결, 이미 공개 정보)** — 승인 완료.

## 3. 두 갈래 게이트
- **IR 스냅샷**(`tests/opengov_corpus_snapshot.rs` + `tests/fixtures/opengov_snapshot.tsv`):
  악화→실패(회귀), 개선→실패(승격 강제). Linux CI 가능. `cargo test`: **2 passed**.
- **한글 페이지 오라클**(#1560 `tools/verify_hangul_pages.py`): 동결 말뭉치 한글 verdict
  기록 — **36382669 OK(8→8)로 #1557 secCnt 회귀 가드 실증**. (OK 6/COLLAPSE 2)
- `hwpx_roundtrip_baseline` 은 opengov 하위 제외(자체 스냅샷). baseline **4 passed**(회귀 없음).

## 4. 가치
실문서 충실도 측정을 **재현 가능·게이트 가능**하게 고정. 결함 수정(개선)은 스냅샷 승격으로
명시 추적, 회귀(악화)는 즉시 실패. #1557 secCnt 회귀를 한글 오라클로 상시 감시.

## 5. 변경 파일
- 신규: `samples/hwpx/opengov/`(8 hwpx + README), `tests/fixtures/opengov_snapshot.tsv`,
  `tests/opengov_corpus_snapshot.rs`, `mydocs/manual/opengov_corpus.md`
- 수정: `tests/hwpx_roundtrip_baseline.rs`(opengov 제외)
- 계획/보고: `mydocs/plans/task_m100_1564{,_impl}.md`, `mydocs/working/task_m100_1564_stage{1..3}.md`

## 6. 한계 / 후속
- IR 스냅샷은 구조 회귀, 한글 verdict 는 #1560(로컬 한글). 시각 픽셀 diff(T4)·pic 시각
  triage(측정도구 3순위)는 별도.
- 잔여 2→1 한글 붕괴(36383351·36387103)는 secCnt 무관 별도 결함 — 후속 이슈 후보.
