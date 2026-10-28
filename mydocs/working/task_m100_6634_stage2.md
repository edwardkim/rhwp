---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6634.md
issue: 6634
last_verified: 2026-09-05
---

# #6634 Stage 2 완료보고 — 직접 호출과 exact release source guard

## 수행 결과

`Publish All Packages`를 `release.published` 이벤트에 의존하는 workflow에서 같은 저장소의 reusable
workflow로 전환했다. `Release Binary`는 다섯 플랫폼 build와 Release 자산 첨부가 모두 성공한 뒤 같은
workflow commit의 package workflow를 직접 호출한다.

수동 `Release Binary` 실행은 다섯 플랫폼 산출물과 reusable package build를 검증하되 Release job을
`skipped` 처리한다. 따라서 수동 검증 경로에는 `contents: write` 토큰이 발급되지 않고 외부 publish 입력도
기본값 `false`다. production stable tag 경로에서만 Release 성공을 조건으로 publish 입력이 `true`가 된다.

권한과 secret 경계는 다음과 같이 제한했다.

- workflow 기본 권한은 `contents: read`다.
- GitHub Release를 수정하는 job에만 `contents: write`를 부여한다.
- npm publish job에만 `id-token: write`를 부여해 Trusted Publishing을 유지한다.
- caller는 `secrets: inherit` 대신 `VSCE_PAT`, `OVSX_PAT` 두 이름만 명시적으로 전달한다.
- verify-only 경로의 validation·WASM build job에는 publish secret과 OIDC 권한이 없다.

## exact source 보호

`scripts/release_publish_guard.py`는 외부 상태를 읽는 부분과 결정적 판정 함수를 분리했다. production
publish 전에 다음 조건을 모두 검증하며 하나라도 불일치하면 exit 1로 실패한다.

1. checkout commit과 `GITHUB_SHA`가 같은 40자리 commit SHA인가.
2. 실행 ref가 실제 tag이고 `refs/tags/<ref_name>`과 일치하는가.
3. tag가 현재 Cargo version의 `v<version>`이며 그 tag commit이 `GITHUB_SHA`와 같은가.
4. Cargo, `@rhwp/editor`, VS Code extension version이 모두 같은가.
5. 같은 tag의 GitHub Release가 게시됐고 draft·prerelease가 아닌가.

직접 수동 복구도 움직이는 branch가 아니라 exact tag ref에서 `publish=true`로 실행해야 같은 guard를
통과한다. 반면 안전한 기본 `publish=false`는 branch에서도 build 검증을 허용한다.

## 검증 결과

| 검증 | 결과 |
| --- | --- |
| #6634 orchestration·guard 계약 | 18건 중 15 PASS, 후속 경계 3 RED |
| exact source guard 단위 계약 | 6건 모두 PASS |
| 기존 release channel·promotion 회귀 | 43건 모두 PASS |
| 실제 현재 HEAD verify-mode CLI | `accepted=true`, error 0 |
| 두 workflow YAML parse | 성공 (`release-binary` 3 jobs, `npm-publish` 5 jobs) |
| `git diff --check` | 성공 |
| 변경 문서 상대 링크 | 4개 문서, 이상 없음 |
| 문서 metadata | 신규 오류 0; 저장소 기존 문서 4개의 baseline 오류 16건 유지 |
| actionlint | 로컬 실행 파일이 없어 미실행; Stage 5 Actions 검증 대상으로 유지 |

남은 RED 3건은 오류 회피가 아니다. Stage 3의 VS Code Marketplace/Open VSX 독립 job, 명시적 publish
완료 집계와 Stage 4의 두 workflow promotion policy 등록을 각각 가리킨다. Stage 2에서 미리 GREEN으로
위장하지 않고 승인된 단계 경계를 유지했다.

## 범위와 안전

- Rust 제품 source, renderer, WASM API와 package version은 변경하지 않았다.
- workflow dispatch, tag, Release, npm·Marketplace·Open VSX publish는 수행하지 않았다.
- secret 값은 조회·출력·fixture 저장하지 않았다.
- Release job을 production tag push로 한정해 verify-only 실행에서 쓰기 권한이 생기지 않도록 했다.
- 채널 상태 판정과 부분 재시도는 아직 구현하지 않았다. 현재 combined extension job을 실제 production에
  적용하기 전에 Stage 3을 완료해야 한다.

## 다음 게이트

메인테이너가 Stage 2 결과와 15 GREEN/3 후속 RED를 승인하면 Stage 3 channel idempotency·부분 재시도
구현에 진입한다.
