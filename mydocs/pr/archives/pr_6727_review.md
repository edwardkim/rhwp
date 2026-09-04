---
kind: pr_review
status: active
pr: 6727
---

# PR #6727 검토 기록

## 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6727](https://github.com/edwardkim/rhwp/pull/6727) |
| 작성자·검토자 | `jangster77` self-review |
| base / head | `devel` / `codex/remove-hwp2024-sync-guide` |
| 검토 대상 head | `ac033c3b2da0ba33c1e9e0bd7abd135da39997cc` |
| 변경 규모 | 1개 파일, 15 추가·83 삭제 |
| 작성 시점 mergeability | `MERGEABLE`; CI 완료 전 `BLOCKED` |

## 검토 범위

- `mydocs/manual/mcp_hwp2024Convert_usage.md`에서 동기 CLI `convert`와 동기 MCP tool
  `convert_local_document`의 호출 방법을 제거했다.
- 모든 변환과 암호 문서 요청을 비동기 `start → status → download` 흐름으로만 안내한다.
- source, test, fixture, client artifact, server 설정은 변경하지 않았다.

## 절차와 검증

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, review_only_fast_pass.md, post_merge.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md,
  review_only_fast_pass.md, post_merge.md, local_validation.md
current head: ac033c3b2da0ba33c1e9e0bd7abd135da39997cc
```

- `git diff --check`를 통과했다.
- 변경 범위는 `mydocs/`의 사용 가이드 한 파일이다. Markdown link target을 추가·변경하지 않았고,
  Cargo·npm 검증은 문서 전용 변경이라 실행하지 않았다.
- renderer, layout, HWP/HWPX fixture, 기준 PDF는 변경하지 않아 시각 검증은 적용하지 않는다.

## 최종 판정

- 판정: 승인
- 근거: 동기 호출 예제·도구명·암호 전달·과거 사용 기록까지 제거했고, 비동기 요청의 시작·상태 확인·결과
  저장 순서는 유지했다.
- merge 전 조건: 최신 PR head의 required GitHub Actions가 모두 성공하고, mergeability와 head SHA를
  다시 확인한다.
- 원격 조치: 이 기록은 GitHub approve event를 남기지 않는다. 작업지시자가 승인한 PR 병합만 수행한다.
