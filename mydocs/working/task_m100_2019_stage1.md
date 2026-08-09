# #2019 Stage1 완료 보고 — 진단 하네스 + 기준선

- 브랜치: `fix/2019-through-wrap-overlay-vpos`, 베이스 survey8 (git 36c3102f), **소스 미수정**.

## 산출

아래 기준선·표본 목록·캡처 하네스·오라클 메모는 Stage 1 당시 `output/poc/task2019/`에 만든 POC다.
[PR #4154](https://github.com/edwardkim/rhwp/pull/4154)에서 이 네 파일의 저장소 추적본을 제거했으며,
[삭제 전 Git 이력](https://github.com/edwardkim/rhwp/tree/5a4f26d0d0a4e2fc96f4b73510d2aecdad916722/output/poc/task2019)으로만
보존한다. 현행 장기 근거는 [최종·정정 보고서](../report/task_m100_2019_report.md),
[tracked 회귀 래칫](../../tests/issue_2019_floating_form_overpagination.rs)과
`samples/hwpx/issue2019_floating_form_74312.hwpx`다.

- `output/poc/task2019/nogo_sample.txt` — 당시 무회귀 표본 **80문서**:
  8차 서베이 `pipage.tsv` MATCH 랜덤 60 + MORE 클러스터(글상자/도형 밀집 과분할) 20 + 74312(중복 흡수).
- `output/poc/task2019/capture_pages.py` — 당시 dump-pages 페이지수 캡처 하네스(before/after 공용).
- `output/poc/task2019/baseline.tsv` — 80문서 **당시 미수정본 페이지수**.
- `output/poc/task2019/oracle_truth.txt` — 당시 기록한 74312 정답(한글2022 PDF): **18쪽**.
- `output/poc/task2019/74312_before_rhwp11_vs_hwp4.png` — 당시 비추적 결함 시각 증거
  (왼쪽 서식 조각 깨짐 vs 오른쪽 한글 정상 표). #4154 삭제 목록에는 포함되지 않았다.

## 기준선 핵심 수치

- **74312(결함 대상): rhwp 81쪽 vs 한글 18쪽** — 4.5배 과분할, 35쪽 near-empty, 서식 조각화.
- 무회귀 표본 80문서 전부 페이지수 캡처 성공(ERR 0). 페이지수 분포 1~145쪽.

## 다음 (Stage2)

`layout.rs:866-890 para_has_overlay_shape` 에 `TextWrap::Through` 추가 → 74312 페이지수 81→18(±1) + 서식 렌더 정상화 시각 확인.

**소스 수정은 Stage2에서 승인 후 진행.**
