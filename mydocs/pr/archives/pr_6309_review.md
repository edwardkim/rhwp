---
kind: review
status: accepted_with_maintainer_correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-28
---

# PR #6309 검토 - 균일 ladder square band

- PR: [#6309](https://github.com/edwardkim/rhwp/pull/6309)
- 이슈: [#6175](https://github.com/edwardkim/rhwp/issues/6175)
- 작성자: `@planet6897`
- 원 source head: `00fdc870bd424c3ab311ec1fef2922acdcaa7516`
- 누적 검토 적용: 원 PR `a1666c563` + 메인터너 보정 `9ac5f9b3a`

## 변경 검토

이 PR은 `line_breaking.rs`와 `typeset.rs`에서 Square wrap band 안의 균일하게 좁은 저장 행을
보존하고 #6175 회귀 fixture를 추가한다. 문제는 원 PR이 실제 도형 근거 없이 균일 inset만으로
외부 geometry를 추정했다는 점이다. 일반 문단 들여쓰기나 table inset도 재흐름에서 제외될 수 있었다.

메인터너는 `#6314`의 폭·세로 위치 기반 `FloatCarveEvidence`를 보존한 채 `#6309`의
`GroupShape` 사례를 결합했다. 기본 composer 경로는 균일 inset만으로 저장 행을 보존하지 않으며,
실제 비-TAC `Square` Picture/Shape wrap anchor가 확인된 pagination·render 경로 또는
폭과 세로 band가 모두 맞는 float evidence가 있을 때만 보존한다.

## 검증 증적

- 원 source head의 GitHub required CI는 성공했다.
- focused Rust: `regression_suite_007`의
  `issue_6175_uniformly_narrowed_ladder_keeps_square_band`가 `1 passed`로 성공했다.
- 원본 corpus `경찰청/156518601_220728(0900) 모바일 운전면허증 전국 발급(교통기획).hwpx`는
  한컴오피스 2018 저장본이므로 HWP2024 MCP를 `--engine 2020`으로 단일 요청해 PDF를 산출했다.
- 통합 코드의 native-skia 1쪽 render tree에서 대상 `pi=8`은 폭 `388.5px`의 네 줄이고,
  `GroupShape`는 `x=471.6px`에서 시작한다. 네 줄의 우단은 도형 영역에 침범하지 않는다.
- 한컴 PDF와 rhwp PNG는 같은 네 줄 Square band 구조를 보인다. macOS의 글꼴 외형 차이는
  글꼴 환경 차이이므로 glyph pixel 일치 판정에는 사용하지 않았다.
- 산출물: `pdf/pr_6309_156518601_mobile_driver_license-2020.pdf`와
  `output/pr_6309_156518601_main_combined_maintainer_correction_20260828/`.

## 최종 판정 - 메인터너 보정 후 수용

기존 보류는 `#6314`와 같은 이슈를 다룬다는 이유만으로 두 구현을 경쟁 대안으로 본 판단이었다.
그러나 `#6309`은 빈 host 문단과 `GroupShape` 때문에 일반 Paper float 경로만으로는 재현되지 않는
원본 문서 사례를 해결한다. 원 PR의 과도한 균일-inset 일반화는 메인터너 보정으로 실제
`Square GroupShape` wrap anchor 또는 `FloatCarveEvidence`가 확인된 경로로 제한했다.

focused 회귀와 통합 코드의 원본 문서 렌더가 모두 성공했으므로, 이 통합 묶음에서는
**메인터너 보정 후 수용**한다.
