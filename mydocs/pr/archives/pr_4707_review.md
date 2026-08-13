---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4707 검토 - 쪽 경계 병합 셀의 저장 조각 소유

| 항목 | 기록 |
| --- | --- |
| PR | [#4707](https://github.com/edwardkim/rhwp/pull/4707) |
| 작성자 / 원 head | @planet6897 / `5726681bec5df3ffb34bff064204900cc4660e2e` |
| 적용 commit | `c7cfaefb9`의 최상단 commit |
| 관련 이슈 | [#4698](https://github.com/edwardkim/rhwp/issues/4698) |

PR source ref에는 과거 history가 함께 보이지만 GitHub PR의 고유 변경은 위 한 commit뿐이다. 따라서
그 commit만 최신 `devel` 위에 적용했다. 쪽 경계에 걸친 rowspan 셀에서 저장 `LINE_SEG.vpos == 0`
재시작을 조각 경계로 사용하고, 실제 row filter 분할인 경우에만 문단을 앞·뒤 셀 조각으로 나눈다.

## 완료한 검증

- focused nextest 8개 target, 35건을 실행해 모두 통과했다. #1073·#2007·#3592·#3820·#4326·#4252·#1156 회귀를 함께 포함했다.
- 누적 후보 전체 `nextest`는 5,923건 통과, 37건 제외, 실패 0건이었다.
- `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`를 통과했다.

## 한컴 2020 기준 시각 확인

기준은 `samples/kps-ai.hwp`(SHA-256 `9b0fceb3d96956f27c893e15a72a1ad94f7ee005bd581381a1aadfcb1f57a7b9`)와
한컴 2020 PDF `pdf/kps-ai-2022.pdf`(SHA-256 `7c064fd290368369a3c8eaa7d7b03668c46fb4dfe0fc18ba52d00456ffe01d28`)다.

- 첫 조각: [한컴 p65](../assets/pr_4707_hancom2020_p065_first_fragment_label.png)와 [rhwp p66](../assets/pr_4707_rhwp_p066_first_fragment_label.png)에 `3. 민간 / 소프트웨어`가 남는다.
- 다음 조각: [한컴 p66](../assets/pr_4707_hancom2020_p066_continuation_label.png)와 [rhwp p67](../assets/pr_4707_rhwp_p067_continuation_label.png)에 `시장침해 / 가능성`이 나온다.

fidelity helper의 Snap Chromium PNG 출력은 AppArmor Fontconfig 제약으로 실패했으므로 `rsvg-convert`와
`pdftoppm`으로 같은 대상 라벨 crop을 만들었다. 이 증적은 이번 병합 셀 소유를 확인한 것이며,
문서 전체의 pixel-perfect 정합을 주장하지 않는다.

**통합 수용 대상이다. #4698 close는 통합 PR merge 뒤 최신 상태를 다시 확인해 처리한다.**
