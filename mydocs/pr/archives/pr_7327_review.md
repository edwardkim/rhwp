---
kind: snapshot
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-22
---

# PR #7327 검토 기록 — 저장 밴드 TAC 표의 중간 줄바꿈

- 원 PR: [#7327](https://github.com/edwardkim/rhwp/pull/7327)
- 관련 이슈: [#7312](https://github.com/edwardkim/rhwp/issues/7312)
- 원 code head: `5c09c4e1d292689417a4a1294377c3bdd3b2323d`
- 통합 검토 branch: `review/planet-open-20260922`
- 적용 commit: `f90312f3e` (`cherry-pick -x`, 충돌 없음)
- 메인터너 보정: `0fc33d759`

## 검토와 보정

원 변경은 저장 밴드가 표 외곽 상자와 정확히 일치할 때 중간앵커 TAC 표를 다음 줄로 내리지 않는다.
다만 판별 헬퍼가 문단 첫 line segment만 읽으면서 모든 inline 표에 적용될 수 있었다. 다중 segment 문단에서는
표의 소유 줄을 증명할 수 없으므로 메인터너 보정에서 **단일 segment 문단에만** 이 예외를 제한했다.
다중 segment는 종전 줄바꿈 경로를 유지한다.

## 기준 입력과 시각 검증

- 입력: `samples/issue7312/36494702_approval_tac_band.hwpx`, `lastSavedWith=hancom-office-2020 11.0.0.8227`.
- 기준 PDF: `pdf/issue7312/36494702_approval_tac_band-2020.pdf`, Hancom 2020 engine, 1쪽,
  SHA-256 `5df9d58da46140144a6f1a8e0d7b8f28cbcf8579e949ae7dea93109629419b19`.
- Native·fresh WASM Visual Sweep 1쪽: 구조 flag 0건. raster pixel match 92.310%, ink match 43.895%이며
  글꼴 raster 차이는 수치와 overlay에 남기고 정합으로 과장하지 않는다.
- overlay: `mydocs/pr/assets/pr7327_review/native_overlay_001.png`,
  `mydocs/pr/assets/pr7327_review/wasm_overlay_001.png`.

## 검증

- `cargo fmt --all -- --check`, workspace/native·WASM Clippy, workspace build, test-suite manifest — 통과.
- #7312 및 관련 TAC 회귀 10건: `cargo nextest run --cargo-profile release-test --tests -j 8` — **10 passed**.

## 최종 판정

**승인.** 기준 PDF와 보수적 적용 범위 보정이 같은 통합 후보에 포함되어 있다. #7312는 통합 PR 본문에서만
실제 종결 관계를 선언하고, merge 후 자동 종료 여부를 확인한다.

## Merge 후 contributor PR comment 계획

통합 PR의 **실제 merge SHA와 최종 head CI URL**이 확정된 뒤 원 PR에 한국어로 감사와 함께 남긴다.
`src/renderer/layout/paragraph_layout.rs`의 단일-segment 게이트가 보정한 계약, 실제 검증 범위
(한컴 2020 PDF 1쪽, Native·fresh WASM) 및 구조 flag 0건을 적는다. 다음 raw URL 두 개를 본문에
이미지로 넣어 Native/WASM overlay를 직접 보이게 한다.

```text
![Native overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7327_review/native_overlay_001.png)
![WASM overlay](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/pr/assets/pr7327_review/wasm_overlay_001.png)
```

글꼴 raster 차이로 ink match가 43.895%인 점은 숨기지 않고, 구조 후보가 0건인 범위와 별개로 기록한다.
[Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결한다.


본문은 UTF-8 파일로 만든 뒤 `gh pr comment --body-file`로 게시하고, 게시 후 한국어 본문·실제 merge SHA·CI URL·두 이미지 URL이 최종 head를 가리키는지 다시 확인한다.
