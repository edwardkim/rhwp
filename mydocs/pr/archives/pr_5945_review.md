---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-23
---

# PR #5945 검토 - PR 검토 PDF 기준 MCP 선택 명확화

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#5945](https://github.com/edwardkim/rhwp/pull/5945) / [@jangster77](https://github.com/jangster77) |
| base / 문서 candidate | `devel` `d78cb44798ddfc5c0bd061f5faa9a7e54a756b54` / `948cd34191a9a3df49b06e9938e4ab7c56acb26d` |
| 변경 규모 | 1 file, +19 / -6 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `BLOCKED`; required check 집계 전 |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

적용 경로는 `collaborator_self_merge`이며, `intake_and_review`와 `review_only_fast_pass`를 보조 경로로
선택했다. GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 review·오늘할일 trailing commit의 최신
head가 required check를 통과하고, merge 직전에 같은 head SHA와 `MERGEABLE/CLEAN`을 다시 확인한 뒤에만 merge한다.

## 변경과 판단

- #5944가 HWP5 `HwpSummaryInformation.revisionNumber`와 HWPX `version.xml/appVersion`에서
  `lastSavedWith`를 반환하도록 구현하고, HWP 2020/2024 MCP 사용 문서를 갱신했다.
- 이 PR은 기준 PDF가 없는 PR의 상위 시각 검증 절차에도 `rhwp info --json <원본>`을 명시해, 실제 MCP
  선택 근거가 구현·서비스별 사용 문서와 일치하도록 한다.
- `hancom-office-2010`·`2018`·`2020`·`2022`는 HWP 2020 MCP, `hancom-office-2024`는 HWP 2024 MCP를
  선택한다. 확장자와 HWP 포맷 `version`은 제품 연도 판정 근거가 아니다.
- `lastSavedWith` 또는 `product`가 `null`인 경우에는 자동 선택하지 않고, 기준 PDF 또는 저장 환경 같은 별도
  근거를 확보해 review 문서에 기록하도록 fail-closed 규칙을 추가했다.
- MCP 선택 전 `format`·`lastSavedWith`와 이후 서비스·PDF·job 식별자를 같은 review 기록에 보존하게 했다.

이 PR은 renderer, parser, MCP client, fixture 또는 기준 PDF를 변경하지 않는다. 따라서 실제 변환이나 visual
sweep은 수행 대상이 아니며, #5944에서 검증한 저장 메타데이터 계약을 문서 절차에 연결하는 범위다.

## 로컬 검증

- `git diff --check`를 통과했다.
- `cargo fmt --all`과 `cargo fmt --all -- --check`를 통과했다.
- 변경은 `mydocs/` 문서 한 파일뿐이다. 문서 링크 대상인 HWP 2020/2024 MCP 사용 문서의 존재를 확인했다.
- 일반 Markdown 수정에는 자동 전체 검증을 실행하지 않는 문서·Git 절차에 따라 Cargo 회귀, MCP 변환, visual
  sweep은 실행하지 않았다.

## GitHub Actions와 최종 판정

이 PR 전체는 `mydocs/`만 변경하는 review-only 범위다. 최신 head의 preflight와 branch protection 집계가
성공하면 heavy job의 skip은 정상이다. CI 완료 뒤 exact trailing head의 check, `MERGEABLE/CLEAN`, source
repository와 SHA를 다시 확인한다.

**수용 권고, trailing CI 대기.** #5944의 HWP5/HWPX 저장 제품 판별 계약을 상위 PR 검토 절차에 빠짐없이
연결하며, 메타데이터 누락 시 서비스 선택을 추정하지 않는다.
