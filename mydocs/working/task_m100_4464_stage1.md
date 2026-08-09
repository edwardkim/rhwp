# task_m100_4464 stage1 — 에이전트 짐(운동장) 개장 + 1호 선수 실주행

- 이슈: #4464 / 브랜치: task_m100_4464 (base devel)
- 산출물: gym/ — README(에이전트 대면)·tasks 12종·score.py(라이브 오라클 채점기)·baselines/claude-fable-5(1호 기록)

## 실측

- 1호 선수(claude-fable-5) 12과제 전 수행 → **26/26 만점** (report.md 원문 커밋).
- 과제 오라클 경로는 실봉투로 검증 후 확정(pageCount·injectionSignals 등 — 추측 기재 없음).
- 1호 주행이 채점기 결함 1건을 실측으로 잡음: 상대경로 RHWP_BIN 을 Windows
  CreateProcess 가 자식 cwd 기준으로 풀지 않아 전 과제 WinError 2 → find_bin
  절대화로 수리(주석에 실측 기록). 첫 주행의 존재 이유 증명.
- 결정론 과제(T10)는 같은 계획 2회 실행의 바이트 동일을 해시로 판정 — 검증
  사다리의 토대를 운동장 과제로 상설화.

## 규약

- 산출물(.hwp/.hwpx)은 커밋하지 않음(gym/.gitignore) — 재실행으로 재생산 가능.
- 채점은 골든 박제 없이 라이브 재계산 — 픽스처 진화에 오라클이 따라감.
- 2부(캡슐 제출·audit/gate 채점)는 검증 사다리 머지 후 확장(#4463 좌표).
