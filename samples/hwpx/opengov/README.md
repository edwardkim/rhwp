# opengov 고정 실문서 회귀 말뭉치 (Task #1564)

서울 정보소통광장(`opengov.seoul.go.kr`) **정보공개 결재문서** 중 무손실 검증 클래스별
대표를 동결한 회귀 기준선. hwpdocs(수집기로 건수가 변하는 비재현 대상) 대신 **고정·재현
가능**한 충실도 회귀 추적용.

- 출처: opengov.seoul.go.kr 정보공개(공개 문서), 수집일 2026-06-26
- PII 방침: **A(그대로 동결)** — 이미 공개된 정보공개 문서 (Task #1564 승인)
- 회귀 게이트: `tests/opengov_corpus_snapshot.rs` (스냅샷, diff=0 강제 아님)
- 한글 페이지 verdict: `tools/verify_hangul_pages.py`(#1560, 로컬 오라클)

## 클래스 매핑 (기대 status, 현재 코드 HEAD 기준)
| 파일(36xxxxxx) | 클래스 | 기대 status / diff |
|----------------|--------|--------------------|
| 36389298 | PASS 클린 | PASS / 0 |
| 36384285 | PASS 클린 | PASS / 0 |
| 36382669 | 다중구역/secCnt 회귀 가드(#1557) | PASS / 0 (한글 8쪽 보존) |
| 36388571 | 표 셀 pic 드롭(V2-B) | IR_DIFF / 2 |
| 36385464 | 표 셀 pic 드롭(V2-B) | IR_DIFF / 1 |
| 36383351 | char_shape 8유닛 시프트(F3 #1556) | IR_DIFF / 1 |
| 36388853 | char_shape 8유닛 시프트(F3 #1556) | IR_DIFF / 1 |
| 36387103 | 잔여 단일구역 2→1 한글 붕괴 | IR_DIFF / 1 |

## 갱신 절차
- 결함 수정으로 status **개선**(IR_DIFF→PASS, diff 감소) 시 `tests/opengov_corpus_snapshot.rs`
  가 실패 → `tests/fixtures/opengov_snapshot.tsv` 갱신(승격).
- **악화**(PASS→IR_DIFF, diff 증가, REPARSE_FAIL) 시 실패 = 회귀 → 결함 조사.
- 신규 클래스 추가 시 파일 동결 + 스냅샷 행 추가 + 본 표 갱신.

> 주의: 스냅샷은 IR 구조 회귀(Linux CI 가능). 한글 전용 페이지 붕괴는 #1560 도구로 별도 검증.
