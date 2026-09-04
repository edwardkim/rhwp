# PR 초안 — #2233 wasm-pack 0.15.0 pin 및 Actions/Docker 정합화

## 생성 결과

- PR: [#2420](https://github.com/edwardkim/rhwp/pull/2420)
- 상태: Draft / open
- base: `devel`
- head: `task_m100_2233-wasm-pack-pin`
- 생성일: 2026-07-19
- 생성 직후 `CI preflight`, `CodeQL preflight`, `Render Diff preflight`: PASS
- 본 CI / CodeQL / Render Diff: 진행 중

## 제목

```text
[CI] wasm-pack 0.15.0 pin 및 Actions/Docker toolchain 정합화 (#2233)
```

## 본문

```markdown
## 목적

GitHub Actions의 미고정 wasm-pack installer와 Docker의 `wasm-pack 0.15.0` pin을 하나의 명시적
정책으로 정합합니다.

Closes #2233
Supersedes #2274

## 변경

- `.github/actions/install-wasm-pack/action.yml` composite action 신설
  - `wasm-pack 0.15.0` linux-musl release binary 사용
  - `wasm-pack --version` 로그 출력
- wasm-pack을 사용하는 5개 workflow, 6개 설치 지점을 composite action으로 통일
  - CI dev frontend gate / release WASM
  - deploy-pages
  - full-renderer-sweep
  - npm-publish
  - Render Diff
- Docker의 `cargo install wasm-pack@0.15.0` pin과 동기화 규칙 기록
- `mydocs/tech/wasm_pack_version_policy.md`에 버전 정책·갱신 절차 문서화
- 기술 문서 지도에 wasm-pack 정책 진입점 추가

workflow trigger, required check 이름, PR cache restore-only, Node/Rust toolchain, build 명령과 release
artifact 정책은 변경하지 않습니다.

## contributor provenance

닫힌 PR #2274의 원 구현 commit을 최신 `upstream/devel` 위에 `git cherry-pick -x`로 이식했습니다.

- source PR: #2274
- source commit: `d6c9494812a054b1497f29cd9865185e92675d8a`
- integration commit: `0213e52a4b4ea3388759272f78382586ce5ac2be`
- original author와 `Co-Authored-By` trailer 보존
- contributor commit rewrite/squash 없음

최신 문서 front matter와 탐색 규칙 보정은 별도 collaborator commit으로 분리했습니다.

## 로컬 검증

- `actionlint 1.7.12`: 변경 workflow PASS
- wasm-pack 0.15.0 linux-musl release asset·tar 내부 경로 확인
- fresh dev WASM build: PASS
- generated WASM binding/editor embed contract: 3/3 PASS
- release WASM build + `wasm-opt`: PASS
- Markdown link/redirect 검사: 393개 문서, PASS
- document metadata 검사: 384개 문서, PASS
- `git diff --check`: PASS
- Git LFS/worktree: clean

제품 renderer/layout/WASM source는 변경하지 않아 별도 visual sweep 대상이 아닙니다. 최신 PR head의
`CI / Build & Test`, frontend package gates, CodeQL, Render Diff 결과를 최종 merge gate로 사용합니다.

준비 중 최신 `upstream/devel@62bcae43`을 merge한 뒤 dev/release WASM, consumer 3건과 `wasm-opt`를
같은 결합 트리에서 다시 통과했습니다.
```

## 생성 기록

작업지시자 승인 후 아래 경로로 remote branch와 Draft PR을 생성했다.

```bash
git push -u upstream HEAD:task_m100_2233-wasm-pack-pin
gh pr create --repo edwardkim/rhwp \
  --base devel \
  --head task_m100_2233-wasm-pack-pin \
  --draft \
  --title "[CI] wasm-pack 0.15.0 pin 및 Actions/Docker toolchain 정합화 (#2233)" \
  --body-file /private/tmp/rhwp-2233-pr-body.md
```
