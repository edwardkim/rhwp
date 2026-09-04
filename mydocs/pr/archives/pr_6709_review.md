---
kind: pr-review
pr: 6709
reviewed_at: 2026-09-04
source_head: 36b5500891e750be7680c2559e2c278d4cbbe175
maintainer_correction: bc7baa359
---

# PR #6709 검토 - 용지 기준 어울림 개체의 배제 밴드

## 판정: 메인터너 보정 후 수용 가능

**보정 상태: 메인터너 보정 완료.** 원 PR `#6709`은 용지 기준으로 배치된 어울림
개체의 배제 밴드를 계산하고, 개체 이동 뒤 본문을 되감도록 renderer를 보정한다.
원 PR head `36b5500891e750be7680c2559e2c278d4cbbe175`는 현재 통합 후보에 다음 두
commit으로 체리픽되어 있다.

| 구분 | 통합 후보 commit |
| --- | --- |
| 배제 밴드 계산 및 회귀 테스트 | `e6b9a3ed5` |
| 용지 기준 세로를 밴드-로컬 좌표로 변환 | `ffd47191e` |
| 정식 fixture 등록 메인터너 보정 | `bc7baa359` |

## 검토 범위

- `src/renderer/composer/line_breaking.rs`와 `src/renderer/float_placement.rs`에서
  용지 기준 float의 exclusion band와 본문 재조판 경로를 변경한다.
- command/text editing 경로는 본문 되감기와 연동하는 최소 범위로만 수정한다.
- 회귀 테스트 `issue_6202_paper_relative_float_exclusion`가 포함되어 있다.
- 원 PR의 required `Build & Test`는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6709/checks)

## 메인터너 보정

원 테스트는 비공개 Windows 경로와 환경 변수에 의존해 공개 CI에서 sample을 보장하지
못했다. 메인터너 보정 `bc7baa359`는 다음을 수행했다.

- 기준 HWP를 `samples/issue6202/156483689-turmeric-industry-standardization.hwp`로
  정식 등록했다.
- `MANIFEST.json`, `README.md`, `.gitattributes`로 출처, SHA-256, binary 취급을
  저장소 계약으로 만들었다.
- 테스트가 정식 sample을 반드시 읽도록 바꾸고, 개인 경로 탐색과 silent skip을
  제거했다.

## 실행한 검증

다음은 보정 뒤 현재 통합 후보에서 성공한 로컬 호환/통합 검증이다. 이는 GitHub
required CI 또는 nextest 공식 full lane을 대체한다고 표기하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 보조 N-up sweep 기록

Hancom 2018 저장 원본은 `printMethod=4` N-up PDF로 출력돼 물리 페이지와 논리 페이지의
1:1 pixel 판정을 만들지 않는다. 아래 자료는 물리 시트의 좌우 영역을 논리 A4 크기로만
균일 변환해 구조·프레임·흐름 후보를 검사한 보조 기록이다. 이를 Hancom 정본과의 완전한
시각 동치나 Studio 직접 비교로 주장하지 않는다.

| 자료 | 경로 | SHA-256 / 결과 |
| --- | --- | --- |
| Hancom 2020 기준 PDF | `pdf/issue6202-156483689-2020.pdf` | `3154313e2bbaf793dfe2f6c505768cffb6d1097d9019fb6cf8d50d7659c701a7` |
| 논리 페이지 매핑 | `../assets/pr_6683_6705_20260904/visual-6709-6710/nup-logical-a4-normalized-page-map.json` | 8 논리 페이지 |
| 대표 contact sheet | `../assets/pr_6683_6705_20260904/visual-6709-6710/issue6202-a4-normalized-contact-sheet.png` | 8/8 완료, 규칙 후보 0 |

## 병합 후 상태 및 contributor PR comment 계획

정식 fixture와 HWP5 `LIST_HEADER` 보정까지 포함한 [통합 PR #6722](https://github.com/edwardkim/rhwp/pull/6722)는
`MERGEABLE`·`CLEAN`과 required CI를 확인한 뒤 merge commit
[`4041acf`](https://github.com/edwardkim/rhwp/commit/4041acf298ffde2f02866587cf8ed4dcacd45f31)로
병합됐다. 원 PR은 직접 merge하지 않고 이 체리픽 통합으로 수용한다.

- comment에는 실제 PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33854487320)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33854487302)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33854487296)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33854487297)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33854487178), devel push의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33856097121)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33856096995)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33856097070)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33856097150) 성공을 적는다.
- 로컬 검증은 `cargo nextest run --profile ci-duration-observation --cargo-profile release-test`의 실제 결과 `9010 passed, 46 skipped`만 기록한다.
- [통합 시각 sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)은 issue6202의 실제 PNG 8쪽·SVG 8쪽과 N-up contact sheet를 보관한다. comment에는 `mydocs/pr/assets/pr_6683_6710_green_ci_batch_20260904/formal-fixture-render/issue6202/png/156483689-turmeric-industry-standardization_001.png`와 `mydocs/pr/assets/pr_6683_6705_20260904/visual-6709-6710/issue6202-a4-normalized-contact-sheet.png`를 직접 표시하고, N-up 물리 시트의 제한 때문에 Hancom PDF와의 pixel/물리 페이지 동치는 주장하지 않는다.
- 수용 근거는 공개 fixture, fail-closed 테스트 계약, HWP5 직렬화 보정과 공식 CI다. 시각 asset은 현재 rhwp 출력의 검토 범위만 보여 준다.
- comment와 close는 이 계획이 devel에 merge되고 devel CI가 성공한 뒤 각각 한 번만 수행한다.
