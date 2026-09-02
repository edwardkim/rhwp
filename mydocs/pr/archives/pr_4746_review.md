---
kind: pr-review
status: active
issue: 4089
pr: 4746
---

# PR #4746 리뷰 — Windows Docker WASM 빌드 격리

## 라우팅과 접수

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, review_only_fast_pass.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  collaborator_self_merge.md, intake_and_review.md, local_validation.md,
  review_only_fast_pass.md
current head: a9dcecdac16d2a713e25fe0e15463d51e5d59608 (문서 작성 시점 참고값)
```

| 항목 | 값 |
| --- | --- |
| PR | [#4746](https://github.com/edwardkim/rhwp/pull/4746) |
| 작성자 | `jangster77` (collaborator) |
| 대상 / head | `devel` ← `codex/issue-4089-wasm-docker` |
| 코드 후보 | `a9dcecdac` — `fix: #4089 Windows Docker WASM 빌드 격리` |
| 규모 | 6 files, +208 / -3, 2 commits (작성 시점) |
| mergeable | `MERGEABLE`, `mergeStateStatus=BLOCKED` (CI 진행 중인 작성 시점 참고값) |
| 관련 이슈 | [#4089](https://github.com/edwardkim/rhwp/issues/4089), assignee `jangster77`, OPEN |

작업지시자의 collaborator self-review 지시에 따라 외부 reviewer를 요청하지 않았다. 이 문서는
자체 검토 기록이며 GitHub approval이나 merge 권한 행사를 뜻하지 않는다.

## 변경과 범위 판정

`docker-compose.yml`의 `wasm` 서비스는 Cargo target을 `wasm-target:/build-target` named volume으로
옮긴다. 중단된 wasm-pack의 `pkg/*-opt.wasm`만 build 전 제거하고, HUP/INT/TERM·일반 종료 시
`pkg/` ownership을 compose `UID`/`GID`로 복원한다. 개발 가이드는 Docker 표준 경로와 Windows
네이티브 `--no-opt` 진단 경계를 설명하며, Python 계약 테스트가 이 구성을 고정한다.

Rust renderer·WASM API·frontend 출력·fixture·golden·CI workflow는 변경하지 않았다. 따라서
render/fixture 시각 증적 보조 경로는 적용하지 않는다. Docker runtime과 `wasm-pack` 실행 경로는
바뀌므로 문서만 변경한 PR은 아니며, trailing review 기록을 push한 뒤에는 code 후보 `a9dcecdac`의
최신 GitHub CI와 fast-pass 조건을 분리해 확인한다.

## 검증과 자체 검토

- `python scripts\\tests\\test_docker_wasm_compose.py`를 실행해 4/4 통과했다. named target volume,
  stale `*-opt.wasm` 정리 순서, signal/EXIT ownership 복원, Docker 우선 가이드와 `--no-opt`
  진단 경계를 확인했다.
- `git diff --check upstream/devel...a9dcecdac`를 실행해 통과했다.
- `upstream/devel@c121f6185`에서 `a9dcecdac`을 `--no-commit --no-ff`로 merge simulation했다.
  자동 병합 뒤 `git diff --check`가 통과했고, merge를 abort한 뒤 임시 `pr4746-merge-test` branch를
  삭제했다.
- 이 Windows host에는 Docker CLI와 `.env.docker`가 없고, host `wasm-pack`은 정책 고정판 0.15.0이
  아닌 0.14.0이다. 따라서 실제 Docker Compose build, named-volume cache 재사용, optimized WASM
  산출물은 통과로 기록하지 않는다.

코드 후보 SHA에서 CI preflight는 성공했고 CI lint/frontend-package와 CodeQL은 문서 작성 시점에
진행 중이다. 최신 head의 CI 결과는 merge 직전에 다시 확인해야 하며, 후보 뒤 review-only commit의
aggregate도 별도로 성공해야 한다.

## 기준선 갱신과 최신 CI

초기 review 기록 뒤 GitHub update branch가 `devel@be7dabdd1`을 source branch에 병합해 새 head
`43117ad38414680549c6e0595fd1e8fd62e7eda6`을 만들었다. 이 merge는 #4745의 변경을 기준선으로
가져온 것이며, #4089의 고유 diff는 최신 `upstream/devel...HEAD`에서 다시 분리해 확인했다. 이전
`b1ce261a` head의 진행 중 CI는 최종 근거로 재사용하지 않았다.

- 최신 head에서 `python scripts\\tests\\test_docker_wasm_compose.py`를 다시 실행해 4/4 통과했고,
  `git diff --check upstream/devel...HEAD`도 통과했다.
- [CI run 31736384129](https://github.com/edwardkim/rhwp/actions/runs/31736384129)의 Build & Test,
  Lint, frontend package, Native Skia, 모든 default-feature shard가 성공했다. frontend unit과
  WASM Build의 skip은 preflight 분류에 따른 정상 상태다.
- [CodeQL run 31736383870](https://github.com/edwardkim/rhwp/actions/runs/31736383870)의 Rust,
  JavaScript/TypeScript, Python 분석과 aggregate가 성공했다.

이 commit은 review·오늘할일만 갱신한다. `43117ad3`의 녹색 code candidate 뒤에 단일 review-only
commit으로 이어지므로, push 뒤에는 최신 head의 preflight/Build & Test aggregate를 다시 확인한다.

## 위험과 권고

Compose 실행은 root로 named volume을 초기화하되 `pkg/`만 사용자 UID/GID로 반환한다. 실제 Docker
Desktop Windows host에서 표준 명령을 두 번 순차 실행해 hard-link 회피와 cache 재사용을 확인하기 전에는
runtime 보장을 주장하지 않는다.

**권고: 보류.** 최신 code candidate의 GitHub CI는 성공했다. 다만 review-only trailing head의
aggregate, Docker Desktop Windows 실실행 확인, 그리고 작업지시자 승인이 모두 갖춰진 뒤에만 merge
판단으로 전환한다. #4089은 merge 및 검증 완료 전까지 닫지 않는다.
