# Task #1564 Stage 1 완료보고서 — 대표 동결 + README

## 변경
- `samples/hwpx/opengov/` 신설 — opengov 정보공개 결재문서 대표 **8건**(1.7MB) 동결.
  - 12MB 대형(36384160)은 제외(다중구역 클래스는 36382669로 커버, repo 비대화 회피).
- `samples/hwpx/opengov/README.md` — 출처·PII 방침 A·클래스 매핑·갱신 절차.
- `tests/hwpx_roundtrip_baseline.rs`: collect_samples 가 `opengov` 하위를 **제외**
  (자체 스냅샷 게이트로 검증, diff=0 강제 baseline 대상 아님).

## 동결 말뭉치 (status 골든)
| 파일 | 클래스 | status/diff |
|------|--------|-------------|
| 36389298 | PASS 클린 | PASS/0 |
| 36384285 | PASS 클린 | PASS/0 |
| 36382669 | 다중구역/secCnt 회귀가드(#1557) | PASS/0 (한글 8쪽) |
| 36388571 | 표셀 pic 드롭(V2-B) | IR_DIFF/2 |
| 36385464 | 표셀 pic 드롭(V2-B) | IR_DIFF/1 |
| 36383351 | char_shape 시프트(F3 #1556) | IR_DIFF/1 |
| 36388853 | char_shape 시프트(F3 #1556) | IR_DIFF/1 |
| 36387103 | 잔여 2→1 한글 붕괴 | IR_DIFF/1 |

분포: PASS 3 / IR_DIFF 5.

## 검증
- `rhwp hwpx-roundtrip --batch samples/hwpx/opengov` → 위 status 확인.
- `hwpx_roundtrip_baseline`: opengov 제외로 기존 전건 PASS 유지(회귀 없음).

## 다음
Stage 2 — 골든 스냅샷(`tests/fixtures/opengov_snapshot.tsv`) + 회귀 테스트(`tests/opengov_corpus_snapshot.rs`).
