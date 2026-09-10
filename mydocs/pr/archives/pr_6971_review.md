# PR #6971 검토

## 제출 및 전체 회귀 재검증 (2026-09-10)

- 통합 PR: [#6995](https://github.com/edwardkim/rhwp/pull/6995), source branch는 `maintainer/pr6971-decoration-host-20260910`이다.
- 검증 및 보정 code candidate: `874a709fef806b6f470bcf7932d365c5328eed5d`. 원 contributor 변경은 아래에 기록한 cherry-pick을 통해 계보에 보존했다.
- 리베이스 후 전체 nextest를 재실행했다: **9,417 passed, 0 failed, 46 skipped**, 12 threads, 실행 summary 497.336초, 컴파일 포함 781초, exit 0.
- Native Skia lib 네 summary의 합계는 **4,112 passed, 13 ignored, 0 failed**였다. placeholder 2 passed/177 filtered-skipped 및 direct-PDF 4 passed/144 filtered-skipped도 exit 0이었다.
- fmt check를 다시 통과했다. 동일 source의 workspace build, native/WASM/workspace-all-targets Clippy, manifest, focused 2개, 혼재 프로브 4개, 경계 12개 구성 및 직접 시각 확인 결과도 유지된다.
- 소스 SHA-256 `f6b5b88b59eb68f5186f4d8a7c725e74942bbfa25b190c8edc4773b220b6dcca` 일치를 commit 직전에 확인했다. 기준 PDF와 대표 PNG는 이 code candidate에 포함됐다.
- 임시 결과는 `output/pr_6971_20260910/final/`에만 두었으며 log, probe, raw SVG/JSON/HTML, 중간 raster는 commit하지 않았다.

### 코드 head의 실제 GitHub CI

다음은 같은 PR #6995, branch 및 exact code candidate의 완료 결과다. CI run의 `event=pull_request`, `pull_requests=[6995]`, head SHA 일치를 API로 확인했다.

- [CI / Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34476649332): success. Lint, Native Skia, archive A/B/C/D 빌드 및 회귀 shard를 실제 실행해 통과했다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/34476649381): Rust Analyze 포함 완료. 최종 GHAS CodeQL check는 정책상 skipped였다.
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/34476649009): Canvas visual diff 통과.
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/34476649278): 통과.
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/34476649391): 통과.
- code head의 `MERGEABLE/CLEAN`을 확인했다. 이는 작성 시점 값이며 아래 문서 tail을 push한 뒤 최신 상태를 다시 확인한다.

### 문서 trailing과 승인 범위

사용자는 전체 회귀, commit, PR 생성 및 CI 완료 후 merge/후속 처리를 승인했다. 이 기록은 code CI가 성공한 뒤 **같은 PR**에 추가하는 docs-only trailing commit이다. 원 PR 번호의 review 두 개를 archive로 옮기고 오늘할일과 함께 제출하며, 별도의 통합 PR 번호용 리뷰나 docs-only PR은 만들지 않는다.

문서 작성 직전 upstream/devel은 `66f25e744edd3a5f8cfd6a09836c1372729791e0`이었다. 오늘할일의 upstream 변경은 기존 하단에만 있어 이번 기록은 변경되지 않은 상단 경계에 추가했다. source를 rebase/merge하거나 최신 devel의 다른 작업 기록을 source에 복사하지 않았다.

남은 gate는 문서 trailing head의 CI/required aggregate와 최신 mergeability다. 통과 뒤 일반 merge로 통합하고, 문서·asset의 devel 반영을 확인한 후 아래 comment 계획에 따라 원 PR #6971을 superseded로 종료하고 관련 #6969의 종료 상태와 증적을 기록한다. 이번 전용 local/upstream branch만 정리하고 contributor fork, 기본 작업공간과 공유 `target/pr-review`는 보존한다. merge SHA와 실제 close 결과를 미리 완료로 기록하지 않는다.

이 절 뒤의 미커밋·전체 회귀 미실행 문구는 이전 단계의 이력이다.


## 최종 판정

**메인터너 보정 후 수용 가능.** 최신 `upstream/devel`에 리베이스한 보정본에서 제목 복원,
일반 표/decoration 표 혼재 시 제목 중복 방지, decoration 표의 하단 anchor/분할 경계를
확인했다. 기존 두 보류 사유는 아래 근거로 해제했다. 원 contributor head만을 승인하는
판정은 아니다. 보정 commit과 code candidate Full CI를 고정했고, 사용자가 merge/후속 처리를
승인했다. 최신 문서 trailing head의 CI와 mergeability 확인은 별도 게이트다.

이 문서에서 아래에 남긴 `머지 보류`와 `미검증`은 과거 단계의 기록이다. 현재 판정과 검증
범위는 이 절 및 바로 아래 리베이스 후 결과를 우선한다.

## 리베이스 후 보류 사유 해소 (2026-09-10)

### 동기화와 대상 식별

- `git fetch upstream devel` 뒤 로컬 검토 브랜치에서 `git rebase --autostash upstream/devel`을 실행했다.
- 새 upstream 기준: `2a780e0d296846df577866eba6ac8f388527551b`.
- 브랜치: `review/davindev-pr6971-20260910`.
- 리베이스된 로컬 cherry-pick: `a6d48e1c7ff3a2e7425666935651119d1fac2abf`.
- 원 contributor head: `5c68bbd909be51a059121096bda1ae7617efcdd6`.
- 리베이스 충돌이 없었고 기존 미커밋 메인터너 보정을 autostash에서 복원했다. 이후 제품 코드를
  추가로 바꾸거나 진단 경고를 숨기는 보정을 넣지 않았다.
- `src/renderer/typeset.rs` SHA-256:
  `f6b5b88b59eb68f5186f4d8a7c725e74942bbfa25b190c8edc4773b220b6dcca`.
- 이번 재빌드/시각 검증 바이너리 `target/pr-review/debug/rhwp` SHA-256:
  `c624b22e65ff0b96312cc9d18f99a687d370ba75a6c4875064e13bf8a4533d36`.
- 원격 contributor 브랜치는 변경하지 않았으며 commit, push, PR 생성, GitHub 게시 또는 merge는 수행하지 않았다.

### 보류 1: 복수 decoration 표의 페이지 하단 경계

측정 높이 71.92px인 fixture의 결재표에 대해 가용 높이를 72/75/80/90/120/150px로 설정하고,
각 높이에서 동일 anchor의 표 1개와 2개를 비교했다. **6개 높이, 12개 구성 모두 제목 1회,
1 page였고**, 표 2개 구성의 모든 table bbox가 단일 표 bbox를 두 번 놓은 결과와 같았다.
따라서 이 경계에서 형제 표의 anchor 이동이나 불필요한 continuation page는 없었다.
겹치는 같은 위치에 표를 의도적으로 복제한 프로브의 overlap 진단을 원 fixture의 배치 결함으로
해석하지 않는다.

처음 프로브에 넣었던 60/70px 용지는 표 자체보다 작다. `typeset_table_paragraph`의 기존
`oversized_multirow` 가드(#992)는 이 경우를 decoration 단축에서 제외하고 일반 본문 표
분할 경로로 보낸다. 따라서 그 경로에도 decoration의 동일-page/동일-anchor 계약을 적용한
프로브 판정이 잘못됐다. 사용자 승인 후 검사 입력을 실제 decoration 경계 6개로 바로잡았고,
재컴파일 및 재실행이 exit 0, `BOUNDARY_SUMMARY failures=0`이었다.

초기 실패 로그는 `output/pr_6971_20260910/rebased/boundary-probe.log`에 그대로 보존했다.
60/70px에서 관찰한 2-page/제목 2회 결과를 통과로 바꾸거나 삭제하지 않는다. 그 결과는
이번 보정이 변경하지 않는 oversized 일반 표 경로의 별도 관찰이며, 이 검토로 해당 경로
전체의 정확성을 보증하지 않는다. 수정된 프로브와 성공 로그는 같은 디렉터리의
`boundary_probe.rs`, `boundary-probe-scoped.log`다.

### 보류 2: baseline overflow 13.3px의 실질 영향

원 fixture의 paragraph 2는 공백 세 개(`"   "`)와 표 control 하나를 가진 host다.
진단은 이 host의 `PartialParagraph` 후행 줄에서 발생한다. 실제 render-tree에서 본문 표
하단은 **1003.213px**, 본문 하단은 **1009.093px**, 마지막 비공백 글자의 bbox 하단은
**988.607px**로 모두 본문 영역 안에 있다. overflow 위치에 비공백 TextRun은 없었다.

리베이스 후 대표 PNG도 직접 열어 표의 마지막 행과 하단 테두리가 남아 있고, 제목이 표시되며
결재표와 본문 표가 겹치지 않는 것을 확인했다. 따라서 이 fixture의 13.3px 진단은 가시
콘텐츠 clipping의 증거가 아니며 제목 복원 PR의 blocker에서 해제한다. **경고 자체를
없앤 것은 아니고**, 다른 문서의 overflow까지 무해하다고 일반화하지 않는다.

### 리베이스 후 실행한 검증

| 항목 | 결과 |
| --- | --- |
| `cargo build --locked --workspace --target-dir target/pr-review` | exit 0 |
| suite prepare / manifest check | 각 exit 0 |
| 기존 host heading focused 회귀 | 2 passed, 207 filtered/skipped, exit 0 |
| baseline / flow-first / decoration-first / two-decorations 프로브 | 네 구성 모두 제목 1회, exit 0 |
| decoration 하단 경계 프로브 | 6개 높이의 12개 구성 및 bbox 비교 통과, exit 0 |
| native / WASM lib / workspace all-targets Clippy | 각각 `--locked`, `-D warnings`, exit 0 |
| 실제 visual sweep 및 대표 PNG 직접 확인 | 1/1 page complete, missing 0, flagged 0 |

검증 로그와 프로브는 `output/pr_6971_20260910/rebased/`에만 두며 제출하지 않는다.
전체 nextest와 Native Skia 전체 묶음의 이전 통과 기록은 **리베이스 전 대상의 결과**다.
이번에는 빌드, focused 회귀, Clippy와 핵심 시각/경계 검증을 재실행했으며, 새 기준에서 전체
회귀까지 다시 통과했다고 기록하지 않는다. push 전 필요한 전체 검증과 최신 대상 Full CI는
별도 게이트로 남긴다.

### 최종 증적

기존 Hancom PDF를 그대로 재사용했으며 재변환하지 않았다. 입력과 PDF SHA-256 및 생성
metadata는 아래 기록과 같다. 추가로 입력 SHA-1은 `a336e5f4fc2b78e0e33a799d346e6e86e07cdf35`,
기준 PDF SHA-1은 `86df8dbe3f42d33550a6e06048ff214698e37a95`다.

```bash
venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin target/pr-review/debug/rhwp \
  --hwp samples/issue6969/synth_decoration_host_title.hwp \
  --pdf pdf/synth_decoration_host_title-2020.pdf \
  --key pr6971-rebased --dpi 96 --svg-rasterizer webfont \
  --out output/pr_6971_20260910/rebased/visual-sweep
```

대표 PNG는 [pr6971-maintainer-p001-review.png](../assets/pr_6971_20260910/pr6971-maintainer-p001-review.png)이며
이번 리베이스 후 산출물로 교체했다. 원 산출물은
`output/pr_6971_20260910/rebased/visual-sweep/pr6971-rebased/review/review_001.png`다.
pixel match 97.96066%, visual accuracy proxy/ink match 12.38738%이며 도구 라벨과 본문을
직접 확인했다. 결재표의 미세한 위치, 선과 글꼴 굵기는 residual이다. 이 낮은 ink match를
전체 fidelity 통과로 해석하지 않고 제목 복원과 겹침/잘림 여부로 판정 범위를 한정한다.

### Merge 후 contributor PR comment 계획

- 문서 비교 정본: [PDF/SVG visual sweep 가이드](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- 원 PR head, 실제로 고정한 보정 commit, 최신 검증 대상 SHA와 CI를 구분해 기록한다.
- 실제 확인 범위는 p001, 1/1 page, flagged 0, pixel match 97.96066%, visual accuracy proxy 12.38738%다.
- 사람의 판정은 제목 복원, 원 fixture의 표 겹침/하단 잘림 없음이다. 선/글꼴 residual과 비가시
  공백 줄 overflow가 남는다는 한계를 함께 적는다.
- 이미지 형식:
  `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6971_20260910/pr6971-maintainer-p001-review.png`.
- asset이 merge commit을 통해 devel에 반영되고 최신 head의 merge가 완료된 뒤에만 사용자 승인
  범위에서 `--body-file`로 게시한다. 게시 뒤 API로 Markdown과 이미지 URL을 확인한다.
- 최종 제출 증적은 기존 입력, 기존 기준 PDF와 위 대표 PNG다. 이전 PNG를 별도 중복 보관하지
  않고 raw SVG/JSON/HTML/raster/log/probe/binary는 `output/`에 남긴다.

## 리베이스 전 검증 기록 (2026-09-10 20:41 KST)

**당시 판정: 머지 보류.** 빌드나 Clippy 실패 때문에 보류하는 것은 아니다. 아래 구현 검증은
모두 통과했고 실제 비교 이미지에서 제목 복원도 확인했다. 다만 페이지 하단의 여러 decoration
table 경계 조건은 아직 직접 검증하지 않았으며, baseline 프로브의 overflow 진단도 해소 또는
무해성 확인을 하지 않았다. 이 두 항목을 제목 중복 검사 통과로 대체하지 않는다.

아래의 이전 `미검증` 및 최초 검토 기록은 당시 상태다. 현재 실행 여부와 결과는 이 절을 우선한다.

### 검증 대상 식별

- 브랜치: `review/davindev-pr6971-20260910`.
- upstream 기준: `4e0ce9283086ddd33e3660cf3927db8175384a43`.
- 원 PR head: `5c68bbd909be51a059121096bda1ae7617efcdd6`.
- 로컬 cherry-pick: `557a4f2e43c9c72cce7d7f3af6e1db5901c72d47` 위의 미커밋 메인터너 보정.
- `src/renderer/typeset.rs` SHA-256: `a5930bf00c76768758aa8d14445ac4ce868bf722ba931489744059f5df25ee95`.
- 시각 검증 바이너리 `target/pr-review/debug/rhwp` SHA-256:
  `f0dcb65f53a53a19466d06a7a886d7a062484610e1aaee5b301a28ed21d1694b`.
- commit SHA만으로 미커밋 수정본을 식별하지 않는다. 이번 작업에서 commit, push, GitHub 게시,
  PR 생성 또는 merge를 수행하지 않았다.

### 실행 결과

| 항목 | 결과 | 비고 |
| --- | --- | --- |
| integration suite prepare / manifest check | 통과 | 파생 suite는 검증용이며 제출 대상이 아님 |
| `cargo fmt --all` / fmt check | 통과 | 각 exit 0 |
| workspace build | 통과 | exit 0, 91초 |
| 기존 host heading 회귀 | 통과 | 2 passed, 202 filtered/skipped |
| native Clippy | 통과 | `--locked -- -D warnings` |
| WASM lib Clippy | 통과 | `wasm32-unknown-unknown`, `-D warnings` |
| workspace all-targets Clippy | 통과 | `--workspace --all-targets`, `-D warnings` |
| 전체 nextest 회귀 | 통과 | exit 0, 779초, 아래 전체 명령 사용 |
| native-skia lib 테스트 | 통과 | 로그의 4개 summary 합계 4,112 passed, 13 ignored, 0 failed |
| Skia missing-picture placeholder | 통과 | 2 passed, 185 filtered/skipped |
| Skia direct-PDF export | 통과 | 4 passed, 178 filtered/skipped |
| 추가 host-text 프로브 | 통과 | 아래 4가지 구성 모두 제목 1회 |

```bash
cargo nextest run --locked --cargo-profile release-test \
  --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
```

실행 명령은 `output/pr_6971_20260910/validation/run.sh`, 단계별 종료 코드와 소요 시간은
`output/pr_6971_20260910/validation/results.tsv`에 있다. 로그 및 임시 프로브는 같은
`output/` 아래에만 두며 커밋하지 않는다. 종료 후 Cargo/Rust 검증 프로세스가 남지 않았음을 확인했다.

추가 프로브는 처음에 비공개 import 경로로 컴파일이 실패했다. 사용자 승인 후
`rhwp::model::control::Control`로 수정했고, 재컴파일과 실행 모두 exit 0이었다.
제품 코드나 tracked test source는 이 프로브 오류 수정 과정에서 바꾸지 않았다.

| 임시 IR 구성 | 제목 렌더 횟수 | 검사 범위 |
| --- | --- | --- |
| baseline | 1 | 원 fixture |
| flow-first | 1 | 일반 표 다음 decoration 표 |
| decoration-first | 1 | decoration 표 다음 일반 표 |
| two-decorations | 1 | decoration 표 2개 |

이 프로브는 page 0 render-tree의 제목 횟수를 검사한다. 일반 표와 decoration 표의 양방향
순서에서 제목 중복이 없음을 확인했지만, 모든 표 배치나 페이지 분할을 검증하는 테스트는 아니다.
복제 표를 같은 위치에 넣은 구성에는 `LAYOUT_TABLE_OVERLAP` 진단이 있고, 원 fixture 구성에도
`LAYOUT_OVERFLOW` 13.3px 진단이 있다. 프로브 exit 0을 이 진단들까지 해소됐다는 뜻으로 쓰지 않는다.

### 시각 증적과 직접 확인

- 입력: `samples/issue6969/synth_decoration_host_title.hwp`, SHA-256
  `de26419c269764722f0db21f8d1c1fac779883dba84660909576f6b6ce5035f7`.
- 기준 PDF: `pdf/synth_decoration_host_title-2020.pdf`, SHA-256
  `53ac43d4680d9623d32316aaa4572d7bcb3122d9991a2538eb6caf7fd1df7eba`.
- 기준 생성: Hancom MCP `engine=2020`, job `77df69c6-93f2-40fa-8b37-dbdb144672f3`,
  terminal succeeded 후 다운로드. 생성된 PDF를 재사용했으며 비교마다 재출력하지 않았다.
- `pdfinfo`: 1 page, 595 x 841 pt, PDF 1.6, Creator `Hwp 2022 0.0.0.0`,
  Producer `Hancom PDF 1.3.0.550`. 입력 lastSavedWith는 Hancom Office 2022다.
- 최종 비교 PNG: [pr6971-maintainer-p001-review.png](../assets/pr_6971_20260910/pr6971-maintainer-p001-review.png).

```bash
venv/bin/python scripts/visual_sweep.py \
  --rhwp-bin target/pr-review/debug/rhwp \
  --hwp samples/issue6969/synth_decoration_host_title.hwp \
  --pdf pdf/synth_decoration_host_title-2020.pdf \
  --key pr6971 --dpi 96 --svg-rasterizer webfont \
  --out output/pr_6971_20260910/visual-sweep
```

`run_state=complete`, 요청/완료 1/1 page, missing 0이며 SVG, render-tree, PDF와 raster가
각 1 page다. 비교 PNG를 직접 열어 제목 표시, 원 fixture에서 제목 중복 없음, 상단 결재표와
본문 표가 겹치지 않는 것을 확인했다. 결재표의 미세한 위치와 선/글꼴 굵기는 기준과 다르다.
pixel match 97.96066%와 ink match 12.38738%를 함께 기록하며, 큰 흰 배경이 포함된 pixel
match만으로 시각 완전 일치를 주장하지 않는다.

보조 `fidelity_compare.py`는 exit 0 및 run-state complete였지만 `page-count-ledger.tsv`가
`rhwp_svg=0`, `rhwp_render_tree=1`, `reference_pdf=1`을 기록했다. 이 불일치는 미해결이며
보조 harness를 독립적인 시각 통과 근거로 사용하지 않는다. 위 visual sweep의 실제 1-page
산출물과 직접 열어 본 PNG가 이번 제목 복원 확인 근거다.

### 남은 확인과 원 PR comment 계획

1. 페이지 하단에서 여러 decoration 표가 같은 anchor를 공유할 때 불필요한 분할이 없는지 직접 검사한다.
2. baseline overflow 13.3px가 실제 clipping/배치 결함인지 확인하고, 기존 동작인지 PR 영향인지 구분한다.
3. 수용 판정 후 게시할 comment에는 원 head, 메인터너 보정 commit, 검증 결과, 잔여 차이와 위 대표 PNG를 기록한다.

현재 comment 초안의 결론은 "메인터너 보정본은 빌드/전체 회귀/Clippy/Skia 및 제목 1회 검사 통과,
제목 복원 시각 확인. 페이지 하단 경계와 overflow 진단의 영향은 추가 확인 필요"이다.
실제 correction commit 또는 merge SHA가 아직 없으므로 이를 만들어 적거나 close/merge 완료를
미리 기록하지 않는다. 최종 보관 대상은 입력, 기준 PDF와 대표 PNG이며 raw SVG, raster,
ledger, JSON, HTML, log, probe 및 실행 바이너리는 `output/`에 남긴다.

## 이전 검토 기록 (아래 미검증 표기는 당시 상태)

## 메인터너 보정 적용 (2026-09-10, 미검증)

- 검토 branch의 체리픽 commit `557a4f2e43c9c72cce7d7f3af6e1db5901c72d47` 위에 미커밋 보정을 적용했다.
- 데코레이션 단축에서는 host 텍스트의 방출 필요 여부만 기록한다. 일반/TAC 표 경로를 사용한 문단은
  해당 경로의 기존 텍스트 방출 및 layout fallback에 소유권을 두어 별도 전체 줄 방출을 억제한다.
- 데코레이션 표만 있는 host는 전체 control 루프가 끝난 뒤, 기존 문단 높이 조정 전에 텍스트를 한 번
  방출한다. 이때의 높이 전진이 후행 형제 데코레이션 표의 앵커/분할 계산에 끼어들지 않게 했다.
- 아래 발견 사항 1/2에 대응한 구현 변경이며, 재현 입력을 실행해 해결을 확증한 상태는 아니다.
- 빌드, 포맷, focused/전체 회귀, Clippy 및 시각 검증은 실행하지 않았다. 테스트 파일도 변경하지 않았다.
- 최종 판정은 검증 전이므로 **머지 보류**를 유지한다. commit/push/새 PR 생성/merge는 하지 않았다.
- [보정 계획 및 적용 기록](pr_6971_review_impl.md)에 범위와 남은 검증을 기록했다.


## 최초 검토 판정: 머지 보류 (과거 기록)

2026-09-10 최신 `upstream/devel`에 원 PR을 충돌 없이 체리픽한 뒤 정적 코드 검토를 수행했다.
아래 두 경로는 코드상 발견 사항이며, 추가 입력을 실행하여 재현했다고 주장하지 않는다.
원 head의 CI 성공과 체리픽 head의 로컬 검증은 구분한다.

## 접수 및 적용

| 항목 | 내용 |
| --- | --- |
| PR | [#6971](https://github.com/edwardkim/rhwp/pull/6971) |
| 관련 이슈 | [#6969](https://github.com/edwardkim/rhwp/issues/6969) |
| 작성자 | `davindev` |
| 원 source branch | `kidsnote/rhwp:fix/pk39448-decoration-host-text` |
| 원 head | `5c68bbd909be51a059121096bda1ae7617efcdd6` |
| 검토 base | `4e0ce9283086ddd33e3660cf3927db8175384a43` |
| 검토 branch | `review/davindev-pr6971-20260910` |
| 체리픽 commit | `557a4f2e43c9c72cce7d7f3af6e1db5901c72d47` |
| 적용 결과 | 1 commit, 3 files, 96 insertions, 충돌 없음 |
| reviewer | `jangster77`; REST API로 지정 완료 |
| 원 PR 상태 | 조회 시 OPEN, devel 대상, Draft 아님 |

PR 본문, issue 본문, 일반 코멘트, inline review 코멘트, 제출된 review를 조회했다.
조회 시 PR 코멘트와 review 대화, issue 코멘트는 없었다. 기존 GitHub CLI의 reviewer 지정 명령은
Projects classic GraphQL 오류로 실패하여 REST requested_reviewers endpoint로 처리했다.
원 fork 또는 upstream 원격 branch에 코드를 push하거나 새 통합 PR을 생성하지 않았다.

## 변경 의도

글앞으로/글뒤로 표를 `PageItem::Shape`로 처리하는 단축 경로가 host 문단의 제목 텍스트를
방출하지 않는 문제를 고친다. 표 처리 뒤 `PartialParagraph(0..total_lines)`를 추가하고 줄 높이를
흐름에 반영한다. 추가된 `host_text_emitted`는 같은 루프의 데코레이션 단축 사이 중복만 막는다.

## 발견 사항

### 1. [P1] 일반 표와 데코레이션 표가 섞이면 host 텍스트를 중복 방출한다

- 위치: [typeset.rs:20036](../../../src/renderer/typeset.rs#L20036),
  [기존 pre-text 방출:21381](../../../src/renderer/typeset.rs#L21381).
- 같은 문단에 일반 비-TAC 표와 데코레이션 표가 있고 일반 표가 먼저 놓이는 경우를 처리하지 않는다.
  예를 들어 양수 vertical offset을 가진 Square 표가 먼저 있으면 기존 `place_table_with_text`가
  `is_first_placed` 조건에서 host 텍스트 전체를 `PartialParagraph`로 발행할 수 있다.
- 이 경로는 새 `host_text_emitted`를 갱신하지 않으며, 해당 플래그를 인자로 받지도 않는다.
  뒤의 데코레이션 단축은 여전히 false인 플래그를 보고 같은 `0..total_lines`를 다시 발행하고
  `current_height`도 추가로 전진시킨다. 제목 PageItem 중복과 흐름 과다 소비가 발생하는 경로다.
- 보정 방향: host 텍스트 소유/방출 상태를 데코레이션 단축과 일반/TAC 표 경로에서 공유하거나,
  이미 발행된 줄 범위를 고려해 미발행 텍스트만 처리한다. mixed-control 순서를 양방향으로 고정해야 한다.

### 2. [P2] 첫 제목 방출이 뒤 데코레이션 표의 페이지 경계 계산을 바꾼다

- 위치: [흐름 전진:20044](../../../src/renderer/typeset.rs#L20044),
  [후속 표의 anchor 계산:19959](../../../src/renderer/typeset.rs#L19959).
- 제목 방출은 전체 control 루프 종료 뒤가 아니라 첫 번째 해당 표의 단축 안에서 이루어진다.
  같은 문단/앵커 줄에 두 번째 데코레이션 표가 있으면, 그 표의 `anchor_y` 및 `room` 계산은
  제목 높이만큼 증가한 `st.current_height`를 사용한다.
- 코드 경로 예: 원 앵커의 잔여 공간이 20px, 제목 advance가 10px, 뒤 데코레이션 표가
  6px씩 3행인 경우 실제 앵커에서는 18px 표가 들어간다. 하지만 첫 제목을 방출한 뒤 계산하는
  잔여 공간은 10px가 되어 `pending_overlay_continuations`와 `current_column_overlay_cuts`에
  불필요한 분할을 등록할 수 있다. 이 수치는 정적 경로 설명용이며 fixture 실측값이 아니다.
- 보정 방향: 모든 co-anchored 표의 앵커/분할 계산이 제목 방출 전 문단 기준을 유지하게 한다.
  host 텍스트를 한 번만 발행하는 것과 형제 표의 좌표 기준을 보존하는 것은 별도의 조건이다.

### 테스트와 시각 증적의 공백

[추가 테스트](../../../tests/issue_1755_host_heading_pre_emit.rs#L60)는 제목 문자열의 존재만 확인한다.
`items.contains("pi=0")`는 표의 Shape 항목만 있어도 참일 수 있고, 마지막 `rendered.contains(&title)`는
제목의 중복 횟수, 뒤 표의 좌표/잘림/쪽수, 일반 표와의 혼재를 검사하지 않는다.
위 두 경로를 고정할 mixed-control 및 복수 데코레이션 표 사례가 필요하다.

렌더링과 페이지 배치가 변경되므로 [시각 증적 정책](../../manual/pr_review/visual_fixture_evidence.md)에 따른
직접 비교가 수용 전 필요하다. 이번 검토에서는 해당 fixture의 한컴 기준 PDF를 확보하거나
비교 PNG를 생성하지 않았으며, 원 PR의 좌표 실측/사내 지표 주장을 독립 시각 검증으로 승격하지 않는다.

## 원 CI와 로컬 검증 구분

원 head `5c68bbd909be51a059121096bda1ae7617efcdd6`에서 다음 결과를 확인했다.

| 검사 | 결과 |
| --- | --- |
| [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/34426867348/job/102716683993) | SUCCESS |
| [Lint](https://github.com/edwardkim/rhwp/actions/runs/34426867348/job/102713921540) | SUCCESS |
| [Analyze Rust](https://github.com/edwardkim/rhwp/actions/runs/34426867390/job/102713944426) | SUCCESS |
| [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34426867000/job/102713896322) | SUCCESS |

이번 요청에서는 체리픽과 정적 검토만 수행했다. 체리픽 head의 빌드, focused/전체 회귀,
Clippy, 시각 출력 비교는 실행하지 않았다. 이전 PR #6985 검증 결과나 기존 바이너리를
PR #6971의 검증으로 사용하지 않는다.

## 재현 입력과 증적 정책

- 입력: `samples/issue6969/synth_decoration_host_title.hwp`, 22,528 bytes.
- SHA-256: `de26419c269764722f0db21f8d1c1fac779883dba84660909576f6b6ce5035f7`.
- 원 PR에 포함된 fixture를 그대로 적용했다. 별도 PDF/PNG/로그 등 중간 산출물은 추가하지 않았다.
- 향후 검증 로그와 중간 SVG/JSON/PNG는 `output/`에 두고, 기준 PDF와 최종 코멘트용 대표 PNG만
  정책에 따라 영구 보존한다.

## 보류 해제 조건

1. mixed-control host 텍스트 소유권 및 형제 데코레이션 표의 앵커 계산을 보정한다.
2. 해당 경계 사례의 회귀 검증과 변경 범위별 로컬 게이트를 실제 실행한다.
3. 한컴 기준과 제목/표 좌표/페이지 경계의 직접 시각 비교를 수행한다.
4. 보정 SHA, 결과, 최종 대표 증적과 contributor 코멘트 계획을 이 기록에 반영한다.

현재 GitHub approve/comment/close/push/merge 또는 새 PR 생성은 하지 않았다.
