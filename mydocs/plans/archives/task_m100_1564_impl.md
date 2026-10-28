# Task #1564: 고정 실문서 회귀 말뭉치 — 구현 계획서

> 수행계획서 `task_m100_1564.md` 승인 + **PII 방침 A(그대로 동결, 이미 공개 정보)** 확정.
> 3단계. 대상: 서울 정보소통광장 정보공개 결재문서.

## 동결 대상 (클래스 커버, ~10건)
| 클래스 | 파일(36xxxxxx) |
|--------|----------------|
| PASS 클린 | 36389298, 36384285 |
| 표 셀 pic 드롭(V2-B) | 36388571, 36385464 |
| char_shape 8유닛 시프트(F3 #1556) | 36383351, 36388853 |
| 다중구역/secCnt 회귀 가드(#1557) | 36382669(8쪽), 36384160(29쪽) |
| 잔여 단일구역 2→1 붕괴 | 36387103 |

## Stage 1 — 대표 동결 + 출처 README
- hwpdocs 에서 위 파일을 `samples/hwpx/opengov/` 로 복사(git-tracked, 평탄 배치).
- `samples/hwpx/opengov/README.md`: 출처(opengov.seoul.go.kr 정보공개), 수집일, 클래스 매핑, 갱신 절차.
- `rhwp hwpx-roundtrip --batch samples/hwpx/opengov` 로 status 확인.
- 산출: `task_m100_1564_stage1.md` + 커밋(말뭉치 동결).

## Stage 2 — 스냅샷 골든 + 회귀 테스트
- 골든 스냅샷 `tests/fixtures/opengov_snapshot.tsv`: 파일별 기대 `status`/`ir_diff_count`.
- `tests/opengov_corpus_snapshot.rs`:
  - 각 파일 parse→serialize→reparse(diff_documents)로 현재 status/diff 산출.
  - 골든과 비교 — **악화**(PASS→IR_DIFF, diff 증가, REPARSE_FAIL 등) → 실패(회귀).
  - **개선**(IR_DIFF→PASS, diff 감소) → 실패 + "스냅샷 갱신 필요" 안내(승격 강제).
  - HWP3/배포용 자동 제외(없을 예정이나 가드).
- 산출: `task_m100_1564_stage2.md` + 커밋.

## Stage 3 — 매뉴얼 + #1560 연동 + 최종 보고
- `mydocs/manual/opengov_corpus.md`: 말뭉치 구성·스냅샷 갱신·#1560 한글 오라클 연동.
- #1560 도구로 동결 말뭉치 한글 verdict 기록(secCnt 케이스 OK 회귀 가드).
- `mydocs/report/task_m100_1564_report.md` + 커밋.

## 주의
- PII 방침 A — 이미 공개 정보로 그대로 동결(승인 완료).
- 대표 소수(~10건, 각 수십~수백 KB)로 repo 비대화 회피.
- 스냅샷은 IR 구조 기준 회귀 게이트(Linux CI 가능). 한글 페이지 verdict 는 #1560(로컬 오라클).
- rhwp 소스 무변경(말뭉치+테스트+문서).
