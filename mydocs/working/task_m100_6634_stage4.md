---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-05
---

# #6634 Stage 4 완료보고 — promotion 정책과 배포 운영 절차

## 수행 결과

`scripts/workflow_promotion_policy.json`에 Release Binary와 Publish All Packages를 `verify-only` 실행
대상으로 등록했다. promotion은 외부 package를 게시하지 않고 같은 exact `devel` SHA의 build·호출·artifact
경로를 실행해야 한다.

Release Binary 수동 검증은 다음 증적을 요구한다.

- Linux x86_64/AArch64, macOS x86_64/AArch64, Windows x86_64 build 5개 성공
- reusable package의 source validation, WASM, VSIX, aggregate 성공
- GitHub Release job과 npm·VS Code·Open VSX publish job 4개 skipped
- CLI archive 5개, `wasm-pkg`, `vscode-vsix`, `release-publish-evidence` artifact 존재
- evidence 안의 `verdict=completed`

Publish All Packages 직접 수동 검증은 source validation, WASM, VSIX, aggregate 성공과 외부 publish job
4개의 skip, package artifact 3개와 `completed` verdict를 요구한다. 두 정책 모두 허용 event는
`workflow_dispatch`, 허용 actor는 `edwardkim`이다.

단순 artifact 이름만 확인하지 않도록 Stage 3 evidence에 문자열 `verdict`를 추가했다. 성공 집계는
`completed`, 하나라도 실패하면 `failed`이며 #6689 collector가 artifact ZIP 안의
`release-publish-evidence.json`을 직접 읽어 판정한다.

## 운영 매뉴얼 현행화

[GitHub 저장소 운영 매뉴얼](../manual/github_operations.md)은 promotion 실행 표면에 다음 두 명령을
추가했다.

```bash
gh workflow run release-binary.yml --ref devel -f tag=test
gh workflow run npm-publish.yml --ref devel -f publish=false -f publish_extensions=true
```

두 명령은 원격 run mutation이므로 실행 전 메인테이너 승인이 필요하다. verify-only 성공은 정식 tag의
실제 외부 publish를 대신하지 않으며 다음 release에서 production evidence를 별도로 확인한다.

[배포 가이드](../manual/publish_guide.md)는 다음과 같이 정정했다.

1. 자동 기동 원인은 GitHub Release의 `published` 이벤트가 아니라 stable tag의 Release Binary 직접 호출이다.
2. tag push 직후 `gh release create`를 경쟁 실행하지 않고 Release job 성공 뒤 `gh release edit`으로 본문을 반영한다.
3. package workflow는 tag/SHA/version/Release 검증 뒤 네 채널을 독립 조회·게시한다.
4. 수동 복구는 branch가 아니라 exact release tag에서 `publish=true`로 실행한다.
5. 기게시 채널은 `already-present`로 skip되며 조회 실패는 부재로 간주하지 않는다.
6. `release-publish-evidence`의 SHA, channel state, `completed` verdict를 공개 상태와 함께 확인한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| #6634 orchestration·channel·promotion 계약 | 67건 모두 PASS |
| 기존 release channel·promotion·wiring 회귀 | 47건 모두 PASS |
| `upstream/devel → 후보` promotion inventory | executable 2개, policy violation 0 |
| inventory 실행 모드 | 두 workflow 모두 `verify-only` |
| policy·fixture JSON과 Python 구문 | 성공 |
| 두 workflow YAML parse | 성공 |
| 변경 문서 상대 링크 | 8개 문서, 이상 없음 |
| 문서 metadata | 신규 오류 0; 저장소 기존 문서 4개의 baseline 오류 16건 유지 |
| `git diff --check` | 성공 |

Stage 3까지 남아 있던 promotion policy RED 1건이 GREEN으로 전환돼 #6634의 로컬 목표 계약은 모두
구현됐다. 실제 Actions job 이름과 artifact 생산은 Stage 5 exact-head 비게시 run에서 다시 판정한다.

## 범위와 남은 위험

- GitHub workflow dispatch, push, tag, Release와 외부 publish는 수행하지 않았다.
- 정책의 reusable job 이름은 기존 GitHub API naming 규칙인 `<caller job id> / <called job name>`으로
  지정했다. Stage 5 실제 run에서 한 글자라도 다르면 policy와 문서를 추정으로 완화하지 않고 실제 API
  이름에 맞춰 정정한다.
- 로컬 `actionlint`가 없어 Actions 표현식·reusable workflow의 최종 등록 검증은 Stage 5에 남아 있다.
- `main`에는 현재 `devel`에 없는 release hotfix commit이 있으므로 Stage 6 통합 전에 최신 topology를
  재조회하고 main→devel 동기화 조건을 해결해야 한다.

## 다음 게이트

메인테이너가 Stage 4 결과를 승인하면 Stage 5 로컬 package 산출물 검증을 먼저 수행한다. 그 뒤 별도
승인을 받아 exact task/devel head의 두 verify-only workflow를 원격 실행하고 job·artifact·verdict를
수집한다.
