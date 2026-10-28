# Task M100 #6584 Stage R5 결과 — release-prep PR·5플랫폼 dry-run

- Issue: [#6584](https://github.com/edwardkim/rhwp/issues/6584)
- PR: [#6585](https://github.com/edwardkim/rhwp/pull/6585)
- 검증 code head: `4280831d1f25a189416c2fcec14e0d252dfb90c3`
- release base: `upstream/devel@063041a2ced54085b5cf94c2e646ac7aa0e1960d`
- Release Binary run: [#33569503350](https://github.com/edwardkim/rhwp/actions/runs/33569503350)
- 검증일: 2026-09-02 KST

## 1. Stage R5 판정

PR #6585의 exact code head에서 PR-triggered check 29건이 성공했고 정책상 3건이 생략됐다. 실패·대기
check는 없었다. `Release Binary`를 같은 head와 `tag=test` 입력으로 별도 실행해 다섯 native build가 모두
성공했으며, `v`로 시작하지 않는 dry-run 입력이므로 `Attach to GitHub Release` job은 의도대로 생략됐다.

검증 시점의 `upstream/devel`은 계획에서 고정한 release base와 같았다. merge-tree 검사는 tree
`00f4cd19e2195ae74315cba643e08331709b0d0c`를 충돌 없이 생성했고, PR 상태는
`MERGEABLE / CLEAN`이었다. 이 값들은 작성 시점 snapshot이며 merge 전 최신 상태를 다시 확인한다.

## 2. PR CI와 focused 대사

- CI [run #33568191662](https://github.com/edwardkim/rhwp/actions/runs/33568191662): Build & Test,
  lint, workspace archive, Native Skia, frontend package gate가 성공했다.
- CodeQL [run #33568191630](https://github.com/edwardkim/rhwp/actions/runs/33568191630): Rust,
  JavaScript/TypeScript, Python 분석과 GHAS CodeQL check가 성공했다.
- Render Diff, Proptest roundtrip, Adapter inter-diff가 같은 PR·branch·candidate SHA에서 성공했다.
- 정책상 생략된 3건은 WASM Build, Frontend unit gates, nextest duration refresh였다. required aggregate는
  성공했다.
- `python3 -m unittest scripts.tests.test_release_channel_policy_workflow
  scripts.tests.test_release_contributor_audit scripts.tests.test_release_record_contributors`를 다시 실행해
  19/19 통과했다.
- `node --test scripts/tests/font_decision_trace_contract.test.mjs`를 다시 실행해 12/12 통과했다.
- `git diff --check upstream/devel...HEAD`가 통과했다.

Stage R4와 exact code head의 GitHub Full CI가 이미 Rust·WASM·frontend 광범위 회귀를 통과했으며 self-review
중 source, test, fixture, workflow, baseline, asset 보정을 추가하지 않았다. 따라서 같은 candidate에서
release-test와 Native Skia 전체를 로컬로 반복하지 않았다.

## 3. Release Binary dry-run

| target | GitHub job | 결과 | payload binary 확인 |
|---|---|---|---|
| `x86_64-pc-windows-msvc` | `Build x86_64-pc-windows-msvc` | 성공 | PE32+ x86-64, `rhwp.exe` |
| `x86_64-unknown-linux-gnu` | `Build x86_64-unknown-linux-gnu` | 성공 | ELF 64-bit x86-64, mode 0755 |
| `aarch64-unknown-linux-gnu` | `Build aarch64-unknown-linux-gnu` | 성공 | ELF 64-bit AArch64, mode 0755 |
| `x86_64-apple-darwin` | `Build x86_64-apple-darwin` | 성공 | Mach-O 64-bit x86_64, mode 0755 |
| `aarch64-apple-darwin` | `Build aarch64-apple-darwin` | 성공 | Mach-O 64-bit arm64, mode 0755 |

Linux AArch64 job은 GitHub 표준 `ubuntu-24.04-arm` image에서 exact SHA를 checkout했다. 같은 native runner가
`target/aarch64-unknown-linux-gnu/release/rhwp --version`을 실행했고 `rhwp v0.8.6`과 종료 코드 0을
확인했다. 이는 cross compile 산출물의 형식만 검사한 결과가 아니다.

다섯 artifact를 임시 디렉터리에 내려받아 각각 풀었다. 모든 archive의 payload는 다음 네 항목만 가졌다.

```text
rhwp/rhwp 또는 rhwp/rhwp.exe
rhwp/LICENSE
rhwp/README.md
rhwp/README_EN.md
```

### 3.1 payload archive hash

아래 값은 Actions artifact API가 표시하는 외부 전달용 ZIP digest가 아니라, `gh run download`로 복원한
실제 `.tar.gz` 또는 `.zip` payload의 SHA-256이다.

| archive | SHA-256 |
|---|---|
| `rhwp-test-linux-aarch64.tar.gz` | `01176ac628a130063babc543e27d82fdf0c558a43536aefd1aa0a90d6b4e4f61` |
| `rhwp-test-linux-x86_64.tar.gz` | `6766f71061271bd476470ce551e3fd127b1f4aeec690e08874f7811b9870d811` |
| `rhwp-test-macos-aarch64.tar.gz` | `3023494845c2c7225c066994c76ddd799930ea22a2a3a9c451128e6906690b74` |
| `rhwp-test-macos-x86_64.tar.gz` | `ab5621f9ede7ee868152576355ac268ed33fe03bfa67c264fe4c61af8e15eeba` |
| `rhwp-test-windows-x86_64.zip` | `ffaec1f81b53351b1c0908422e55d0e9c7535e1db0074e08660a652f56442a13` |

payload binary 크기는 Linux AArch64 21,795,824 bytes, Linux x86_64 24,473,080 bytes, macOS
AArch64 19,945,824 bytes, macOS x86_64 22,649,200 bytes, Windows x86_64 28,185,088 bytes였다.
검사 뒤 임시 다운로드 디렉터리를 삭제하고 부재를 확인했으며 저장소에는 산출물을 추가하지 않았다.

## 4. Self-review 결과와 잔여 위험

- Cargo, npm editor, Studio, VS Code, Chrome/Edge, Firefox, Safari 정본 버전은 모두 0.8.6이다.
- 기여자 ledger, 한·영 CHANGELOG와 GitHub Release note의 사람 기여자 20명 집합은 동일하고 bot을
  별도로 유지한다.
- 이 PR은 renderer/layout/typeset 실행 코드를 바꾸지 않는다. 새 시각 증적은 필수가 아니며 Stage R4의
  Native Skia·CDP·responsive 검증을 참고 근거로 사용했다.
- `.github/workflows/**`와 `.github/actions/**`는 이 PR에서 바뀌지 않았다. 따라서 self-review trailing
  문서 commit은 기존 녹색 code candidate를 대상으로 review-only fast-pass A 경로를 적용할 수 있다.
- 실제 `v0.8.6` tag, GitHub Release asset과 `SHA256SUMS.txt`, npm·VS Code·Open VSX 게시 및 브라우저
  스토어 제출은 아직 수행하지 않았다. 이는 Stage R6~R8의 승인 게이트다.
- #5949는 실제 release asset 검증 전, #6243은 실제 post-release canary 성공 전, #6584는 모든 공식
  채널 정산 전까지 OPEN으로 유지한다.

## 5. 종료 판정

Stage R5의 code candidate·Full CI·5플랫폼 dry-run·Linux AArch64 native 실행 조건은 통과했다. 차단 발견
사항은 없다. 다음 절차는 self-review·오늘할일 trailing commit을 같은 source branch에 반영하고 최신 head의
review-only CI와 `MERGEABLE / CLEAN`을 재확인하는 것이다. 원격 push와 PR merge는 각각 별도 승인을 받는다.
