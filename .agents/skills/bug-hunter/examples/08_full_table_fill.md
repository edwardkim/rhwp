# 예제 — 실물 표 양식 전 항목 채움

이슈 #5324. playbook 카탈로그 "46칸 규모". gym 아님. F13.

## 정답지

칸마다 가상 값 + `export-tables --json` 재독 100%.
부분 채움으로 "된다"고 하지 않는다 (F05).

## 명령

```bash
rhwp export-tables --json 양식.hwp > before.json
# 칸 목록을 UTF-8 파일로 남긴다. 콘솔에 한글 리터럴을 넣지 않는다
# 각 칸: rhwp edit set-cell … -o 누적본.hwp --json
rhwp export-tables --json 최종.hwp > after.json
venv/bin/python compare_cells.py --utf8 before.json after.json expect.json
```

`compare_cells.py` 는 여정 로컬 스크립트여도 된다. `rhwp` 하위명령으로
추가하지 않는다.

## 읽는 법

한 칸 침묵 유실은 #3358 계열. 전 칸 재독 표를 이슈에 붙인다.
N중 M 없이 "표 채움이 깨진다"고 쓰지 않는다 (F15).
