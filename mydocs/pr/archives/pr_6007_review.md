---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #6007 검토 - 레거시 HWP 2020 MCP 안내 제거

## 접수 메타데이터

| 항목 | 작성 시점 확인값 |
| --- | --- |
| PR / 작성자 | [#6007](https://github.com/edwardkim/rhwp/pull/6007) / [@jangster77](https://github.com/jangster77) |
| base / head | `devel` / `d73fb9fd050899612f3e45f095bdec67cc1b88e5` |
| 규모 | 6 files, +11 / -357 |
| 작성 시점 상태 | non-draft, `MERGEABLE`, `BLOCKED`; CI preflight와 CodeQL preflight 진행 중 |
| reviewer | 작성자 본인 self-review, 별도 reviewer 미지정 |

라우팅은 collaborator self-merge이며, 접수·리뷰 기록, 로컬 검증, review-only fast-pass를 함께 적용했다.
읽은 문서는 `pr_review_workflow.md`, `pr_review/README.md`, `collaborator_self_merge.md`,
`intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`다.

GitHub mergeability와 CI 상태는 작성 시점 참고값이다. 이 review·오늘할일 trailing commit의 최신 head가
required check를 통과하고, merge 직전에 같은 head SHA와 `MERGEABLE/CLEAN`을 다시 확인한 뒤에만 merge한다.

## 변경과 판단

- superseded 상태였던 `mydocs/manual/mcp_hwp2020Convert_usage.md`와
  `tools/hwp-convert-mcp-2022-client-20260805-071707.tar.gz`를 제거했다.
- 현행 문서와 manifest에서 별도 HWP 2020 MCP 안내·링크를 없앴다. 한컴오피스 2022 이하 저장본도 동일한
  Windows `hwp-convert-2024` client에서 `engine: 2020`을 사용하고, 2024 저장본은 `engine: 2024`를
  사용한다는 단일 선택 규칙을 명시했다.
- 역사 PR #2255 기록은 삭제된 매뉴얼의 hyperlink를 현재 제거되었음을 알리는 일반 텍스트로 바꿔
  archive의 링크 무결성을 유지했다.
- Rust, renderer, Studio, fixture, 기준 PDF, CI workflow는 변경하지 않았다. 따라서 시각 검증과 Cargo
  회귀는 적용 대상이 아니다.

## 로컬 검증

- `git diff --check`를 통과했다.
- `venv\\Scripts\\python.exe scripts\\check_markdown_links.py`를 통과했다: 601개 Markdown 문서, 깨진
  내부 링크 없음.
- 삭제된 `mcp_hwp2020Convert_usage.md` Markdown 링크와 레거시 HWP 2020/2022 client archive의 현행 참조가
  남지 않았음을 검색으로 확인했다.
- `scripts\\check_document_metadata.py` 전수 검사는 이번 변경과 무관한 기존 문서 4개의 front matter 누락
  16건으로 실패했다. 변경 문서는 오류 목록에 없으며, 이 기준선 오류를 이번 PR에 섞어 수정하지 않았다.

## GitHub Actions

작성 시점에는 CI preflight와 CodeQL preflight가 진행 중이었고, Adapter inter-diff preflight와 Proptest
preflight는 성공했다. 문서-only PR이므로 최신 trailing head에서 review-only fast-pass aggregate와 required
check의 최종 성공을 다시 확인한다.

## 최종 판정

**수용 권고, trailing CI 대기.** 현행 문서가 단일 remote MCP client와 저장 버전별 `engine` 선택을 일관되게
가리킨다. 최신 trailing head의 required checks 성공, `MERGEABLE/CLEAN`, 작업지시자의 merge 승인을 확인한
뒤 squash merge한다.
