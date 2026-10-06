---
kind: report
status: archived
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-14
---

# PR #7121 GPU 의존성 메인터너 보정 리뷰·검증

## 결론과 식별

**현재 신규 처리 대상은 #7121 한 건이다. 메인터너 보정과 검증을 완료한 후보만 수용한다.**
#7122~#7128은 검증 도중 다른 경로로 이미 merge되어 최신 devel에 포함됐다. 다시 제출하거나 종료하지 않는다.
#7121 원 head의 GPU 실패와 보정본의 수용 근거를 구분하며, 이미 merge된 의존성 변경을 새 PR 변경으로 계산하지 않는다.
#7121 보정 PR 생성·최신 CI·승인·merge 및 원 PR 종료는 아직 수행하지 않았다.

- 브랜치: `review/dependabot-20260914`, 원격 제출: `upstream`의 동명 branch -> `devel`.
- 최초 검증 기준: `93ffc3dd59c120bd54df4c2ac6d1ddbe630f8a2d`.
- 최종 rebase 기준: `upstream/devel@0c581f7e9a2a910f6edfca9045ea04fffbdb43d3`. 문서 충돌은 양쪽 기록을 보존해 해결했다.
- rebase 후 보정 commit: `64b393e63488eed2a24f8fe307685e1386f0414b`, 기록 포함 head: `6d5f9d4d1`.
- 실제 기능 검증 head: `00b5dc1059d129d445ec012c47dfc76788b6f7d3`.
- rebase 때 검증 후 기록 commit `e3453941d`와 `mydocs/**`를 제외한 tracked 파일 전체의 차이가 없음을 확인했다. 코드·의존성·테스트·워크플로가 같아 테스트를 반복하지 않았다.
- 검증 후 추가 범위는 review·오늘할일·대표 PNG와 Cargo.toml의 버전 설명 주석뿐이다. 의존성 값, 소스, 테스트와 baseline은 다시 바꾸지 않는다.
- [#7121 보정 PR 조회](https://github.com/edwardkim/rhwp/pulls?q=is%3Apr+head%3Areview%2Fdependabot-20260914).
- 절차: collaborator 외부 PR 통합 경로, local_validation 4.3 변경 범위별 게이트.
- #7121은 `git cherry-pick -x`로 적용해 작성자와 원 SHA를 보존했다. 이미 devel에 반영된 나머지 7건의 중복 변경은 rebase에서 제외됐다.

## 검토 대상과 현재 PR 변경 범위

- 원 PR: [#7121](https://github.com/edwardkim/rhwp/pull/7121), 작성자 `dependabot[bot]`, base `devel`.
- 원 head: `cd00c2b6316ba340b71886045920ab0431e1b42a`.
- Reviewer: `jangster77`. bot PR이므로 첫 기여자 절차는 적용하지 않는다.
- 원 head 판정: GPU feature의 두 E0308로 머지 보류.
- 보정본 판정: **메인터너 보정 후 수용 가능**. 의존성 정렬과 실제 검증을 완료했으며 원 head 단독 승인은 아니다.
- 제품 변경 파일: `Cargo.toml`, `Cargo.lock`. GPU 의존성과 해당 설명 주석만 변경한다.
- 기록·증적: 이 review, 오늘할일, 대표 GPU/CPU PNG 두 장.
- 변경하지 않는 범위: GPU renderer 소스, 기본 PDF/Native Skia 의존성 설정, Studio·확장 패키지, workflow, 테스트와 baseline.
- 사용자 지시에 따라 이번 보정 review는 이 파일 하나로 관리한다. upstream의 기존 개별 검토 기록은 수정하거나 삭제하지 않는다.

## 원 PR CI와 보정본의 구분

원 head의 [Build & Test SUCCESS](https://github.com/edwardkim/rhwp/actions/runs/34800350025/job/103844275852)를
접수 시 확인했지만 optional GPU 경로는 로컬에서 실패했다. 녹색 CI만으로 수용하지 않고
아래 GPU 보정과 전체 회귀를 실행했다. 원 PR CI는 새 보정 PR의 최신 head CI를 대신하지 않는다.

## GPU 보정 이력과 원인

초기 통합 head `aee00a28c`에서 아래 명령이 exit 101로 실패했다.

```bash
CARGO_BUILD_JOBS=4 cargo check --locked --features gpu --lib --target-dir target/pr-review
```

- `src/renderer/gpu.rs:191`: vello_svg 0.11이 받는 usvg 0.48 Tree와 resvg-gpu 0.46 Tree가 달랐다.
- `src/renderer/gpu.rs:193`: vello_svg가 반환한 vello 0.10 Scene과 직접 의존성 vello 0.9 Scene이 달랐다.
- 원 PR CI의 Build & Test 성공만으로 optional GPU feature 호환성을 보장하지 못했다.
- 최초 제외 revert `107445575` 뒤 사용자 지시에 따라 `10b931430`에서 재적용했다. 두 이력은 지우지 않았다.
- 메인터너 보정 `00b5dc1059d129d445ec012c47dfc76788b6f7d3`: vello 0.10.0, vello_svg 0.11.0, resvg-gpu 0.48.1, vello_shaders 0.10.0으로 GPU 스택 정렬.
- 기본 PDF용 usvg/resvg 및 Native Skia용 resvg 0.47은 유지했다. GPU 소스/API를 우회하거나 기능을 비활성화하지 않았다.
- 최초 lock 갱신용 GPU check는 0, 이후 `--locked` GPU Clippy/build 및 실행 검증도 0이다.
- 제외 상태에서 실행하던 focused-encoding은 사용자 지시에 따른 보정 전환으로 exit 130 중단했다. 이 결과를 통과로 사용하지 않는다.

## 환경과 입력

- Ubuntu 24.04, Rust 1.93.1, Node.js 24.15.0; build jobs 4, test threads 8.
- Cargo target은 기존 `target/pr-review` 재사용. Cargo 검증은 순차 실행했으며 공유 target을 삭제하지 않았다.
- Docker 미설치로 WASM은 host `--dev --no-opt` 진단 빌드다. Docker 최적화 release WASM 또는 release size gate를 통과했다고 주장하지 않는다.
- Vite는 세 패키지 모두 8.3.0, Studio @types/chrome은 0.2.9, Firefox는 155.0.1이다.
- 새 입력·fixture·baseline을 도입하지 않아 신규 fixture 등록·보안 corpus 래칫 갱신은 해당 없음이다.
- 기존 입력 `samples/para-001.hwp`의 실행 파일과 HEAD Git blob SHA-256:
  `bab4561ceb02cdfa184a1689be9619c08e18d6021cdbc423486b848bc14d267e`.
- 검증 WASM `pkg/rhwp_bg.wasm` SHA-256:
  `206775a88eb62606e7199afcfad7166e0a9a433166360f04bc6a5b54545a1b12`.

## 실제 검증 결과

아래는 GPU 보정을 포함한 최종 기능 head 기준이다. 로그는 `output/dependabot-20260914/`에만 보관한다.
Native Skia·Studio·확장·workflow 검사는 기존 기능의 회귀 확인 기록이며 해당 파일을 이번 PR에서 바꾼다는 뜻이 아니다.

| 검사 | 결과 |
| --- | --- |
| GPU workspace/all-target Clippy | exit 0 |
| GPU CLI build 및 gpu-info | exit 0 |
| GPU 첫 페이지 출력 + CPU benchmark | exit 0 |
| fmt check | exit 0 |
| native / WASM / workspace-all-target Clippy | 각각 exit 0 |
| workspace build | exit 0 |
| suite manifest prepare/check | 1311 sources, 48/48 targets; exit 0 |
| encoding/CP949 집중 nextest | 10 passed, 9855 filtered/skipped, exit 0 |
| 전체 nextest | 9814 passed, 51 skipped, 7 slow; 496.657초; exit 0 |
| Native Skia lib | 3930+15+165+2 = 4112 passed, 13 ignored; exit 0 |
| Native Skia placeholder PNG | 2 passed, 187 filtered; exit 0 |
| Native Skia direct PDF | 4 passed, 205 filtered; exit 0 |
| 새 WASM dev/no-opt | exit 0 |
| 두 변경 workflow actionlint | exit 0 |
| nextest workflow 계약 검사 | 15 passed; exit 0 |
| Studio/Chrome/Firefox npm ci 및 npm ls | 모두 exit 0 |
| Studio unit/editor 계약 | 1692 passed, 2 skipped, 0 failed |
| Studio TypeScript + production JS/PWA build | exit 0 |
| Chrome/Firefox packaged build | 각각 exit 0 |
| Studio 실제 browser document-title/open/save | exit 0 |
| Chrome page-budget | 4 passed; exit 0 |
| Chrome packaged extension smoke | viewer/options/print/service worker/content script; exit 0 |
| Firefox 실제 download/save | 완료 다운로드 3개, viewer tab 1개, editPreserved=true; exit 0 |

Firefox 첫 시도는 `Could not find profile folder`로 exit 1이었다.
`TMPDIR=/home/tsjang/rhwp/output/dependabot-20260914/firefox-tmp`로 바꾼 같은 테스트가
exit 0으로 끝났다. 테스트 코드를 수정하거나 실패를 skip 처리하지 않았다.

기존 nextest 설정의 `junit.report-skipped` unknown-key warning, Vite의 CanvasKit fs/path
externalization 및 큰 chunk warning은 남아 있다. 검사 실패나 경고 없음으로 바꿔 쓰지 않는다.
npm audit 및 OS/물리 GPU별 전체 행렬은 실행하지 않았다.

### 재현 명령

```bash
export CARGO_BUILD_JOBS=4
node scripts/rust-test-suite-manifest.mjs --prepare
cargo clippy --locked --features gpu --workspace --all-targets --target-dir target/pr-review -- -D warnings
cargo build --locked --features gpu --bin rhwp --target-dir target/pr-review
target/pr-review/debug/rhwp gpu-info
timeout 180 target/pr-review/debug/rhwp export-png-gpu samples/para-001.hwp --page 0 --scale 1 --benchmark --output output/dependabot-20260914/gpu-render
cargo fmt --all -- --check
cargo clippy --locked --target-dir target/pr-review -- -D warnings
cargo clippy --locked -p rhwp --lib --target wasm32-unknown-unknown --target-dir target/pr-review -- -D warnings
cargo build --locked --workspace --target-dir target/pr-review
cargo clippy --locked --workspace --all-targets --target-dir target/pr-review -- -D warnings
node scripts/rust-test-suite-manifest.mjs --check
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast --no-tests fail -E 'test(/encoding|codepage|euc_kr|cp949/)'
cargo nextest run --locked --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 8 --no-fail-fast
cargo test --locked --profile release-test --target-dir target/pr-review --features native-skia --lib -- --test-threads 8
node scripts/run-rust-test.mjs issue_2225_missing_picture_placeholder -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
node scripts/run-rust-test.mjs render_p37_direct_pdf_export -- --cargo-profile release-test --target-dir target/pr-review --features native-skia
CARGO_TARGET_DIR=target/pr-review scripts/wasm-pack-locked.sh --target web --out-dir pkg --dev --no-opt
actionlint .github/workflows/build-nextest-archives.yml .github/workflows/run-nextest-archives.yml
python3 -m unittest discover -s scripts/tests -p test_nextest_archive_workflow.py
for package in rhwp-studio rhwp-chrome rhwp-firefox; do
  npm --prefix "$package" ci --no-audit --no-fund
  npm --prefix "$package" ls --depth=0
done
npm --prefix rhwp-studio test
npm --prefix rhwp-studio run build
npm --prefix rhwp-chrome run build
npm --prefix rhwp-firefox run build
npm --prefix rhwp-studio run e2e:document-title
node --test rhwp-chrome/e2e/page-budget.test.mjs
node rhwp-chrome/e2e/extension-smoke.test.mjs
TMPDIR=/home/tsjang/rhwp/output/dependabot-20260914/firefox-tmp FIREFOX_EXECUTABLE_PATH=/usr/bin/firefox npm --prefix rhwp-firefox run test:e2e:download
```

## GPU 직접 PNG 판정과 증적

GPU 바이너리는 빌드 시각 15:47의 Cargo 보존 실행 파일
`target/pr-review/debug/deps/rhwp-55279e0e4104cbaf`,
SHA-256 `c62b8341d8295dd80acad35542337680406ec1f12165759d9f0a2e17501bf973`이다.
`target/pr-review/debug/rhwp`는 이후 기본 workspace 빌드로 교체되므로 GPU 증적 바이너리와 혼동하지 않는다.

비교 입력은 3쪽 HWP의 첫 페이지(0-based 0), scale 1, 같은 usvg 트리다.
두 PNG를 도구로 직접 열어 보았고 해당 페이지의 글자 겹침·누락 및 문단 위치 차이를 발견하지 못했다.
GPU 글자 가장자리/농도 차이는 보이며 픽셀 완전 일치로 판정하지 않는다.
Vulkan llvmpipe(Cpu) 어댑터에서 실행했으므로 물리 GPU 성능 증거가 아니다.

- 두 출력: 794x1123.
- 평균 절대 픽셀차: 0.85/255; |delta|>=16 픽셀: 2.88%.
- 한컴 기준 PDF, compare/overlay/review 패널 및 visual-sweep 후보·pixel-match·proxy 지표: 미산출/해당 없음.
- 이것은 GPU 의존성 호환성과 제한된 동일 입력 출력 판정이며, 전체 fidelity/visual sweep 통과가 아니다.
- 임시 GPU: `output/dependabot-20260914/gpu-render/para-001.png`.
- 임시 CPU: `output/dependabot-20260914/gpu-render/para-001_001.cpu.png`.
- 최종 코멘트 GPU: [대표 GPU PNG](../assets/pr_7121_maintainer_20260914/pr7121-gpu-p001.png),
  SHA-256 `f8abcf014874d22975ded35e6c4b37e00071b089eb6a0e30f29e04123e9934c5`.
- 최종 코멘트 CPU: [대표 CPU PNG](../assets/pr_7121_maintainer_20260914/pr7121-cpu-reference-p001.png),
  SHA-256 `a9ec5d9464725363784c5328a7dce14e77e3fd5118ee7ca33199f6a5fe2fd7a6`.

대표 두 PNG만 commit하고 로그, JSON, generated suite, pkg/dist, Firefox 임시 profile 및 HTML 보고서는 제외한다.
원본 HWP는 기존 commit 경로를 재사용하며 PDF나 중복 입력을 새로 만들지 않았다.

## #7121 보정 PR 제출 및 merge 후 comment 계획

현재 단계는 #7121 보정 PR 제출 준비이며 원 PR 종료, merge, 원격 브랜치 삭제는 수행하지 않는다.
보정 PR merge 후 실제 merge SHA·최신 CI와 #7121 원 head 및 보정 포함 관계를 확인한다.
그 뒤 #7121에 아래 내용을 한국어 `--body-file`로 게시하고 API로 본문·링크를 재조회한 다음 종료한다.
이미 merge된 #7122~#7128에는 이 작업으로 중복 코멘트·종료 처리를 하지 않는다.
추가 개별 review 파일이나 별도 기록 PR은 만들지 않는다.

```text
#7121 원 head cd00c2b6316ba340b71886045920ab0431e1b42a를 GPU 보정 PR <보정-PR-URL>에 반영했습니다.
보정 PR merge SHA: <merge-commit-sha>
판정: 메인터너 보정 후 수용 가능 (보정·검증 완료)
검증 코드 head: 00b5dc1059d129d445ec012c47dfc76788b6f7d3
검증 및 범위·한계: <merge-SHA-고정-pr_7121_review_impl.md-URL>
보정 PR 최신 CI: <실제-보정-head-CI-URL>
```

#7121에는 원 head의 두 타입 불일치와 보정 commit을 분리해 설명한다.
확인 범위는 3쪽 중 첫 페이지 1쪽, 동일 usvg 입력의 GPU/CPU 비교이며
자동 visual-sweep 후보 수·pixel match·proxy는 미산출이라고 적는다.
직접 확인 결과와 수치, llvmpipe(Cpu) 및 한컴 기준 PDF 미비교의 한계를 함께 게시한다.
코멘트에서 실제 표시할 증적은 아래 두 파일뿐이며 merge SHA 고정 raw URL을 사용한다.

```markdown
![PR 7121 GPU](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7121_maintainer_20260914/pr7121-gpu-p001.png)
![PR 7121 CPU reference](https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_7121_maintainer_20260914/pr7121-cpu-reference-p001.png)
```
