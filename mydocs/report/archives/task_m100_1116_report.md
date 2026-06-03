# Task #1116 최종 보고서 — sample16 한컴 3mm 격자 정합 후속

- 이슈: [edwardkim/rhwp#1116](https://github.com/edwardkim/rhwp/issues/1116)
- 브랜치: `local/task1116`
- 기준 브랜치: `upstream/devel`
- 기준 커밋: `dd4bbfed`
- 작성일: 2026-05-25
- PR: [edwardkim/rhwp#1120](https://github.com/edwardkim/rhwp/pull/1120)
- 이전 PR: [edwardkim/rhwp#1119](https://github.com/edwardkim/rhwp/pull/1119) 닫힘

## 1. 작업 범위

`hwp3-sample16-hwp5*` 계열 문서에서 한컴오피스 3mm 격자 정답지와 rhwp 출력 사이에 남아 있던 목차, 본문 세로 배치, 표 분할, 영문 폰트 차이를 후속 정리했다.

주요 대상 파일:

- `samples/hwp3-sample16.hwp`
- `samples/hwp3-sample16-hwp5.hwp`
- `samples/hwp3-sample16-hwp5-2010.hwp`
- `samples/hwp3-sample16-hwp5-2018.hwp`
- `samples/hwp3-sample16-hwp5-2022.hwp`
- `samples/hwp3-sample16-hwp5-2024.hwp`
- `samples/k-water-rfp-2024.hwp`

## 2. 구현 요약

1. sample16 p2 목차 right-tab/leader 계열 배치를 보정하고 회귀 테스트를 추가했다.
2. HWP3-origin sample16 p3 본문 흐름에서 문단앞 간격과 `LINE_SEG` 기준 세로 진행을 3mm 격자 기준으로 재조정했다.
3. `hwp3-sample16-hwp5-2022.hwp`의 BCP 꼬리 줄 처리 차이를 좁은 조건으로 보정했다.
4. `k-water-rfp-2024.hwp` p5 RowBreak 표에서 마지막 단위 문단이 고립되어 과다 표시되는 절단 위치를 정정했다.
5. HWP3/HWP5 변환본의 legacy Latin face(`HCI Poppy` 등)를 한컴 HFT 치환과 같은 방식으로 해석해 p3 영문 폭과 모양 차이를 줄였다.
6. PR #1120 CI에서 발견된 `spacing_before` 사전 차감 회귀를 수정했다. 일반 문서 경로는 기존 #643/#1027처럼 `curr_sb`를 사전 차감하고, HWP3-origin 흐름에서만 #1116 예외로 사전 차감을 생략한다.
7. #1116의 SVG/browser 폭 고정 출력 변경에 맞춰 SVG golden 스냅샷을 갱신했다.

## 3. 변경 파일

주요 소스:

- `src/renderer/layout/paragraph_layout.rs`
- `src/renderer/layout/table_layout.rs`
- `src/renderer/layout/text_measurement.rs`
- `src/renderer/style_resolver.rs`
- `src/renderer/svg.rs`
- `src/renderer/web_canvas.rs`
- `src/renderer/skia/text_replay.rs`
- `src/main.rs`

테스트:

- `tests/issue_1116.rs`
- `tests/issue_1105.rs`

문서:

- `mydocs/plans/task_m100_1116.md`
- `mydocs/plans/task_m100_1116_impl.md`
- `mydocs/working/task_m100_1116_stage1.md` ~ `mydocs/working/task_m100_1116_stage20.md`
- `mydocs/working/task_m100_1116_stage21.md`
- `mydocs/working/task_m100_1116_stage22.md`
- `mydocs/orders/20260525.md`
- `mydocs/orders/20260526.md`
- `mydocs/manual/codex/docs_and_git_workflow.md`
- `mydocs/manual/memory/feedback_pr_requires_explicit_approval.md`

## 4. 검증 결과

통과한 명령:

```bash
cargo test --test issue_1116 -- --nocapture
cargo test --test issue_1105 -- --nocapture
cargo test --test issue_1086 -- --nocapture
cargo test --test issue_1035_alignment -- --nocapture
cargo test --test issue_713 -- --nocapture
cargo test --test svg_snapshot -- --nocapture
cargo fmt --all -- --check
cargo build --bin rhwp
git diff --check
```

PR #1120 CI 실패 후 추가 검증:

```bash
cargo test page_path_sb_prededuct --lib -- --nocapture
cargo test hwp3_origin_page_path_keeps_spacing_before_in_vpos --lib -- --nocapture
cargo test --lib
UPDATE_GOLDEN=1 cargo test --test svg_snapshot -- --nocapture
```

추가 결과:

- `cargo test --test svg_snapshot -- --nocapture`: 8 passed, 0 failed.

시각 검증용 산출물:

- `output/poc/render-spacing/stage20-font-diff-after/`

Stage 20 확인 결과:

```text
hwp3-sample16: C Palatino=6 HCI=0
hwp3-sample16-hwp5-2010: C Palatino=6 HCI=0
hwp3-sample16-hwp5-2018: C Palatino=6 HCI=0
hwp3-sample16-hwp5-2022: C Palatino=6 HCI=0
hwp3-sample16-hwp5-2024: C Palatino=6 HCI=0
```

## 5. PR 준비 상태

- PR #1119는 작업지시자 지시에 따라 닫았다.
- 작업지시자 승인 후 PR #1120을 생성했다.
- `local/task1116` 브랜치는 `origin/local/task1116`에 푸시되어 있다.
- PR 본문 초안은 `mydocs/report/task_m100_1116_pr_body.md`에 별도로 정리했다.
- PR #1120은 `jangster77:local/task1116` → `devel` 대상으로 열린 Open PR이다.
- PR #1120 CI에서 발견된 `spacing_before` 회귀와 SVG golden 미갱신을 수정했다.

생성 명령:

```bash
gh pr create \
  -R edwardkim/rhwp \
  --base devel \
  --head jangster77:local/task1116 \
  --title "#1116 sample16 한컴 3mm 격자 정합 후속" \
  --body-file mydocs/report/task_m100_1116_pr_body.md
```

## 6. 남은 절차

- PR #1120 최신 CI 통과 여부를 확인한 뒤 리뷰/머지 절차를 진행한다.
- 이슈 #1116 close는 작업지시자 승인 후, 대상 브랜치 포함 여부를 확인한 뒤 수행한다.
