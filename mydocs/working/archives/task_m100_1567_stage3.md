# Task #1567 Stage 3 완료보고서 — 회귀 + 측정 + 보고

## 회귀
- `cargo test --test hwpx_roundtrip_baseline`: **4 passed**(samples/hwpx 전건, 회귀 없음).
- 단위 테스트 `task1567_empty_binary_ref_pic_preserved`: passed.

## 광역 측정 (hwpdocs 2601건, 수정 후)
| | v4(미수정) | 수정 후 |
|--|--|--|
| 표셀 pic 드롭 | 907 | **0** ✅ |
| PASS율 | 35% | **71%** |
| IR_DIFF율 | 65% | **28%**(전부 char-shift F3 #1556) |

pic 드롭 클래스 완전 해소. 남은 IR_DIFF 737은 char_shape 8유닛 시프트(F3, 별개).

## opengov 스냅샷 (교차 후속)
opengov 고정 말뭉치/스냅샷은 #1564 브랜치(PR #1566)에 있어 본 브랜치(devel 기반)엔 없음.
**#1564 + #1567 머지 시** 다음 개선으로 스냅샷 갱신(승격) 필요:
- 36385464: IR_DIFF/1 → **PASS/0**
- 36388571: IR_DIFF/2 → **IR_DIFF/1**
→ `tests/fixtures/opengov_snapshot.tsv` 해당 행 갱신.

## 결론
빈 binaryItemIDRef pic 보존으로 표 셀 pic 드롭(실문서 최다 IR_DIFF, 907건) 완전 해소.
PASS율 35%→71%. baseline 회귀 없음.
