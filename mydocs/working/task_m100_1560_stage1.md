# Task #1560 Stage 1 완료보고서 — 핵심 도구

## 변경
- 신규 `tools/verify_hangul_pages.py`:
  - `--batch <원본_폴더> <rt_폴더>`: 원본 재귀 스캔 → 상대경로 매칭으로 rt(`.rt.hwpx`/`.rt.hwp`) 탐색.
  - 각 쌍 한글 `PageCount` 비교 → `OK`/`COLLAPSE`/`EXPAND`/`ERR`.
  - 파일별 try/except 격리(한글 행/오류로 전체 중단 방지), `clear(option=1)` 정리.
  - TSV(`verdict/orig_pg/rt_pg/note/rel`) + 요약(붕괴율) + **COLLAPSE>0 시 종료 코드 1**(게이트).
  - pyhwpx 미설치 시 명확 안내 후 종료 코드 2(크래시 금지).
  - `tools/verify_hwpx.py` 컨벤션(argparse+pyhwpx+종료코드) 합류.

## 검증 (알려진 케이스 3건)
```
[1/3]       OK  pg 8->8   36382669   # #1557 secCnt 수정 케이스 — 정상 OK
[2/3] COLLAPSE  pg 29->3  36384160   # secCnt 부분복구 잔여 — 붕괴 검출
[3/3] COLLAPSE  pg 2->1   36387103   # 단일구역 잔여 — 붕괴 검출
종료 코드 1 (COLLAPSE 존재)
```
IR 게이트가 못 잡는 붕괴를 정확히 검출하고 게이트 종료코드 동작 확인.

## 다음
Stage 2 — `--inventory`/`--sample`/`--pdf`/`--status` + 재현성(git HEAD 기록).
