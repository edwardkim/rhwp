---
kind: pr-review
pr: 6710
reviewed_at: 2026-09-04
source_head: 4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb
maintainer_correction: bc7baa359
---

# PR #6710 검토 - 저장 첫 조각의 source-frame allowance

## 판정: 메인터너 보정 후 수용 가능

**보정 상태: 메인터너 보정 완료.** 원 PR `#6710`은 저장된 첫 조각의 초과 허용치를
origin marker 유무가 아니라 source-frame 계약으로 적용하도록 `typeset` 처리를
보정한다. 원 PR head `4a1eb7c27552dd9dd619a9aef1b9aaaf997a6fdb`는 현재 통합 후보에
다음 commit으로 체리픽되어 있다.

| 구분 | 통합 후보 commit |
| --- | --- |
| source-frame allowance 보정 및 회귀 테스트 | `c340bd7a8` |
| 전·후 PNG 보고 자료 | `61cd71fb9` |
| 정식 fixture 등록 메인터너 보정 | `bc7baa359` |

## 검토 범위

- `src/renderer/typeset.rs`에서 저장 첫 조각 allowance의 적용 조건을 origin marker와
  분리한다.
- `issue_4658_ir_diff_pagecount`와
  `issue_5057_profile_agnostic_source_frame_allowance` 회귀 계약이 함께 수정됐다.
- 원 PR의 required `Build & Test`는 2026-09-04 조회 시 성공이었다.
  [Checks](https://github.com/edwardkim/rhwp/pull/6710/checks)

## 메인터너 보정

원 회귀 테스트는 비공개 Windows 경로와 환경 변수로 sample을 찾고, 자료가 없으면
성공처럼 반환했다. 메인터너 보정 `bc7baa359`로 다음을 완료했다.

- 원본을 `samples/issue5057/21484591-gimcheon-sewage-ordinance.hwp`로 정식 sample로
  등록했다.
- `MANIFEST.json`, `README.md`, `.gitattributes`로 공개 fixture 계약을 명시했다.
- 테스트가 저장소 sample을 반드시 읽도록 바꾸고 private-path 탐색과 silent skip을
  제거했다.

## 전·후 PNG의 실제 범위

`mydocs/report/5057-origin-marker-profile/before_p7.png`와 `after_p7.png`는
renderer 변경 전후를 보여 주는 보조 보고 자료다. 이 두 PNG만으로 Hancom 정본과
일치한다고 주장하지 않으며, 외부 기준 PDF를 대체하지 않는다.

| 자료 | 경로 | 의미 |
| --- | --- | --- |
| 변경 전 | `mydocs/report/5057-origin-marker-profile/before_p7.png` | 기존 renderer 출력 |
| 변경 후 | `mydocs/report/5057-origin-marker-profile/after_p7.png` | 보정 renderer 출력 |

## 실행한 검증

다음은 보정 뒤 현재 통합 후보에서 성공한 로컬 호환/통합 검증이다. GitHub required
CI 또는 nextest 공식 full lane을 실행한 것으로 표기하지 않는다.

```sh
node scripts/rust-test-suite-manifest.mjs --prepare
CARGO_TARGET_DIR=target/pr-review/green-ci-batch-20260904-full \
  cargo test --profile release-test --tests
```

## 보조 N-up sweep 기록

Hancom 2010 저장 원본의 `printMethod=4` 출력은 물리 PDF 시트와 논리 페이지가 일대일이
아니다. 아래 자료는 물리 시트의 좌우 영역을 논리 A4 크기로만 균일 변환해 구조·프레임·흐름
후보를 검사한 보조 기록이다. 이는 Hancom 정본과의 완전한 시각 동치나 Studio 직접 비교의
대체 근거가 아니다.

| 자료 | 경로 | SHA-256 / 결과 |
| --- | --- | --- |
| Hancom 2020 기준 PDF | `pdf/issue5057-21484591-2020.pdf` | `78ef349ce8936a7cfaa4e671c1ca1318e31cda3b91c303ebc86b91c1660bae54` |
| 논리 페이지 매핑 | `../assets/pr_6683_6705_20260904/visual-6709-6710/nup-logical-a4-normalized-page-map.json` | 13 논리 페이지 |
| 대표 contact sheet | `../assets/pr_6683_6705_20260904/visual-6709-6710/issue5057-a4-normalized-contact-sheet.png` | 13/13 완료, 규칙 후보 0 |

## 병합 후 상태 및 contributor PR comment 계획

정식 fixture와 HWP5 `LIST_HEADER` 보정까지 포함한 [통합 PR #6722](https://github.com/edwardkim/rhwp/pull/6722)는
`MERGEABLE`·`CLEAN`과 required CI를 확인한 뒤 merge commit
[`4041acf`](https://github.com/edwardkim/rhwp/commit/4041acf298ffde2f02866587cf8ed4dcacd45f31)로
병합됐다. 원 PR은 직접 merge하지 않고 이 체리픽 통합으로 수용한다.

- comment에는 실제 PR head의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33854487320)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33854487302)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33854487296)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33854487297)·[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/33854487178), devel push의 [CI](https://github.com/edwardkim/rhwp/actions/runs/33856097121)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/33856096995)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/33856097070)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/33856097150) 성공을 적는다.
- 로컬 검증은 `cargo nextest run --profile ci-duration-observation --cargo-profile release-test`의 실제 결과 `9010 passed, 46 skipped`만 기록한다.
- [통합 시각 sweep](pr_6683_6710_green_ci_batch_visual_sweep.md)은 issue5057의 실제 PNG 13쪽·SVG 13쪽과 N-up contact sheet를 보관한다. comment에는 `mydocs/pr/assets/pr_6683_6710_green_ci_batch_20260904/formal-fixture-render/issue5057/png/21484591-gimcheon-sewage-ordinance_007.png`와 `mydocs/pr/assets/pr_6683_6705_20260904/visual-6709-6710/issue5057-a4-normalized-contact-sheet.png`를 직접 표시하고, N-up 물리 시트의 제한 때문에 Hancom PDF와의 pixel/물리 페이지 동치는 주장하지 않는다.
- 수용 근거는 공개 fixture, fail-closed 테스트 계약, HWP5 직렬화 보정과 공식 CI다. 시각 asset은 현재 rhwp 출력의 검토 범위만 보여 준다.
- comment와 close는 이 계획이 devel에 merge되고 devel CI가 성공한 뒤 각각 한 번만 수행한다.
