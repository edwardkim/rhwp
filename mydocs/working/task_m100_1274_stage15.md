# Task 1274 Stage 15

## 대상

- 이전 로컬 task 번호 정정
- GitHub Issue 등록: https://github.com/edwardkim/rhwp/issues/1274

## 배경

- `#1265`는 이미 닫힌 PR 번호다.
- 현재 작업은 GitHub issue 없이 로컬 task 번호로 진행되어 PR 번호와 충돌했다.
- 작업지시자 요청에 따라 현재까지 발견된 교육 통합 HWP/PDF 시각 정합 문제를 GitHub Issue로 등록하고, 해당 이슈 번호 `1274`를 새 task 번호로 사용한다.

## 변경

- 계획서와 단계 문서 파일명을 `task_m100_1274*` 기준으로 변경했다.
- sweep 스크립트명을 `scripts/task1274_visual_sweep.py` 기준으로 변경했다.
- 문서 본문의 task 번호, 출력 폴더, 로컬 브랜치 표기를 `1274` 기준으로 변경했다.
- 회귀 테스트 함수명을 `issue_1274_*` 기준으로 변경했다.
- 오늘할일 문서의 임의 로컬 task 설명을 GitHub Issue #1274 설명으로 변경했다.

## 제외

- 실제 숫자 데이터인 폰트 메트릭 값 `1265`는 변경하지 않는다.
- 기존 외부 PR 검토 문서 `mydocs/pr/pr_1265_*`는 PR #1265 기록이므로 변경하지 않는다.

## 검증 계획

- `rg`로 current task 산출물에 남은 `1265` 표기가 없는지 확인한다.
- `cargo fmt --check`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo build --bin rhwp`
- `python3 scripts/task1274_visual_sweep.py`

## 검증 결과

- current task 산출물의 이전 task 번호 표기는 제거했고, 남은 `1265`는 PR 번호/숫자 데이터 예외로만 확인했다.
- `cargo fmt --check`: 통과.
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 48개 통과.
- `cargo build --bin rhwp`: 통과.
- `python3 scripts/task1274_visual_sweep.py`:
  - `2022-09`: SVG/PDF/compare 23/23/23, overflow 0건.
  - `2023-09`: SVG/PDF/compare 20/20/20, overflow 0건.
  - `2024-09-below20`: SVG/PDF/compare 23/23/23, overflow 0건.
  - `2024-09-between20`: SVG/PDF/compare 24/24/24, overflow 0건.
  - `2022-10`: SVG/PDF/compare 18/18/18, overflow 0건.
  - `2022-11-practice`: SVG/PDF/compare 21/21/21, overflow 0건.
