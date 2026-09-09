---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-03
---

# PR #3755 검토 - 중간 vpos 리셋 문단의 쪽 경계 분할

## 라우팅

base route: `maintainer_general.md`. 적용 보조 절차는 `intake_and_review.md`,
`local_validation.md`, `visual_fixture_evidence.md`, `multi_pr_update_branch.md`다.

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#3755](https://github.com/edwardkim/rhwp/pull/3755) / @planet6897 |
| 원 head | `801a16d42c2521659a545166f328a175d65bcd94` |
| 기준 devel | `6ab503fe97b7abfd1839800c5c018da9f9abf4c5` |
| 가시성 검토 브랜치 | `review/planet6897-20260803` |
| 누적 적용 commit | `f00143987` |
| 충돌 | 없음 |
| 작성 시점 원 PR 상태 | `MERGEABLE` / `BEHIND`, 원 head CI 성공. merge 전 재확인 필요 |

## 변경 검토

`height_for_fit`가 저장된 `LINE_SEG`의 처음과 끝만 비교해 span을 사용할지 정하던
경로를, 모든 인접 줄의 `vertical_pos`가 단조 증가하는 경우로 제한한다. 문단 중간에서
쪽 경계 때문에 vpos가 0으로 리셋되면 끝점은 증가해 보여도 span이 실제 문단 높이를
크게 과소평가한다. 변경은 그 경우 `st.current_height` 기반 경로를 사용하게 하므로,
남은 공간에 들어간다고 오판해 문단을 쪽 밖으로 배치하는 결함을 막는다.

새 fixture는 중간 리셋이 있으나 끝점만 보면 증가하는 모양을 고정한다. 따라서 변경 전
가드가 통과하던 형태와 변경 뒤 페이지 분할 결과를 각각 검증한다.

## 로컬 검증

| 게이트 | 결과 |
| --- | --- |
| `issue_3751_vpos_reset_midparagraph_fit` | 완료. 2 / 2 통과 |
| IR field sweep baseline | 완료. 589행 기준 TSV와 일치 |
| overflow-cell baseline | 완료. 22행 기준 TSV와 일치 |
| `cargo test --profile release-test --tests` | 완료. 실패 표식 없이 종료, 마지막 visual round-trip baseline 3 / 3 통과 |
| Native Skia 라이브러리 | 완료. 58 / 58 통과 |
| Native Skia `issue_2225_missing_picture_placeholder` | 완료. 2 / 2 통과 |
| Native Skia `render_p37_direct_pdf_export` | 완료. 4 / 4 통과 |
| WASM build | 완료. `wasm-pack build --target web --out-dir pkg` 성공, 생성물은 ignore 경로에만 존재 |
| `cargo fmt --check` / `git diff --check` | 완료. 모두 통과 |
| `cargo clippy --all-targets -- -D warnings` | 완료. 경고 없이 통과 |

## 시각 검증

typeset과 페이지 경계를 바꾸고 HWPX fixture를 추가하므로 시각 검증 대상이다.

- 원본 fixture SHA-256: `7005846cca1e651e028c9882d4aa330c7173f3ed09ffdfe078ac3ffbf4de5809`
- 한컴 2020 MCP 기준 PDF 시도: `run_status=1`, `validation=fail`,
  `validation_detail=invalid_output`. 한컴이 1쪽, 953바이트의 빈 PDF를 생성해 client가
  결과 파일을 보존하지 않았다. 따라서 이 파일에는 유효한 PDF/SVG sweep 지표를 만들 수 없다.
- rhwp 직접 SVG/PNG 확인: 2쪽을 생성했고 `overflowCellLines=0`이었다.
  1쪽에는 긴 문단의 앞부분, 2쪽에는 16–34행과 끝문단이 이어졌으며, focused test는
  쪽 높이에서 60px을 넘는 글자가 없음을 확인했다.
- 임시 산출물: `output/pr-planet6897-20260803/visual/pr3755-vpos/`.
- 보존 asset: `mydocs/pr/assets/pr_3755_vpos_reset_midparagraph_fit_rhwp_001.png`,
  `mydocs/pr/assets/pr_3755_vpos_reset_midparagraph_fit_rhwp_002.png`.

한컴 기준 PDF 생성 실패 원인은 이 검토에서 확정하지 않는다. 다만 MCP가 무효 출력을 거부한
사실은 `issue_3751`의 SVG 쪽 밖 글자 계약 실패와 별개이므로, 기준 PDF fidelity는 별도 MCP
호환성 작업으로 추적해야 한다.

## 현재 판정

**로컬 검증 수용 권고.** focused 회귀, 신규 fixture baseline, release-test 전체, Native
Skia 3종, WASM과 정적 검사를 통과했고, SVG/PNG로 문단의 쪽 경계 분할도 확인했다. 통합 PR의
최신 원격 CI와 작업지시자 승인만 merge 전 외부 조건으로 남는다.
