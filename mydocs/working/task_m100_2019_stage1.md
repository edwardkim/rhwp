# #2019 Stage1 완료 보고 — 진단 하네스 + 기준선

- 브랜치: `fix/2019-through-wrap-overlay-vpos`, 베이스 survey8 (git 36c3102f), **소스 미수정**.

## 산출

- `output/poc/task2019/nogo_sample.txt` — 무회귀 표본 **80문서**:
  8차 서베이 `pipage.tsv` MATCH 랜덤 60 + MORE 클러스터(글상자/도형 밀집 과분할) 20 + 74312(중복 흡수).
- `output/poc/task2019/capture_pages.py` — dump-pages 페이지수 캡처 하네스(before/after 공용).
- `output/poc/task2019/baseline.tsv` — 80문서 **현재(미수정) 페이지수**.
- `output/poc/task2019/oracle_truth.txt` — 74312 정답(한글2022 PDF): **18쪽**.
- `output/poc/task2019/74312_before_rhwp11_vs_hwp4.png` — 결함 시각 증거(왼쪽 서식 조각 깨짐 vs 오른쪽 한글 정상 표).

## 기준선 핵심 수치

- **74312(결함 대상): rhwp 81쪽 vs 한글 18쪽** — 4.5배 과분할, 35쪽 near-empty, 서식 조각화.
- 무회귀 표본 80문서 전부 페이지수 캡처 성공(ERR 0). 페이지수 분포 1~145쪽.

## 다음 (Stage2)

`layout.rs:866-890 para_has_overlay_shape` 에 `TextWrap::Through` 추가 → 74312 페이지수 81→18(±1) + 서식 렌더 정상화 시각 확인.

**소스 수정은 Stage2에서 승인 후 진행.**
