---
kind: pr-review
pr: 6690
reviewed_at: 2026-09-04
source_head: c379257716458c30028dbd44f84ce8b463c0b96d
---

# PR #6690 검토 - 개체만 담긴 칸 마지막 줄의 꼬리 줄간격

## 판정: 승인

원 PR은 글자 없이 개체만 담긴 table cell의 마지막 줄에 꼬리 줄간격을 더하지 않도록
`src/renderer/height_measurer.rs`를 보정한다. 원 PR head
`c379257716458c30028dbd44f84ce8b463c0b96d`는 통합 후보에 다음 commit으로 적용됐다.

| 구분 | commit |
| --- | --- |
| 원 변경과 회귀 테스트 | `eb84bbbc7` |
| clippy 후속 정리 | `9f0455b6f` |

## 검토 범위와 결과

- 변경은 개체만 있는 마지막 줄로 한정되어 일반 텍스트 줄간격 처리에는 적용되지 않는다.
- 회귀 테스트 `issue_6681_cell_last_object_line_drops_trailing_ls`가 포함되어 있다.
- 원 PR의 required check는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6690/checks)
- 통합 후보의 로컬 호환/통합 검증은 다음 명령으로 성공했다. 공식 CI full lane 또는
  nextest 전체 검증을 대체하지 않는다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 시각 증적

이 변경도 `samples/exam_science.hwp`의 4쪽에서 원본 Hancom 2020 PDF와 현재
Studio 출력을 직접 비교했다. `#6683`과 같은 page evidence를 공유하지만, 판정은
개체-only 마지막 줄의 trailing line-spacing 계약에만 한정한다.

| 자료 | 경로 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 4쪽 | `../assets/pr_6683_6705_20260904/reference-6683-6690-exam-science-p4.png` | `42d4a5018d80272e07efb09e45eb9d556381fd5537d5ad08e44899020b596dbe` |
| 현재 Studio 4쪽 | `../assets/pr_6683_6705_20260904/studio-6683-6690-exam-science-p4.png` | `c2af0d6d027f4fda2909a282aa6e3d03b4a57914a54719c5da96d7196c81005b` |

## 메인터너 보정 판단

추가 메인터너 코드 보정은 필요하지 않다. 최종 병합은 통합 PR 최신 head의 required
CI와 mergeability를 다시 확인하는 일반 절차를 따른다.

## Merge 후 contributor PR comment 계획

원 PR은 직접 merge하지 않고 [통합 PR #6722](https://github.com/edwardkim/rhwp/pull/6722)의
체리픽 통합으로 수용한다. comment에는 merge commit
[`4041acf`](https://github.com/edwardkim/rhwp/commit/4041acf298ffde2f02866587cf8ed4dcacd45f31),
실제 PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33854487320)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33854487302)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33854487296)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33854487297)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33854487178), devel push의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33856097121)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33856096995)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33856097070)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33856097150) 성공을 적는다.

- 로컬 검증은 `cargo nextest run --profile ci-duration-observation --cargo-profile release-test`의 실제 결과 `9010 passed, 46 skipped`만 기록한다.
- 시각 증적은 [Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/4041acf298ffde2f02866587cf8ed4dcacd45f31/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)와 `mydocs/pr/assets/pr_6683_6705_20260904/reference-6683-6690-exam-science-p4.png`, `mydocs/pr/assets/pr_6683_6705_20260904/studio-6683-6690-exam-science-p4.png`를 직접 링크한다. page 4의 개체-only 마지막 줄 계약 범위만 기록하며 전체 pixel-perfect 동치를 주장하지 않는다.
- comment와 close는 이 계획이 devel에 merge되고 devel CI가 성공한 뒤 각각 한 번만 수행한다.
