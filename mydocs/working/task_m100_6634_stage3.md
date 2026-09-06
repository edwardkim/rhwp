---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-05
---

# #6634 Stage 3 완료보고 — 채널별 멱등성과 부분 재시도

## 수행 결과

`Publish All Packages`의 외부 게시 경로를 npm core, npm editor, VS Code Marketplace, Open VSX 네 독립
job으로 나눴다. 각 job은 manifest의 이름·version이 기대 배포 대상과 일치하는지 확인하고 공개 registry의
exact version을 먼저 조회한다.

- exact version이 있으면 publish secret을 사용하는 step을 실행하지 않고 `already-present`로 성공한다.
- 없으면 해당 채널만 게시하고 명령 성공 뒤 `published`로 기록한다.
- timeout, 5xx, JSON 손상, identity 불일치는 “없음”으로 바꾸지 않고 job을 실패시킨다.
- 동일 repository/ref의 중복 실행은 `cancel-in-progress: false` concurrency group으로 직렬화한다.
- 비게시 mode에서는 외부 채널 job을 건너뛰고 aggregate가 `verify-only`로 기록한다.

VS Code extension은 WASM을 받은 `build-vsix` job에서 한 번만 compile·package한다. Marketplace와 Open VSX
job은 같은 `vscode-vsix` artifact를 내려받아 독립적으로 상태를 확인하고 필요한 채널만 게시한다. 따라서
한 확장 채널의 성공이 다른 채널의 실패 뒤 재실행을 막지 않는다.

## 상태 조회와 증적 계약

`scripts/release_channel_status.py`는 npm registry, Visual Studio Marketplace extension query, Open VSX의
공개 응답을 exact-version boolean으로 해석한다. HTTP 404는 npm·Open VSX에서만 정상적인 부재이며,
Marketplace의 정상 빈 결과만 부재다. 그 밖의 HTTP·전송·schema 오류는 실패-폐쇄다.

`scripts/release_publish_evidence.py`는 source validation, WASM build, VSIX build와 네 채널 job 결과를
하나의 JSON 및 job summary로 집계한다. 요청된 채널은 `already-present | published`만 성공으로 인정하며,
build 실패·조회 실패·출력 누락은 aggregate 자체를 실패시킨다. 결과 artifact에는 commit, ref, channel,
상태만 있으며 token·인증 URL은 없다.

[상태 fixture](../tech/investigations/issue-6634/release_channel_status_cases.json)는 다음 재시도 상태를 고정한다.

1. 네 채널 모두 미게시였다가 게시됨
2. 네 채널 모두 이미 게시됨
3. VS Code Marketplace만 먼저 성공한 뒤 나머지를 재시도
4. Open VSX만 먼저 성공한 뒤 나머지를 재시도
5. registry 조회 실패
6. 외부 게시 없는 verify-only 실행

## RED에서 GREEN으로의 전환

Stage 3 착수 RED에서는 조회·집계 source 2개가 없었고, VSIX 단일 build·독립 채널·완료 집계·concurrency
workflow 계약이 모두 실패했다. fixture test 두 그룹은 구현 파일 부재로 skip됐다. 첫 실행은 네 채널
subtest를 개별 실패로 계산해 runner 표시에 9 failure, 2 skip으로 기록됐다.

구현 뒤 결과는 다음과 같다.

| 검증 | 결과 |
| --- | --- |
| Stage 3 상태·부분 재시도·workflow 계약 | 10건 모두 PASS |
| #6634 원인·호출·guard·채널 계약 | 18건 중 17 PASS, Stage 4 경계 1 RED |
| 기존 release channel·promotion·wiring 회귀 | 46건 모두 PASS |
| Python·fixture JSON 구문 | 성공 |
| `npm-publish.yml` YAML parse | 성공, 8 jobs |
| 변경 문서 상대 링크 | 5개 문서, 이상 없음 |
| 문서 metadata | 신규 오류 0; 저장소 기존 문서 4개의 baseline 오류 16건 유지 |
| `git diff --check` | 성공 |

남은 RED는 `release-binary.yml`과 `npm-publish.yml`이 #6689 workflow promotion policy에 아직 등록되지
않았다는 한 건뿐이다. 이는 Stage 4 범위이며 채널 구현 실패가 아니다.

## 공개 상태 read-only 교차 확인

2026-09-05에 새 조회기를 실제 공개 endpoint에 read-only로 실행했다. `@rhwp/core@0.8.6`,
`@rhwp/editor@0.8.6`, `edwardkim.rhwp-vscode@0.8.6`의 VS Code Marketplace와 Open VSX가 모두
`already-present`로 판정됐다. token을 사용하지 않았고 게시 명령은 실행하지 않았다.

## 범위와 남은 위험

- Rust 제품 source, package version, 외부 공개 상태는 변경하지 않았다.
- secret은 해당 채널이 실제 미게시일 때의 publish step에만 연결했다.
- extension packaging CLI의 실제 VSIX 생성과 비게시 Actions artifact 검증은 Stage 5에서 수행한다.
- 로컬에 `actionlint`가 없어 Actions 문법 실검증은 Stage 5 exact-head 원격 run까지 보류한다.
- workflow dispatch, 원격 push, Release·npm·Marketplace·Open VSX publish는 수행하지 않았다.

## 다음 게이트

메인테이너가 Stage 3 결과를 승인하면 Stage 4 workflow promotion policy와 운영 매뉴얼 현행화에 진입한다.
