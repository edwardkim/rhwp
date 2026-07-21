# PR #2753: mydocs/report 파일명 규칙 정리

## 배경

`mydocs/report/` 디렉터리의 보고서 파일명은 `docs_and_git_workflow.md` 에 정의된 대로 `task_m100_{issue}_report.md` 형식을 따라야 한다.

그러나 2026-07-21 기준 전수 조사 결과, `PR-*` / `pr-*` 계열의 비정규 파일명 22건이 발견되어 `task_m100_{issue}_report.md` 형식으로 일괄 rename 하였다.

## rename 대상 22건

| 변경 전 | 변경 후 | 이슈 번호 |
|---|---|---|
| `PR-2607-vscode-hml-context-menu.md` | `task_m100_2606_report.md` | #2606 |
| `PR-2609-ext-hml-context-menus.md` | `task_m100_2608_report.md` | #2608 |
| `PR-2611-download-interceptor-hml.md` | `task_m100_2610_report.md` | #2610 |
| `PR-2613-hml-interceptor-test.md` | `task_m100_2612_report.md` | #2612 |
| `PR-2615-download-interceptor-comment-hml.md` | `task_m100_2614_report.md` | #2614 |
| `PR-2620-url-validator-hml.md` | `task_m100_2619_report.md` | #2619 |
| `PR-2622-compare-dialog-hml.md` | `task_m100_2621_report.md` | #2621 |
| `PR-2624-recovery-format-hml.md` | `task_m100_2623_report.md` | #2623 |
| `PR-combined-dep-bump-2604.md` | `task_m100_2604_report.md` | #2604 |
| `pr-codeql-hml.md` | `task_m100_2642_report.md` | #2642 |
| `pr-editor-readme-hml.md` | `task_m100_2691_report.md` | #2691 |
| `pr-ext-hml-regex.md` | `task_m100_2689_report.md` | #2689 |
| `pr-hwpctl-open-cursor.md` | `task_m100_2693_report.md` | #2693 |
| `pr-hwpctl-saveas-hml.md` | `task_m100_2625_report.md` | #2625 |
| `pr-hwpctl-saveas-markclean.md` | `task_m100_2661_report.md` | #2661 |
| `pr-hwpctl-trycatch.md` | `task_m100_2684_report.md` | #2684 |
| `pr-rawsvg-delay-fix.md` | `task_m100_2635_report.md` | #2635 |
| `pr-recovery-deadcode.md` | `task_m100_2687_report.md` | #2687 |
| `pr-recovery-ui-hml-test.md` | `task_m100_2634_report.md` | #2634 |
| `pr-recovery-ui-hml.md` | `task_m100_2628_report.md` | #2628 |
| `pr-renderdiff-hml.md` | `task_m100_2652_report.md` | #2652 |
| `pr-security-policy.md` | `task_m100_2641_report.md` | #2641 |

> **참고:** `PR-` 접두가 붙은 8건은 파일명의 숫자(PR 번호)와 실제 이슈 번호가 1씩 어긋나 있다.
> 반드시 본문의 `Issue: #NNNN` 줄을 근거로 매핑하였으며, 기계적 rename을 하지 않도록 주의하였다.

## 회차형 측정 기록 예외 명문화

다음 15건은 rename 대상에서 제외한다. 이들은 이슈 1:1 대응이 아니므로 `task_m100_{issue}_report.md` 규칙을 적용하지 않는다.

| 파일 | 성격 |
|---|---|
| `survey_10k_r5_20260706.md` ~ `survey_10k_r18_20260721.md` (12건) | #2279 캠페인의 10k 오라클 서베이 회차 기록 |
| `survey_pipage_20260703.md` (1건) | 파이프페이지 서베이 |
| `hwpx_lossless_3axis_20260627.md` 외 2건 (3건) | HWPX Lossless 3축 실측 시리즈 |

이 문서들은 **동일 축을 반복 측정하여 시계열로 비교하는 회차형 측정 기록**이다.
`task_m100_2279_report.md` 로 몰면 12개 회차가 한 이름을 두고 충돌하며, 시계열 비교라는 문서의 목적도 사라진다.

따라서 `docs_and_git_workflow.md` 에 다음 예외 조항을 추가한다:

> 회차형 측정 기록(서베이·벤치마크 등 동일 축을 반복 측정해 시계열로 비교하는 문서)은
> `{주제}_{회차}_{YYYYMMDD}.md` 를 쓴다. 이슈 1:1 대응이 아니므로 `task_m100_{issue}_report.md`
> 규칙을 적용하지 않는다.

## 재발 방지

위반 22건 중 21건이 2026-07-20~21 이틀 사이에 유입되었다.
통합 PR 검토 시 코드 게이트(fmt/clippy/스위트/red→green)는 통과했으나 문서 파일명은 점검 항목에 없었다.

- `pr_review_workflow.md` 의 검토 체크리스트에 다음 항목을 추가:
  "`mydocs/report/` 신규 파일이 `task_m100_{issue}_report.md` 규칙을 따르는지 확인"
