---
kind: investigation
status: in_progress
canonical: mydocs/working/task_m100_3820_stage1.md
last_verified: 2026-08-06
---

# Task #3820 Stage 22 — issue2007 전체 PDF fidelity 종결 재조사

## 범위

Stage 20에서 p10–p17의 continuation 경계를, Stage 21에서 p2·p4 표 결함을 각각
재판정했다. 이 단계는 같은 HWP/PDF 쌍 17쪽 전체를 대상으로 다음을 함께 확인한다.

- `fidelity_compare --text-only --layout-ledger`의 페이지 수·text owner·표/각주·border 후보
- 아직 직접 감사하지 않은 p1·p3·p5–p9의 PDF 1:1 raster 쌍
- 앞 단계의 p2·p4·p10–p17 판정과 모순되는 새 page owner 또는 표 frame 결함

기준 PDF 및 입력 경로는 Stage 20과 같다. 기계 원장은 후보 선별용이며 최종 시각 판정은
각 페이지 PNG와 함께 기록한다.

## p9 완료 다행 표 뒤 제목 clip — 분석·보정·증적 (2026-08-06)

### 재현과 원인

PDF 물리 p9의 `<국내 유사입법례 분석>`은 rhwp SVG에서 ancestor `Cell` clip 상단보다
위(`y=100.5`)에 배치돼 실제 Canvas/SVG에 나타나지 않았다. 원인은 1×1 중첩 표
continuation이 이전 viewport의 첫 가시 unit을 이미 소비한 잔상으로 간주해, 다음
viewport에서도 같은 reservation을 적용한 데 있었다.

그러나 이 입력의 경계는 `p72`의 완료된 7×3 표 → 빈 `p73` spacer → `p74` 제목이다.
제목은 이전 페이지 잔상이 아니라 새 물리 block이므로, 높이만으로 판단하지 않고 이
source 경계를 fragment metadata로 전달해야 한다.

### 보정 범위

`NestedTableFragment::starts_after_table`을 추가해 완료된 **다행** 중첩 표 뒤의 첫
실제 문단만 식별한다. 빈 spacer는 표식 전달을 끊지 않으며, 그 fragment에는 기존의
`compensate_first_visible` reservation을 적용하지 않는다. 일반 1×1 continuation과
terminal tail은 기존 reservation을 보존한다.

### 검증

- focused: `CARGO_TARGET_DIR=target/task-3820-3821-fidelity CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2007_nested_cell_pagination`
  - 결과: 9 passed, 0 failed. 새 회귀는 p9 제목이 실제 root clip 안 `y=125..140`에
    그려지는지를 검사하며 기존 p12 잔상 방지와 종료 조각 회귀도 함께 통과했다.
- render tree: PDF p9 ↔ `export-render-tree --page 8`(0-based)에서 제목은
  `pi=74, y=132.5`로 새 viewport 안에 있다. p12의 이전 조각 문장은 `y=87.6`으로
  root clip 밖에 남아 기존 경계 보정이 유지된다.
- visual sweep: `--page 9`(1-based)로 기준 PDF와 현재 release-test SVG를 비교했다.
  `visual_accuracy_proxy_percent=10.51371`은 표 내부 glyph·글꼴·줄바꿈까지 포함한
  전체 ink 보조값이므로 p9 제목 clip 해결의 합격 지표로 사용하지 않는다. review에서
  PDF와 rhwp 양쪽의 제목 표시를 직접 확인했다.

### 증적

- [compare](../pr/assets/task_m100_3820_stage22_issue2007_full_pdf_fidelity/compare_p009_after.png)
- [overlay](../pr/assets/task_m100_3820_stage22_issue2007_full_pdf_fidelity/overlay_p009_after.png)
- [review](../pr/assets/task_m100_3820_stage22_issue2007_full_pdf_fidelity/review_p009_after.png)
- [overlay metrics](../pr/assets/task_m100_3820_stage22_issue2007_full_pdf_fidelity/overlay_metrics_p009_after.json)

이 보정은 p9 제목 누락에 한정한다. 전체 17쪽의 PDF fidelity 완료를 주장하지 않으며,
Stage 22의 나머지 페이지별 raster 대조와 p13 owner drift 조사는 계속한다.
