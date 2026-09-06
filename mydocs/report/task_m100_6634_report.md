# Task M100 #6634 — Release 이후 package publish 자동 기동 정상화 최종 보고서

- **이슈**: [#6634](https://github.com/edwardkim/rhwp/issues/6634)
- **브랜치**: `task_m100_6634`
- **최신 devel 기준**: `upstream/devel@ff1ce007b428547da74e0d6b7e9a196592c60ff6`
- **exact 원격 실증 head**: `559edb06826e7b8bfa2348d5951d78cf18d066e9`
- **Stage 5 증적 기록 head**: `5d0fc4ed672f62bb8685157e2879dea4a47274d0`
- **보고일**: 2026-09-06 KST
- **판정**: `stage5-qualified; ready-for-pr-preparation`

## 1. 결론

GitHub Release의 `published` 이벤트에 간접 의존하던 package 배포를 제거하고, stable tag의
`Release Binary`가 다섯 플랫폼 build와 Release 게시 성공 뒤 같은 commit의 reusable
`Publish All Packages`를 직접 호출하도록 바꿨다.

production 배포는 승인된 stable tag·Cargo/package version·게시된 Release·checkout SHA가 모두 일치할
때만 열린다. npm core/editor, VS Code Marketplace, Open VSX는 exact version을 독립 조회해 이미 게시된
채널을 건너뛰고 미게시 채널만 재시도한다. 외부 조회 오류는 “미게시”로 완화하지 않고 실패-폐쇄한다.

두 workflow의 exact-head 비게시 실행은 5개 native archive, WASM, VSIX와 구조화 evidence를 모두 만들었고,
Release와 외부 publish job은 전부 skip됐다. #6689 offline verifier는 정정 policy에서 오류 0건,
waiver 0건, `ok=true`로 두 run을 수락했다. 공개 네 채널과 Git tag·Release는 바뀌지 않았다.

## 2. 원인 계보

v0.8.0~v0.8.6의 실제 Release·Actions 시간축을 다시 수집해 세 결함을 확정했다.

1. v0.8.4와 v0.8.6은 workflow의 `GITHUB_TOKEN`으로 draft Release를 게시했다. 이 토큰이 만든
   `release.published` 이벤트는 새 workflow를 기동하지 않아 `Publish All Packages` run이 생기지 않았다.
2. v0.8.0~v0.8.3에서는 package run이 binary attachment보다 먼저 시작했다. 간접 이벤트가 우연히
   기동되더라도 다섯 플랫폼 binary 성공을 선행 조건으로 보장하지 못했다.
3. v0.8.6 수동 복구는 release tag commit이 아니라 후속 `main` hotfix에서 실행됐다. 공개 package의
   source provenance가 승인된 release tag와 달라질 수 있었다.

정규화한 원본과 해설은 다음 파일에 보존했다.

- [release_publish_lineage.json](../tech/investigations/issue-6634/release_publish_lineage.json)
- [release_publish_causal_lineage.md](../tech/investigations/issue-6634/release_publish_causal_lineage.md)

## 3. 구현 결과

### 3.1 결정적 직접 호출

- `npm-publish.yml`에 `workflow_call`과 안전 기본값의 `workflow_dispatch`를 함께 둔다.
- stable `v*` tag push의 `Release Binary`만 Release 성공 뒤 `publish=true`로 reusable workflow를 호출한다.
- manual `Release Binary(tag=test)`는 native와 package 산출물을 검증하지만 Release attach와 외부 publish는
  실행하지 않는다.
- reusable 호출은 로컬 workflow 경로를 사용하므로 caller와 같은 commit의 workflow가 실행된다.

### 3.2 exact release source guard

`scripts/release_publish_guard.py`는 production publish 전에 다음을 모두 검증한다.

- checkout commit과 `GITHUB_SHA`
- 실제 tag ref와 `refs/tags/<name>`
- Cargo version과 `v<version>` tag
- Cargo, `@rhwp/editor`, VS Code extension version
- tag commit과 실행 SHA
- 같은 tag의 게시된 non-draft, non-prerelease GitHub Release

branch에서 `publish=true`를 요청하거나 위 값 하나라도 다르면 게시 전에 실패한다. 수동 복구도 움직이는
`main`·`devel`이 아니라 exact release tag에서만 허용된다.

### 3.3 채널별 멱등성과 부분 재시도

`scripts/release_channel_status.py`가 공개 registry에서 exact version을 읽는다. 네 publish job은 독립적이며,
기게시 version은 `already-present`, 새 게시 성공은 `published`로 기록한다. timeout, 5xx, JSON·identity
불일치는 부재로 해석하지 않는다.

VSIX는 한 번만 만들고 Marketplace와 Open VSX가 같은 artifact를 사용한다. 어느 한 확장 채널이 실패해도
재실행에서는 성공한 채널을 건드리지 않고 남은 채널만 처리할 수 있다. 동일 repository/ref의 중복 실행은
취소하지 않고 직렬화한다.

### 3.4 증적과 promotion

`scripts/release_publish_evidence.py`는 source guard, WASM, VSIX와 네 채널 상태를 JSON과 job summary로
집계한다. 요청된 채널은 `already-present | published`만 성공이며, verify-only에서는
`skipped/verify-only`만 허용한다. token이나 인증 URL은 artifact에 넣지 않는다.

두 workflow를 #6689 promotion policy의 `verify-only` 대상으로 등록했다. 최초 실제 run에서 reusable job의
REST 이름이 YAML job ID가 아니라 caller 표시명을 접두사로 사용한다는 사실을 발견했다. policy를 실제
`Publish packages after binary release / ...` 이름으로 정정하고 정확한 success·skip 배열을 회귀 테스트로
고정했다. verifier나 waiver를 완화하지 않았다.

## 4. 검증 결과

### 4.1 로컬 검증

| 검사 | 결과 |
| --- | --- |
| 최종 focused promotion test | 33건 PASS |
| workflow 전체 Python test | 161건 PASS |
| actionlint 1.7.7 | 두 변경 workflow PASS |
| Docker optimized WASM | release build와 `wasm-opt -O` PASS, 6분 53초 |
| `@rhwp/editor` package test | 32건 PASS |
| npm core/editor pack | 두 package PASS |
| VSIX package | 37개 파일, 필수 WASM·JS·icon 포함 |
| 금지 package 경로 | `.env*`, `node_modules/`, `target/`, token 이름 없음 |
| Markdown link / `git diff --check` | PASS |

Rust 제품 source와 renderer를 바꾸지 않았으므로 renderer·시각 회귀는 범위에 넣지 않았다. 로컬 VSIX
dependency의 read-only audit에서는 production 취약점 0건, 기존 간접 dev dependency high 2건
(`browserslist`, `fast-uri`)이 관찰됐다. 이번 변경은 dependency와 lockfile을 바꾸지 않았다.

### 4.2 exact-head GitHub Actions 실증

| workflow | run | exact SHA | 소요 | 결과 |
| --- | --- | --- | ---: | --- |
| Release Binary | [34001610087](https://github.com/edwardkim/rhwp/actions/runs/34001610087) | `559edb0682...` | 26분 34초 | success |
| Publish All Packages | [34001611474](https://github.com/edwardkim/rhwp/actions/runs/34001611474) | `559edb0682...` | 11분 23초 | success |

Release Binary는 Linux x86_64/AArch64, macOS x86_64/AArch64, Windows x86_64 build 5개와 reusable package의
source guard, WASM, VSIX, aggregate가 성공했다. Release attach와 외부 publish 4개는 skipped됐고 artifact
8개가 생성됐다. 직접 Publish All Packages도 필수 build·aggregate가 성공하고 외부 publish 4개가
skipped됐으며 artifact 3개가 생성됐다.

두 evidence는 `mode=verify`, exact `githubSha`, 세 gate `success`, 네 채널 `skipped/verify-only`,
`errors=[]`, `accepted=true`, `verdict=completed`였다.

| offline verifier | 값 |
| --- | --- |
| run / pagination | 2개 / 모두 완결 |
| waiver | 0건 |
| policy SHA-256 | `dbd0bcd8d2829fdf7ffebfab5245f2cf9d2fc022906a2cf556d58f0579bd7b24` |
| inventory SHA-256 | `c8ac74b8cfc885817c73b6e317943e0a7fcd8030cb7f2b15e50897eac3805df5` |
| verdict SHA-256 | `febb524fa0a9df0ab39700ebcf1a85115eda85fc4afe8d54b7a1a14c240d0385` |
| 판정 | 오류 0건, `ok=true` |

실행 전후 `@rhwp/core`, `@rhwp/editor`, VS Code Marketplace, Open VSX는 모두 기존 `0.8.6`의
`already-present`였다. `test` Git tag와 GitHub Release도 생성되지 않았다.

## 5. 보안·운영·비용 영향

- workflow 기본 권한은 `contents: read`이며 Release job만 `contents: write`, npm publish job만
  `id-token: write`를 가진다.
- caller는 `secrets: inherit`를 쓰지 않고 `VSCE_PAT`, `OVSX_PAT` 이름만 전달한다. verify-only build에는
  publish secret과 OIDC 권한이 없다.
- 공개 상태 조회 실패를 미게시로 간주하지 않아 장애 중 중복 게시 시도를 차단한다.
- 매 PR에서 두 release workflow를 실행하지 않는다. workflow 변경을 `devel -> main`으로 promotion할 때와
  release canary에만 비게시 실증을 사용한다.
- 이번 exact-head 실증의 wall time은 Release Binary 26분 34초, direct package 11분 23초였다. 두 run은
  WASM·VSIX를 중복 build하므로 일상 CI에 넣으면 비용과 대기시간이 커진다. 독립 실행 두 개는 direct
  workflow와 nested 호출을 각각 검증하기 위한 promotion 증적이다.

## 6. rollback과 수동 복구

production 직접 호출에서 결함이 발견되면 일반 PR로 Release Binary의 reusable caller를 되돌리고,
이미 게시된 package version을 삭제하거나 덮어쓰지 않는다. Release 자체와 다섯 native archive는 보존한다.

복구가 필요하면 Release tag와 공개 채널 상태를 확인한 뒤 exact tag ref에서 Publish All Packages를
`publish=true`로 한 번 실행한다. 각 채널은 exact version이 있으면 skip하므로 부분 성공 뒤에도 미완료
채널만 재시도된다. 움직이는 branch에서의 강제 게시, secret 범위 확대, verifier waiver는 복구 수단으로
사용하지 않는다.

## 7. 다음 release canary와 이슈 종료 조건

비게시 실증은 production stable-tag 경계의 실제 성공을 대신하지 않는다. 다음 정식 release에서 다음을
post-release 증적으로 확인한다.

1. stable `v*` tag SHA와 Release Binary·nested package run의 `head_sha`가 일치한다.
2. 다섯 native build와 Release attach가 성공한 뒤 package 호출이 정확히 한 번 실행된다.
3. npm core/editor, VS Code Marketplace, Open VSX가 각각 `already-present | published`로 완료된다.
4. `release-publish-evidence`가 exact SHA와 `verdict=completed`를 기록한다.
5. 공개 registry의 네 exact version과 GitHub Release asset을 독립 조회한다.

현재 구현은 PR 준비 단계다. 최신 PR head의 CI와 메인테이너 self-review, `devel` merge가 먼저 필요하다.
#6634는 merge와 다음 release canary 증적을 확인하기 전에는 닫지 않는다. 최종 보고서 commit은 원격 실증
뒤의 증적 전용 변경이므로 그 새 SHA를 위 두 run이 검증했다고 주장하지 않으며, 향후 `devel -> main`
promotion에서는 당시 exact `devel` SHA의 새 #6689 증적을 사용한다.

## 8. 관련 문서

- [수행계획](../plans/task_m100_6634.md)
- [Stage 1 원인 계보와 RED](../working/task_m100_6634_stage1.md)
- [Stage 2 직접 호출과 source guard](../working/task_m100_6634_stage2.md)
- [Stage 3 채널별 멱등성](../working/task_m100_6634_stage3.md)
- [Stage 4 promotion·운영 절차](../working/task_m100_6634_stage4.md)
- [Stage 5-A 로컬 산출물 검증](../working/task_m100_6634_stage5a.md)
- [Stage 5-B exact-head 실증](../working/task_m100_6634_stage5b.md)
- [GitHub 저장소 운영 매뉴얼](../manual/github_operations.md)
- [배포 가이드](../manual/publish_guide.md)
