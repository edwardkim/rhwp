---
kind: pr-review
pr: 6705
reviewed_at: 2026-09-04
source_head: 05325df7c4350b101276580803a208c62709c05a
---

# PR #6705 검토 - 이어진 문단의 떠 있는 그림 anchor

## 판정: 승인

원 PR은 앞 쪽에서 이어진 문단에 붙은 떠 있는 그림을 본문 맨 위 기준으로 놓도록
`src/renderer/layout.rs`를 보정한다. 원 PR head
`05325df7c4350b101276580803a208c62709c05a`는 통합 후보에 `7b15d9582`,
`dda7902e5`로 provenance를 보존해 적용됐다.

## 검토 범위와 결과

- 변경은 continued paragraph에 anchor된 floating picture의 top placement로 한정된다.
- 회귀 테스트 `issue_6704_floating_picture_anchor_on_continued_para`가 함께
  포함되어 있다.
- 원 PR의 required check는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6705/checks)
- 통합 후보에서 다음 로컬 호환/통합 검증이 성공했다. 이는 공식 CI의 full lane이나
  nextest 전체 실행을 대체하지 않는다.

```sh
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 시각 증적

`samples/hwp3-sample.hwp`의 문서상 7쪽(렌더러 zero-based index 6)을 Hancom 2020
기준 PDF와 현재 `rhwp-studio`에서 직접 검토했다. 중앙 network diagram, 하단 footer와
그림 영역의 상하 관계가 기준과 같은 범위로 확인됐다. 이는 floating-picture anchor
계약의 특정 페이지 증적이며 문서 전체 pixel-perfect 동치를 뜻하지 않는다.

| 자료 | 경로 | SHA-256 |
| --- | --- | --- |
| Hancom 2020 기준 7쪽 | `../assets/pr_6683_6705_20260904/reference-6705-hwp3-p7.png` | `c2d41f376fb747272129f624d679b6f0b9976873350d518ca3da48e56fad3d17` |
| 기준 PDF | `pdf/hwp3-sample-hwp-2020.pdf` | 저장소 추적 파일 |

## 메인터너 보정 판단

추가 메인터너 코드 보정은 필요하지 않다. 최종 병합은 통합 PR 최신 head의 required
CI, mergeability 및 `mergeStateStatus=CLEAN`을 다시 확인한 뒤에만 진행한다.

## Merge 후 contributor PR comment 계획

원 PR은 직접 merge하지 않고 [통합 PR #6722](https://github.com/edwardkim/rhwp/pull/6722)의
체리픽 통합으로 수용한다. comment에는 merge commit
[`4041acf`](https://github.com/edwardkim/rhwp/commit/4041acf298ffde2f02866587cf8ed4dcacd45f31),
실제 PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33854487320)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33854487302)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33854487296)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33854487297)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33854487178), devel push의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33856097121)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33856096995)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33856097070)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33856097150) 성공을 적는다.

- 로컬 검증은 `cargo nextest run --profile ci-duration-observation --cargo-profile release-test`의 실제 결과 `9010 passed, 46 skipped`만 기록한다.
- `mydocs/pr/assets/pr_6683_6705_20260904/reference-6705-hwp3-p7.png`는 한컴 기준 7쪽의 보관 증적이다. 현재 devel에는 동일 쪽 Studio PNG가 없으므로 comment는 외부 기준과의 시각 동치 통과를 주장하지 않고, continued paragraph floating-picture anchor 회귀 계약과 공식 CI 결과만 수용 근거로 적는다.
- comment와 close는 이 계획이 devel에 merge되고 devel CI가 성공한 뒤 각각 한 번만 수행한다.
