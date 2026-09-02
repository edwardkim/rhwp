# [#4868] 전/후 증빙 — 하네스 성질 러너

`harness-proofs-before-after.png` 는 두 러너의 `--json` 출력에서 **직접 그린** 표다
(수치 하드코딩 없음).

| | 판정 | exit | 비고 |
|---|---|---|---|
| BEFORE (`upstream/devel` `627c8c49a`) | 5/6 | 1 | `[FAIL] P2 commands=85 (expected=68)` |
| AFTER (이 브랜치) | **8/8** | 0 | P7·P8 신규, P2 는 하한으로 |

- **P7** 본문 줄 425개 중 원시 디코딩(UTF-8·UTF-16LE) 어디에도 없는 줄 420개 = 98.8%
  (판정 임계 50%)
- **P8** 표적 줄 78자 · `matchCount=1` · `search page=15` vs `export-text page=15`

BEFORE 는 `git show upstream/devel:tools/harness_proofs.py` 를 그대로 실행한 결과다.

재현:

```bash
cargo build --bin rhwp
python tools/harness_proofs.py          # 표 출력, 하나라도 FAIL 이면 exit 1
python tools/harness_proofs.py --json   # 기계용
```
