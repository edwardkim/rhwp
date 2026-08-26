---
kind: pr-review
status: trailing-docs-pending-ci
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-24
---

# PR #5957 self-review — Windows HWP MCP 클라이언트 0.9.0

## 라우팅과 접수 메타데이터

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `review_only_fast_pass.md`, `post_merge.md`
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서
- 작성자 본인 self-review이므로 reviewer를 지정하지 않는다.
- code candidate: `7205bf66e0c7cb30415d2d970c17d6972069cb11`

| 항목 | 작성 시점 참고값 |
| --- | --- |
| PR / 작성자 | [#5957](https://github.com/edwardkim/rhwp/pull/5957) / [@jangster77](https://github.com/jangster77) |
| base / head | `devel` / `codex/hwp2024-mcp-client-20260824` |
| 규모 | 9 files, +172 / -50, 1 code candidate commit |
| 상태 | Open, non-draft; trailing commit 뒤 최신 상태 재확인 필요 |
| 관련 issue | 없음 |

## 변경 범위와 판단

- `hwp-convert-mcp-2024-client`를 server 계약 0.9.0에 맞춰 갱신하고
  `tools/hwp-convert-mcp-2024-client-20260824-011002.tar.gz`로 교체했다.
- MCP tool 4개에 `engine: 2020|2024`와 선택적 `password` 입력을 노출했다. CLI는
  `--engine 2020|2024`와 현재 사용자만 읽을 수 있는 단일 행 `--password-file`을 사용한다.
- engine `2020`은 기본 `-hwp2020` 출력명을 사용한다. 기본 engine `2024`, blob upload, 비동기
  `start → status → download`, atomic publish와 byte 수·SHA-256 검증 계약은 유지했다.
- 별도 Linux HWP 2020 beta 경로는 역사·재현 전용으로 `superseded` 처리하고, 2022 이하 저장본은
  통합 Windows service의 engine `2020`, 2024 저장본은 engine `2024`를 사용하도록 사용자 문서를 정렬했다.
- package는 Node.js 표준 라이브러리만 사용하며 SDK·Zod·외부 runtime dependency·`node_modules`가 없다.
  server URL, 인증 token, 문서 password는 artifact·문서·Git 변경에 포함하지 않았다.

이 PR은 Rust source, renderer, HWP/HWPX fixture, 기준 PDF, workflow 또는 배포 server를 변경하지 않는다.
따라서 visual sweep은 적용하지 않았다. local mock 검증은 client wire 계약을 확인하지만 배포 server 반영을
증명하지 않으므로 실제 배포와 engine별 운영 smoke는 후속 배포 작업의 별도 gate다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| 세 `.mjs`의 `node --check` | 모두 exit code 0 |
| 실제 tarball의 CLI help | exit code 0, `--engine`·`--password-file` 확인 |
| 실제 tarball의 stdio initialize / `tools/list` | version `0.9.0`, tool 4개, engine·password schema 확인 |
| 실제 tarball CLI → local mock HTTP MCP 비동기 시작 | `queued`, 유효 UUID, engine `2020` 확인 |
| password-file 비노출 | mock server 전달 일치, result key `engine,job_id,status`만 존재 |
| archive 검사 | 8,558바이트, SHA-256 `830b1f7ff3696a9b499d0043e48e3dccfba7817797a698003bd909226f13cf72`, dependency·`node_modules`·비밀정보 0건 |
| 변경 Markdown 링크·metadata | 링크 오류 0건, 변경 장기 문서 6개 metadata 오류 0건 |
| Rust suite manifest prepare/check | 모두 exit code 0 |
| `cargo fmt --all` / `cargo fmt --all -- --check` | 모두 exit code 0 |
| `git diff --check` | exit code 0 |

전체 저장소 metadata 검사는 이번 diff 밖의 기존 `mydocs/tech` 4개에서 front matter 누락 16건을 보고했다.
변경 문서만 같은 validator로 검사한 결과는 오류 0건이므로 이 PR에서 관련 없는 기존 문서를 수정하지 않았다.
Rust source가 없고 client package 계약을 실제 archive·mock transport로 검증했으므로 로컬 전체 Rust 회귀는
수행하지 않았다.

## GitHub Actions

code candidate `7205bf66e0c7`의 [CI run 32652426344](https://github.com/edwardkim/rhwp/actions/runs/32652426344)는
Lint, Native Skia, Frontend package, test archive builder·worker와 최종 Build & Test가 모두 성공했다.
[CodeQL 32652426236](https://github.com/edwardkim/rhwp/actions/runs/32652426236),
[Proptest 32652426244](https://github.com/edwardkim/rhwp/actions/runs/32652426244),
[Adapter inter-diff 32652426219](https://github.com/edwardkim/rhwp/actions/runs/32652426219)도 같은 SHA에서
성공했다. 정책상 WASM Build와 Frontend unit gates의 skip 외에 실패한 check는 없다.

현재 self-review·오늘할일은 이 녹색 code candidate 뒤에 추가하는 `mydocs/` 한정 single-parent trailing
commit이다. push 뒤 review-only fast-pass가 candidate를 재사용하는지, 최신 aggregate가 성공하는지,
최신 `MERGEABLE/CLEAN`과 head SHA가 일치하는지 다시 확인해야 한다.

## 최종 권고

client artifact, engine 라우팅, password 전달과 비노출, 문서 선택 규칙이 같은 0.9.0 계약을 가리키며
추가 blocker는 발견하지 않았다. self-review는 **완료 / 조건부 merge 권고**다. 작업지시자는 최신 trailing
CI 성공 뒤 merge와 후속 처리를 자동 진행하도록 승인했으며, merge 직전 상태 게이트는 그대로 적용한다.
