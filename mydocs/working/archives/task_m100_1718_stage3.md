# Task #1718 Stage 3 — 단위테스트 + 검증

## 신규 단위테스트 (table_layout.rs row_cut_tests)
- `test_advance_row_cut_rowbreak_grace_denied_when_visible_after_spacer` — 오버플로 가시라인 뒤에
  spacer + 가시라인이 있으면 grace 거부(end_cut=[3], 본문 미초과). ✅
- `test_advance_row_cut_rowbreak_grace_kept_for_true_tail_before_spacers` — 뒤가 전부 spacer 면
  grace 유지(end_cut[0]>=4). ✅

## dump-pages 검증 (수정 후)
| 파일 | 수정 전 | 수정 후 | 한글 | 판정 |
|------|--------|--------|------|------|
| 승강기 [별표27] | 40 | **42** | 48 | 개선(over-fill 해소분) |
| byeolpyo1 | 4 | 4 | 4 | 무회귀 ✅ |
| byeolpyo4 | 26 | 26 | 27 | 무회귀 ✅ |

- 수정 후 컷 진행: 텍스트 페이지 34~38줄(수정 전 39~42) → 한글(35~38) 정합.

## cargo test
- `cargo test --lib`: 2042 passed / 0 failed (기존) + 신규 2 passed = 무회귀.
