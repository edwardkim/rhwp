# Task M100 #2233 Stage 1 완료 보고 — wasm-pack pin 이식·로컬 검증

- 이슈: [#2233](https://github.com/edwardkim/rhwp/issues/2233)
- 신규 PR: [#2420](https://github.com/edwardkim/rhwp/pull/2420) — Draft
- source PR: [#2274](https://github.com/edwardkim/rhwp/pull/2274) — closed, unmerged
- 브랜치: `codex/issue-2233-wasm-pack-pin`
- 최초 기준: `upstream/devel@537639445332e85b76eb29c76e1dae4d8930369f`
- 최신 upstream 통합: `upstream/devel@62bcae435370b58373248c284c126c9572098522`
- 통합 merge commit: `98d25a140c9cb20dfa861c1f7d78770226f5ca58`
- source commit: `d6c9494812a054b1497f29cd9865185e92675d8a`
- integration commit: `0213e52a4b4ea3388759272f78382586ce5ac2be`
- collaborator docs commit: `460b1d8f9acecad1f4ec922b86e4c323c11e76f9`
- 작성일: 2026-07-19

## 1. 완료 요약

닫힌 PR #2274를 reopen하거나 review하지 않고, Issue #2233 기반 신규 PR 준비 브랜치에 원 구현 commit을
`git cherry-pick -x`로 이식했다.

- contributor author와 `Co-Authored-By` trailer를 보존했다.
- 최신 `upstream/devel`과 8개 원 변경 파일이 충돌 없이 적용됐다.
- GitHub Actions의 wasm-pack 설치 6곳을 하나의 composite action으로 통일했다.
- Docker pin과 Actions pin이 모두 0.15.0임을 확인했다.
- 최신 문서 metadata·탐색 규칙 보정을 별도 collaborator commit으로 분리했다.
- dev/release WASM, 핵심 consumer, `wasm-opt`와 문서 검사를 통과했다.

## 2. 설치 경로 감사

| 경로 | 참조 수 | runner |
|------|--------:|--------|
| `.github/workflows/ci.yml` | 2 | `ubuntu-latest` |
| `.github/workflows/deploy-pages.yml` | 1 | `ubuntu-latest` |
| `.github/workflows/full-renderer-sweep.yml` | 1 | `ubuntu-latest` |
| `.github/workflows/npm-publish.yml` | 1 | `ubuntu-latest` |
| `.github/workflows/render-diff.yml` | 1 | `ubuntu-latest` |

- `uses: ./.github/actions/install-wasm-pack`: 6곳
- `rustwasm.github.io/wasm-pack/installer/init.sh`: 변경 workflow에서 0곳
- `Dockerfile`: `cargo install wasm-pack@0.15.0`
- local action을 사용하는 모든 job은 `actions/checkout` 뒤에 실행된다.

## 3. release asset 검증

대상 URL:

```text
https://github.com/rustwasm/wasm-pack/releases/download/v0.15.0/
wasm-pack-v0.15.0-x86_64-unknown-linux-musl.tar.gz
```

확인 결과:

- download: PASS
- SHA-256: `c09f971ecaed9a2efc80fdcea7a00ef6b53c7fadc8c57d1f61b53a6aa66b668a`
- tar 내부 경로: `wasm-pack-v0.15.0-x86_64-unknown-linux-musl/wasm-pack`
- binary: `ELF 64-bit LSB pie executable, x86-64, static-pie linked`
- 현재 workflow runner: 모두 `ubuntu-latest`

## 4. 로컬 검증

| Gate | 결과 |
|------|------|
| `actionlint 1.7.12` 변경 workflow 5개 | PASS |
| `wasm-pack --version` | `wasm-pack 0.15.0` |
| `wasm-pack build --target web --dev` | PASS |
| WASM binding/editor embed contract | 3/3 PASS |
| `wasm-pack build --target web --release` | PASS |
| release `wasm-opt` | PASS |
| `git diff --check upstream/devel...HEAD` | PASS |
| changed Markdown link/redirect 검사 | 393개 문서, 이상 없음 |
| document metadata 검사 | 384개 문서, 이상 없음 |
| `git lfs status` | clean |
| `git status --short --branch` | tracked change 없음, `upstream/devel` 대비 commit만 ahead |

`actionlint` 최초 실행은 변경 범위 밖 `.github/workflows/ci.yml`의 기존 SC2012 진단 1건을 출력했다.
해당 줄은 `upstream/devel...HEAD` diff에 포함되지 않는다. 기존 저장소용 SC2086/SC2035와 SC2012를 제외한
동일 5개 workflow 검사는 통과했다.

release 첫 실행은 sandbox가 wasm-pack 관리 `wasm-opt` 실행을 `Operation not permitted`로 막았다.
release Rust compile 자체는 완료됐고, 동일 명령을 sandbox 밖에서 다시 실행해
`Optimizing wasm binaries with wasm-opt`와 최종 성공을 확인했다.

산출물 크기 참고값:

- dev `rhwp_bg.wasm`: 약 23 MiB
- release+wasm-opt `rhwp_bg.wasm`: 약 6.7 MiB

## 5. 최신 upstream 통합

문서 준비 중 `upstream/devel`이 최초 기준보다 5커밋 전진했다. contributor 이식 commit을 rebase로 다시
쓰지 않고 `upstream/devel@62bcae43`을 merge했다.

- upstream 변경: `src/renderer/typeset.rs`, `tools/task2279/band_compare.py`, PR #2412 review와 오늘할일
- #2233 workflow/action/Docker/기술 정책 파일과 겹침: 0건
- `mydocs/orders/20260719.md`: upstream 오후 후반 사이클과 #2233 행을 자동 merge로 모두 보존
- merge 후 `wasm-pack build --target web --dev`: PASS
- merge 후 binding/editor consumer: 3/3 PASS
- merge 후 `wasm-pack build --target web --release`와 `wasm-opt`: PASS
- 최종 PR diff: upstream의 #2412 제품·도구·review 파일은 제외되고 #2233 변경만 유지

## 6. 시각 검증 판정

제품 renderer/layout/typeset/WASM source를 바꾸지 않고 CI installer와 문서만 변경한다. 렌더 출력 의미가
변하지 않으므로 로컬 visual sweep 대상이 아니다. 대신 신규 PR의 Render Diff workflow가 동일 0.15.0으로
fresh dev WASM을 빌드하고 consumer gate를 통과하는지를 최신 PR head에서 확인한다.

## 7. 다음 관문

- [x] Issue #2233 기준 신규 PR branch 준비
- [x] source PR/commit provenance와 contributor credit 보존
- [x] targeted local gate 통과
- [x] 신규 PR 본문 초안 준비
- [x] 작업지시자 승인 후 remote push·Draft PR #2420 생성
- [ ] 신규 PR 최신 head의 CI / CodeQL / Render Diff 통과
- [ ] merge와 Issue #2233 close는 별도 승인 후 처리
