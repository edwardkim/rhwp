---
kind: pr_review
status: local-validation-passed
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4361 검토 — MCP workspace 프로필·스캔 경계

## 라우팅과 접수

기본 경로는 `maintainer_general.md`, 보조 경로는 `intake_and_review.md`와
`local_validation.md`다. contributor code 위에 메인터너 source/test 보정을 추가했으므로
[implementation 기록](pr_4361_review_impl.md)을 함께 유지한다.

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4361](https://github.com/edwardkim/rhwp/pull/4361) / `kevin9327` |
| 관련 이슈 | [#4357](https://github.com/edwardkim/rhwp/issues/4357), #4351, #4352 |
| 원 base / head | `devel` / `c98a3f1101ba78d8cc4d87f3c6f906ab8cf632c1` |
| 원 변경 규모 | 5 files, +595/-11, contributor commits 2개 |
| 검토 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` — 원 head의 조상임을 확인 |
| 작성 시점 원격 참고 상태 | `MERGEABLE` / `CLEAN`, 원 head GitHub checks 성공. merge 전 재확인 필요 |
| 가시성 branch | `review/kevin9327-20260810-pr4361` |
| 메인터너 code head | `058acbe79aa5b45e32507a53a99aaac55d9d6fc3` |
| GitHub 상태 변경 | reviewer assign, comment, push, review, merge 모두 미수행 |

이번 작업지시는 로컬 메인터너 보정과 기록까지이며 GitHub mutation 승인을 포함하지 않는다. 따라서
접수 가이드의 reviewer assign도 실행하지 않았고, 이 문서의 원격 상태는 2026-08-10 조회 시점 참고값이다.

## contributor 변경 범위

원 PR은 `rhwp mcp-serve --workspace <dir>`의 결정론 인벤토리, workspace id 열기, 안정 ID 문서 트리,
변이 SHA-256 저널을 추가한다. 세션 도구 4종을 agent profile의 전체/조회 도구 목록과 MCP annotations
계약에 연결하고, workspace 계약 테스트 및 stage 기록을 더한다.

renderer, layout, WASM, sample, golden 및 fixture 변경은 없다. 문서를 열어 구조를 조회하지만 렌더 출력이나
조판 규칙을 바꾸지 않으므로 시각 검증 대상이 아니며 별도 PDF/SVG 증적을 만들지 않았다.

## 확인한 blocker

### 프로필 직접 호출 우회

`hwp_ws_list`, `hwp_ws_open`, `hwp_doc_tree`, `hwp_ws_journal`은 `tools/list`와 실행 match에는 추가됐지만
`is_session_tool()` 판별에서 빠졌다. 프로필이 이 도구들을 숨겨도 호출자가 이름을 직접 보내면 공통
`session_allows` gate를 건너뛸 수 있어, 프로필이 권한 경계가 아니라 추천 목록으로 약화됐다.

### workspace 링크 추적

스캐너가 `path.is_dir()`을 사용해 링크를 따라갔다. 디렉터리 링크는 같은 경로를 중복 방문하거나 cycle을
만들 수 있고, 파일 링크는 선택한 root 밖의 HWP/HWPX/HML을 인벤토리에 넣을 수 있었다. 파일 수 상한은
지원 문서를 발견할 때만 증가하므로 문서가 없는 cycle의 종료를 보장하지 못했다.

두 항목 모두 profile/workspace 경계를 깨므로 원 head 그대로는 merge blocker로 판정했다.

후속 독립 검토에서는 두 가지 검증·결정성 결함을 더 확인했다. 신규 CLI contract test가
`env!("CARGO_BIN_EXE_rhwp")`만 사용해 nextest archive의 런타임 재매핑 경로를 무시했고, workspace
스캐너는 파일시스템의 비결정적 `read_dir` 순서에서 먼저 10,000개를 자른 뒤 정렬했다. 따라서
10,000개 초과 workspace는 실행·파일시스템마다 서로 다른 subset과 `w1..` id를 낼 수 있었다.

## 메인터너 보정

commit `19fad2a44142b1c3f9c9dde300c6151d245080e8`
(`fix(mcp): enforce profile and workspace boundaries`)을 원 contributor head의 직계 자식으로 추가했다.

| 파일 | 보정 |
| --- | --- |
| `src/mcp_serve.rs` | 세션 도구 판별을 `agent_profiles::ALL_SESSION_TOOLS` 단일 출처에 연결했다. 스캔은 file type을 링크 비추적 방식으로 읽고 링크·비정규 파일을 제외하며, canonical root 안의 방문하지 않은 디렉터리만 순회한다. |
| `tests/agent_profile_router_contract.rs` | `데이터분석` 프로필에서 기존 `hwp_open`과 신규 workspace/tree/journal 4종의 직접 호출이 모두 profile gate 오류로 거절되는지 검증한다. |
| `tests/mcp_workspace_contract.rs` | Unix에서 root 밖 파일 링크와 디렉터리 링크를 만들고 실물 내부 문서 한 건만 인벤토리에 남는지 검증한다. |

contributor commits는 수정·squash·rebase하지 않았으며 위 1차 메인터너 commit을 별도로 추가했다.

후속 commit `058acbe79aa5b45e32507a53a99aaac55d9d6fc3`
(`fix(mcp): make capped workspace inventory deterministic`)은 기존 문서 commit 뒤에 선형으로 추가했다.

| 파일 | 후속 보정 |
| --- | --- |
| `src/mcp_serve.rs` | 경로순 최대 힙으로 가장 작은 10,000개 후보만 보존한다. 전체 순회 순서와 무관한 subset을 만들면서 후보 메모리는 상한 안에 둔다. |
| `tests/agent_profile_router_contract.rs` | 모든 신규/수정 CLI 기동을 nextest 런타임 `CARGO_BIN_EXE_rhwp` 우선 helper로 통일했다. |
| `tests/mcp_workspace_contract.rs` | 같은 runtime helper를 쓰고, root에서 상한을 채운 뒤 더 작은 중첩 경로가 나오는 10,001파일 fixture로 결정적 subset을 고정했다. |

contributor와 기존 메인터너 commit은 amend·rebase하지 않았다.

## 완료한 검증

| 게이트 | 결과 |
| --- | --- |
| `$env:CARGO_INCREMENTAL='0'; cargo test --profile release-test --target-dir "$env:TEMP\rhwp-pr4361-target" --test agent_profile_router_contract --test mcp_workspace_contract` | 12/12 통과: profile 8, workspace 4 |
| 10,001파일 상한 회귀 | `truncated:true`, count 10,000, `a/00000.hwp`가 `w1`, 경로순 마지막 `z09999.hwp` 제외 확인 |
| nextest binary 경로 | 두 수정 test 파일의 모든 직접 `Command`가 런타임 env 우선 helper 사용 |
| 신규 숨김 도구 직접 호출 계약 | 5종 모두 공통 profile gate에서 거절됨을 확인 |
| 신규 Unix file/directory symlink 계약 | 현재 Windows host에서는 `cfg(unix)`로 미실행; Linux 후보 CI에서 실행 필요 |
| `rustfmt --check --edition 2021 src/mcp_serve.rs tests/agent_profile_router_contract.rs tests/mcp_workspace_contract.rs` | 통과 |
| `git diff --check origin/pr/4361..058acbe7` | 통과 |

Windows의 긴 checkout 경로 때문에 `cargo fmt --all`은 OS error 206으로 실행되지 않아, 변경 Rust 파일을
동일 rustfmt와 edition으로 직접 검사했다. focused Cargo 실행의 bin/lib PDB 이름 충돌 경고는 기존 target
명명에서 발생했으며 테스트 결과에는 영향을 주지 않았다.

## 잔여 위험

- Unix symlink 회귀는 현재 Windows host에서 실행되지 않았다. 원격 후보를 만들면 Linux GitHub Actions의
  실제 통과가 필수다.
- Windows junction은 권한 독립적인 fixture를 만들기 어려워 별도 자동 테스트가 없다. 구현은 canonical
  root containment와 방문 집합으로 junction이 directory로 분류되더라도 root 이탈·재방문을 차단한다.
- 결정적 subset을 위해 상한을 넘은 뒤에도 디렉터리 순회는 끝까지 계속한다. 후보 저장은 최대 10,000개로
  제한되지만 매우 큰 tree의 전체 순회 시간·metadata I/O는 별도 총량/시간 예산 없이 입력 크기에 비례한다.
- 스캔 뒤 실제 open 전까지 workspace 내용을 적대적으로 교체하는 동시 mutation은 v1 단일 클라이언트
  전제 밖이다. 이번 보정은 스캔 시점의 링크·cycle·root 경계를 고정한다.
- source/test를 추가한 local head이므로 원 contributor head의 기존 녹색 CI를 메인터너 보정의 근거로
  재사용하지 않는다. push 승인이 나면 새 head 전체 required checks가 필요하다.

## 조건부 권고

**확인한 로컬 blocker는 메인터너 보정으로 해소되어 조건부 통합 권고다.** 다만 현재 branch는 로컬 전용이며
push 또는 merge 승인이 없다. 작업지시자가 반영 경로를 승인한 뒤 Linux symlink 계약을 포함한 새 원격
head required checks가 모두 통과하고, 별도의 명시적 merge 승인이 있을 때만 통합할 수 있다.
