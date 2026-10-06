# Task #1564 Stage 2 완료보고서 — 스냅샷 골든 + 회귀 테스트

## 변경
- `tests/fixtures/opengov_snapshot.tsv` — 골든(id/status/ir_diff_count/class) 8행.
- `tests/opengov_corpus_snapshot.rs`:
  - 말뭉치 각 파일 parse→serialize→reparse(diff_documents)로 현재 status/diff 산출.
  - 심각도 키 `severity(status,diff)` = (tier, diff). tier: PASS0<IR_DIFF1<REPARSE2<SERIALIZE3<PARSE4.
  - **악화**(current>golden) → `regressions` 실패. **개선**(current<golden) → `improvements`
    실패(스냅샷 갱신/승격 강제). 일치 → OK.
  - `opengov_corpus_matches_snapshot`: 회귀·개선 모두 없어야 통과.
  - `opengov_snapshot_and_corpus_consistent`: 스냅샷↔파일 id 집합 일치 가드.

## 검증
- `cargo test --test opengov_corpus_snapshot`: **2 passed**.
- 현재 코드(HEAD)에서 골든(PASS 3 / IR_DIFF 5)과 일치.

## 다음
Stage 3 — 매뉴얼 + #1560 한글 오라클 연동(secCnt OK 회귀가드) + 최종 보고.
