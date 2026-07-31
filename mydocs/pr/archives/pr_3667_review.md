---
kind: review
status: active
canonical: mydocs/pr/archives/pr_3667_review.md
last_verified: 2026-07-31
---

# PR #3667 리뷰 기록

## 라우팅

```text
base route: collaborator self-merge
modifiers: intake_and_review.md, local_validation.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md
current head: 3a4325d4f4b1b9313e4465fa88c22a729081d049 (작성 시점 참고)
```

시각·fixture 증적은 적용하지 않는다. 이 PR은 rhwp renderer/layout, HWP/HWPX sample, 기준 PDF를 바꾸지 않고
외부 HWP 2020 MCP client tarball과 사용법만 갱신한다.

## PR metadata

| 항목 | 값 |
| --- | --- |
| PR | #3667 |
| 작성자 | `@jangster77` |
| base / head | `devel` / `task_m100_3604` |
| head | `3a4325d4f4b1b9313e4465fa88c22a729081d049` (작성 시점 참고) |
| 변경 규모 | 7 files, +198 / -37 (작성 시점 참고) |
| reviewer | `@edwardkim` 요청 완료 |
| 관련 issue | #3604. 이 PR은 issue를 닫지 않는다. |

## 변경 범위

- 이전 MCP client tarball을 삭제하고 `hwp-convert-mcp-client-20260731-230819.tar.gz` 하나를 추가한다.
- `mcp_hwp2020Convert_usage.md`에 사용자 home 경로, 암호 HWP3/HWP5 HWP 및 ODF 암호 HWPX 지원 방향,
  비동기 job lifecycle, 응답 전달과 정리 기준을 기록한다.
- client tarball은 `.env.local`을 포함하지 않고 `.env.local.example`만 포함한다. token, endpoint, 문서
  비밀번호, server 내부 경로는 PR diff와 본문에 넣지 않았다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| tarball manifest | `kind=client`, source `4d67020`, Hancom runtime 미포함 확인 |
| 비밀 파일 | `.env.local` 미포함, `.env.local.example` 포함 확인 |
| `hwp2020-mcp-convert --help` | `npx --package=file:`로 성공 |
| `hwp2020-mcp-bridge --help` | `npx --package=file:`로 성공 |
| archive 개수 | `tools/`에 client tarball 한 개 |
| 문서 검사 | 변경 manual 링크, 전체 문서 metadata, `git diff --check` 성공 |
| 실제 변환 근거 | 운영 MCP 암호 문서 8방향에서 client/server SHA-256 일치 및 `response_finished` 확인 |

Cargo 검증은 생략했다. rhwp Rust source·Cargo metadata·sample·renderer 변경이 없고, local 검증은 external
client artifact의 manifest와 실제 실행 가능한 entrypoint를 대상으로 했다.

## CI와 권고

작성 시점의 CI preflight는 queued, CodeQL preflight는 in progress이며 `mergeStateStatus`는 `BLOCKED`다.
이는 최종 상태가 아니다. 최종 merge 조건은 최신 PR head의 required CI 성공, requested review 완료,
작업지시자 merge 승인이다. 조건 충족 전 권고는 **보류**다.
