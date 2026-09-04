---
kind: pr-review
pr: 6683
reviewed_at: 2026-09-04
source_head: e5dde4373ed0c8d26543482c8031b0e2aa556baa
---

# PR #6683 검토 - 빈 문단 줄의 개체 높이 중복 산정

## 판정: 승인

원 PR은 글자가 없는 문단의 줄을 떠 있는 개체 높이와 중복해 더하지 않도록
`src/renderer/height_measurer.rs`를 좁힌다. 원 PR head는
`e5dde4373ed0c8d26543482c8031b0e2aa556baa`이고, 통합 후보에는 다음 두 commit으로
provenance를 보존해 적용했다.

| 구분 | commit |
| --- | --- |
| 원 변경 | `dd8ca73a2` |
| 후속 범위 축소 | `4c333ab94` |

## 검토 범위와 결과

- 빈 문단의 줄 높이를 항상 빼지 않고, 개체가 실제 칸을 넘는 경우로만 축소했다.
- 회귀 테스트 `issue_6660_empty_para_line_not_added_to_object_height`가 같은 commit에
  포함되어 있다.
- 원 PR의 required check는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6683/checks)
- 통합 후보에서 다음 로컬 호환/통합 검증이 성공했다. 이는 CI의 공식 full lane 또는
  nextest 전체 실행을 대체한다고 기록하지 않는다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 시각 증적

`samples/exam_science.hwp`의 4쪽을 동일 문서의 Hancom 2020 기준 PDF와 현재
`rhwp-studio` 렌더 결과로 직접 대조했다. 이 증적은 빈 문단/개체 높이 변경이
페이지 4의 2단 문제지 구조, 표, 도형의 배치를 무너뜨리지 않았음을 확인하는 범위이며,
문서 전체의 pixel-perfect 동치를 주장하지 않는다.

| 자료 | 경로 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 4쪽 | `../assets/pr_6683_6705_20260904/reference-6683-6690-exam-science-p4.png` | `42d4a5018d80272e07efb09e45eb9d556381fd5537d5ad08e44899020b596dbe` |
| 현재 Studio 4쪽 | `../assets/pr_6683_6705_20260904/studio-6683-6690-exam-science-p4.png` | `c2af0d6d027f4fda2909a282aa6e3d03b4a57914a54719c5da96d7196c81005b` |
| 기준 PDF | `pdf/exam_science-hwp-2020.pdf` | 저장소 추적 파일 |

## 메인터너 보정 판단

추가 메인터너 코드 보정은 필요하지 않다. 현재 보정의 범위가 원 PR의 회귀 계약과
시각 증적으로 뒷받침되며, 통합 PR을 만들 경우에는 그 최신 head에서 required CI를
다시 확인한 뒤에만 병합한다.

## Merge 후 contributor PR comment 계획

원 PR은 직접 merge하지 않고 [통합 PR #6722](https://github.com/edwardkim/rhwp/pull/6722)의
체리픽 통합으로 수용한다. 병합 뒤 comment에는 merge commit
[`4041acf`](https://github.com/edwardkim/rhwp/commit/4041acf298ffde2f02866587cf8ed4dcacd45f31),
실제 PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33854487320)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33854487302)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33854487296)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33854487297)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33854487178), devel push의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33856097121)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33856096995)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33856097070)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33856097150) 성공을 적는다.

- 로컬 검증은 `cargo nextest run --profile ci-duration-observation --cargo-profile release-test`의 실제 결과 `9010 passed, 46 skipped`만 기록한다.
- 시각 증적은 [Visual Sweep 가이드](https://github.com/edwardkim/rhwp/blob/4041acf298ffde2f02866587cf8ed4dcacd45f31/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)와 아래 devel asset을 직접 링크한다. Hancom 2020 기준과 Studio 4쪽의 2단·표·도형 범위만 확인했으며, 문서 전체 pixel-perfect 동치를 주장하지 않는다.
- 기준: `mydocs/pr/assets/pr_6683_6705_20260904/reference-6683-6690-exam-science-p4.png`
- Studio: `mydocs/pr/assets/pr_6683_6705_20260904/studio-6683-6690-exam-science-p4.png`
- comment와 close는 이 계획이 devel에 merge되고 devel CI가 성공한 뒤 각각 한 번만 수행한다.
