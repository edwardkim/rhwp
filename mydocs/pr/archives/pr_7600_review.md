# PR #7600 리뷰 — dyad 붙여쓰기와 양쪽 화살표

## 최종 판정

**승인.** `dbb87b98047ffb73ff038374a59501495a47bdf2`의 dyad 붙여쓰기 분리와 양쪽 화살표 수정을 검증한 최소 재현 범위에서 수용한다. Native/fresh WASM의 전체 1쪽 실루엣 일치율 최저값은 각각 **96.62219%**, gate는 `passed`다. 최신 head CI 성공과 current-base 충돌 없음도 재확인했다. 작업지시자가 GitHub 승인·병합·후속 처리를 승인하여 병합을 완료했다. 아래 CanvasKit 진단 한계와 남은 글꼴·화살촉 차이를 포함한 판정이며, 원본 전체 문항 개선을 주장하지 않는다.

## 접수와 라우팅

- base route: maintainer 일반 (`maintainer_general.md`).
- modifiers: 접수·리뷰, 로컬 검증, 시각·fixture 증적, 첫 기여자, 오래된 base 확인.
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, `maintainer_general.md`, `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `first_time_contributor.md`, `rework_and_exceptions.md`, `review_template.md`. 빌드·시각 작업은 `dev_environment_guide.md`, `visual_verification_governance.md`, `visual_sweep_guide.md`도 적용했다.
- PR: [#7600](https://github.com/edwardkim/rhwp/pull/7600), 작성자 `Phantomn`, FIRST_TIME_CONTRIBUTOR. 기존 merged PR 검색 결과 0건.
- reviewer는 이미 `jangster77`이 지정되어 있어 중복 assign하지 않았다.
- 관련 이슈: [#7593](https://github.com/edwardkim/rhwp/issues/7593), `closes #7593`. U+2AFD 누락 #7594는 범위 밖.
- code head: `dbb87b98047ffb73ff038374a59501495a47bdf2`, commit 1개, 18 files / +121 −6.
- 기준 devel: `fe9ffd27e91d8d260fbccde5ff68aa08aadf94ca`. `git merge-tree --write-tree upstream/devel upstream/pr7600-head` exit 0, tree `c9bbd6ec57e96c1b1b70d8d4859c3aac0c2b58e2`.
- 원 head 그대로 `/tmp/rhwp-pr7600-review-20261007`, branch `review/pr7600-20261007`에서 검토했다. 원 fork를 재작성하거나 최신 devel을 source에 merge하지 않았다.
- 2026-10-07 최종 조회에서 PR은 open / non-draft / mergeable / clean, code head는 동일하다. PR API의 base SHA는 `c167dc6abbebf69546575e2d16d06223791bab82`였으나 base ref는 devel이고, branch API와 fetch 결과의 현재 devel은 위 `fe9ffd27…`다. 충돌 simulation은 현재 devel로 수행했다. 상태값은 merge 직전에 다시 확인한다.

## 변경 범위와 호출 경로

1. `tokenizer.rs:522`의 `GLUE_SAFE`에 `dyad`를 추가했다. `read_command → longest_keyword_prefix → is_glue_safe`의 기존 붙여쓰기 규칙에 참여하며 문서 ID나 임의 수치 조건이 없다.
2. `parser.rs`의 기존 decoration AST와 `layout.rs:1180`의 `layout_decoration` 결과를 유지한다. width는 body.width, 높이·기준선은 기존 장식 메트릭이다.
3. SVG `svg_render.rs:404`, Canvas2D `canvas_render.rs:286`, Skia `equation_conv.rs:663`은 같은 layout의 body 원점·폭으로 장식을 배치한다. `Vec | Dyad`는 기존 Vec의 가로선·오른쪽 화살촉을 사용하고 Dyad에만 대칭인 왼쪽 화살촉을 추가했다. 변경 뒤 원점을 clamp하거나 덮어쓰는 분기는 없다.
4. `src/paint/json.rs:3057`은 기존 Dyad를 `dyad`로 직렬화하고 `canvaskit_policy.rs:1912`는 기존부터 이를 허용했다. Studio `canvaskit-renderer.ts:3605`도 기존 centerX·width·fontSize로 같은 장식을 그린다. 최신 devel의 같은 파일 변경은 뒤의 `renderFormObject`이며 장식 분기와 독립적이다.
5. pagination·rowspan·빈 문단·저장 LineSeg 유효성·baseline·golden 허용치는 바꾸지 않았다.

## 독립 근거와 검증 입력

한컴 한글 2022 `12.0.0.535`에서 저장한 공개 최소 재현 HWPX와 같은 입력의 한컴 PDF를 사용했다. `pdfinfo`의 Creator는 `Hwp 2022 12.0.0.535`, Producer는 `Hancom PDF 1.3.0.545`, 1쪽이다. PDF 글꼴은 HanSantteutDotumRegular와 HyhwpEQ다. 본문의 한컴 화살표를 직접 열어 규칙을 확인했으며 버전 연도 때문에 기준 PDF를 재생성하지 않았다.

| 역할 | 저장소 경로 (`mydocs/pr/assets/issue_7593_dyad_arrow/` 아래) | SHA-256 |
| --- | --- | --- |
| 입력 | `repro-dyad-min.hwpx` | `c8605b6d47949a402de1955df3dd00a3dc2ae56dbab822d12b512fc108d4198d` |
| 독립 PDF | `repro-dyad-min.hancom.pdf` | `103c6ffb34e64bad8992ce6b6284faa7d00a9d82095757ff0ad335ca4ccbe838` |

두 파일은 원 code head에 포함된다. 입력의 저장 메타데이터와 네 script(`rm dyadAB`, `rm dyad AB`, `dyad{rm AB}`, `dyad{{rm AC} it } BOT dyad{{rm BD} it }`)를 직접 확인했다. 입력·PDF 및 작성자 Native/fresh WASM review·overlay 4장의 실행 파일 바이트가 각각 code head의 commit blob과 동일함을 확인했다. SHA는 ignored `build-receipt.json`과 원 PR asset에 연결한다.

## 실행한 검증

| 검증 | 결과·근거 |
| --- | --- |
| head Full CI | [37317981931](https://github.com/edwardkim/rhwp/actions/runs/37317981931), Build & Test / Archive A–D / Lint / Native Skia / Frontend package success |
| 별도 검사 | CodeQL Actions, Render Diff, Adapter inter-diff, Proptest success. CI Impact Policy success; CodeQL aggregate neutral는 선택한 언어 외 두 configuration 부재 경고 |
| focused Rust | `node scripts/run-rust-test.mjs issue_7593_eq_dyad_glue_and_arrow -- --cargo-profile release-test --target-dir target/pr-review`: 3 PASS, 해당 suite의 다른 215개는 필터로 제외 |
| manifest | `--prepare` 뒤 `--check --base-ref fe9ffd27e91d8d260fbccde5ff68aa08aadf94ca` PASS. 파생 파일을 PR에 stage하지 않음 |
| diff | `git diff --check upstream/devel...HEAD` PASS, current-base merge 충돌 없음 |
| Studio | `npm --prefix rhwp-studio test`: 1815 PASS / 0 FAIL / 2 skip |
| Native build | `cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp` PASS |
| Native 시각 | 원 head에서 새 export·Chrome raster·직접 PNG 판독. 현재 canonical Sweep의 `--silhouette-only --png-pair`로 새 PNG를 측정: p1 **96.62219%**, 미달 0, 누락 0 |
| Docker fresh WASM | `docker compose -p rhwp --env-file .env.docker run --rm wasm` PASS. 기존 named volume 재사용, 루트 pkg와 Studio public JS/WASM SHA-256 동일 |
| fresh WASM 시각 | 동일 code head의 새 pkg로 1쪽 전체 재출력·직접 PNG 판독. p1 **96.62219%**, gate `passed`, 90% 미달·누락 0, 문서/기준 PDF/출력 모두 1쪽 |
| TypeScript | fresh pkg 반영 뒤 `npx --prefix rhwp-studio tsc --noEmit -p rhwp-studio/tsconfig.json` PASS. 최초 pkg 미준비로 인한 TS2307과 구분 |
| Canvas 2D 실제 출력 | fresh WASM `HwpDocument.renderPageToCanvas(0, canvas, 1)`을 Chrome 152에서 실행. 네 수식·다섯 양방향 장식·1쪽 표시 직접 확인 |

같은 code head의 성공한 CI와 clean current-base merge, 제품 보정 없음 조건에 따라 전체 nextest·Native Skia 광범위 회귀·lint를 중복 실행하지 않았다(`pr_review_workflow.md` 3.2.2). focused·fresh WASM·직접 시각 검증은 별도로 완료했다. source-side cfg(test) 변경은 없어 unit-tier 추가 검사는 비해당이다. 로컬 nextest는 0.9.137이며 권고 0.9.140 관련 경고가 있었으나 실제 선택된 세 검사는 정상 실행됐다.

수정 전 3 FAIL은 PR 본문의 작성자 실행 기록을 참고했다. reviewer가 수정 전 코드를 재빌드한 결과로 표현하지 않는다. 검사 자체를 읽고 토큰 분리·동일 의미의 붙여/띄어 출력·Vec과의 폭/오른쪽 화살촉 불변 및 Dyad 두 화살촉의 검사 의미를 확인했다.


### 재실행 명령과 증적

Native export는 PR head의 `scripts/visual_sweep.py`, fresh WASM export 및 점수·gate는 현재 devel의 canonical script를 사용했다. 실행 도구와 출력 생성 코드를 구분해 `build-receipt.json`에 SHA를 남겼다. 두 경로의 출력 생성 코드는 모두 위 code head다. 글꼴 디렉터리 `/mnt/c/Windows/Fonts`의 실제 HanSantteutDotum/HYHWPEQ 공급을 확인했다.

```bash
# review worktree에서 Native export
VISUAL_SWEEP_CHROME=/home/edward/.cache/puppeteer/chrome/linux-152.0.7977.54/chrome-linux64/chrome-wrapper RHWP_FONT_PATH=/mnt/c/Windows/Fonts \
  /home/edward/mygithub/rhwp/venv/bin/python scripts/visual_sweep.py \
  --hwp mydocs/pr/assets/issue_7593_dyad_arrow/repro-dyad-min.hwpx \
  --pdf mydocs/pr/assets/issue_7593_dyad_arrow/repro-dyad-min.hancom.pdf \
  --key dyad --pages 1 --rhwp-bin target/pr-review/release-test/rhwp \
  --dpi 96 --embed-fonts=full --font-path /mnt/c/Windows/Fonts \
  --out output/pr-review/pr7600-20261007/native-review
# 현재 devel의 canonical script에 review worktree의 절대 입력/출력 경로를 전달
# fresh WASM은 위 옵션에 --wasm-pkg /tmp/rhwp-pr7600-review-20261007/pkg를 추가
# 현재 devel 루트에서 Native 전체 TSV; WASM은 native를 wasm으로 바꿔 실행
venv/bin/python scripts/visual_sweep.py --silhouette-only \
  --png-pair /tmp/rhwp-pr7600-review-20261007/output/pr-review/pr7600-20261007/native-review/dyad/rhwp_png /tmp/rhwp-pr7600-review-20261007/output/pr-review/pr7600-20261007/native-review/dyad/pdf_png \
  --out /tmp/rhwp-pr7600-review-20261007/output/pr-review/pr7600-20261007/native-scores
```

실행 경로·Chrome wrapper 등 세부 정보는 `native-sweep.log`, `wasm-review/dyad/run_manifest.json`과 `canvas2d-probe.mjs`에 보존했다. 결과는 `native-scores/silhouette.tsv`, `wasm-scores/silhouette.tsv`, 두 `dyad/review/review_001.png`와 `dyad/overlay/overlay_001.png`, `canvas2d-page1.png`/`canvas2d-inspect.png`다. Native/fresh WASM의 `pixel_match`는 99.83678%, `visual_accuracy_proxy_percent`는 8.9658%, 2px 이웃 내용 실루엣은 96.62219%다. 배경 비중과 글꼴 위치에 민감한 보조값을 수용 판정으로 대체하지 않고 직접 판독했다. flagged 후보 0쪽, 누락 0쪽이다.

fresh JS SHA-256 `70cde06a369fa7fd4fc8bc8f3d6acaee158596ba1a116a2c72159002b0b5654e`, WASM `9ae3ba8013684fdd4a1200566d35f80f91eb01546b6603442425913336612c0c`. 빌드 wrapper의 tracked public JS 동기화는 로컬 검증 후 원 head로 복원하며 제품 변경으로 commit하지 않는다.

## 시각 판독과 한계

- 작성자 제공 Native·Skia·CanvasKit 전후 비교와 Native review를 직접 열었다. 새 Native와 fresh WASM 출력도 한컴 PDF와 같은 영역에서 직접 읽었다. 글자 `dyadAB` 노출이 사라지고 AB/AC/BD 위에 폭 전체 양쪽 화살표가 보였다.
- 한컴보다 작은 화살촉, 수식 글리프 모양·세로 위치, BOT 주변 좁은 간격은 남는다. 한컴의 화살촉 픽셀 크기와 완전히 같아졌다는 판정은 하지 않는다.
- Canvas 2D는 실제 bundled Latin Modern Math와 입력 본문 글꼴을 등록한 fresh WASM 브라우저 진단으로 네 행을 직접 확인했다. 진단 페이지의 local-only 글꼴 참조 오류는 실제 공급 파일로 교체해 해소했고 최종 실행에서 font load 오류를 무시하지 않았다. Canvas 2D의 PDF 일치율을 별도로 측정하지 않았으며, SVG의 96.62219%를 이 경로 수치로 쓰지 않는다. 전체 Studio UI E2E는 이번에 수행하지 않았다.
- 작성자의 CanvasKit 직접 호출 비교는 ① 행이 before/after 모두 누락되어 있다. 이는 이번 PR에서 검출된 새 회귀와 구분한다. reviewer는 그 비교만으로 마지막 행의 Studio 표시를 통과로 보고하지 않는다.
- 원 PR의 축소 전 입력 79.8% / 첫 축소 64.3%는 작성자 과거 기록이다. 현재 수용 근거는 commit된 4가지 dyad 사례의 1쪽 최소 재현과 독립 PDF다. 원본 전체 문항이나 마스터쪽 구분선 개선을 주장하지 않는다.
- 로그·새 TSV·PNG는 `output/pr-review/pr7600-20261007/`에 보존했다. 기존 PR 본문에는 원 head SHA로 고정한 Native/fresh WASM review·overlay 4장이 실제 Markdown image로 포함되어 있다.

## 조판 원칙과 주장 대조

| 검토 항목 | 근거 | 판정 |
| --- | --- | --- |
| 근거·일반성 | 독립 한컴 PDF의 직선 양쪽 화살표, 기존 glued keyword 규칙 재사용, 샘플별 예외 없음 | 충족 |
| 측정·배치·backend 일관성 | 기존 공통 LayoutBox의 body 폭·원점; 4 출력 분기에서 Dyad만 왼쪽 화살촉 추가 | 충족 |
| 분할·이어받기 | 컷·페이지 예산·continuation 변경 없음 | 비해당 |
| 줄 소속·저장 LineSeg | 수식 토큰화는 변하지만 저장 줄 수용·문단 줄 나눔·측정 알고리즘 변경 없음 | 비해당 |
| 독립 증거·반례 | 붙여/띄어/단독·중괄호·복합식, Vec 불변 대조, 작성자 red 기록과 reviewer green 분리 | 충족 |
| 기준값 변경 | baseline·golden·허용치 수정 없음 | 비해당 |
| 검증 범위의 시각 조건 | 공개 최소 재현의 1쪽 전체, Native/fresh WASM 각각 96.62219%, 새 PNG 직접 판독·누락 없음 | 충족 |
| Studio CanvasKit 전체 사례 실제 실행 | 코드 분기 및 작성자 전후 이미지 확인; 마지막 행 누락은 before/after 공통. reviewer의 전체 Studio UI E2E 미실행 | 미검증 |
| 입력 commit | 입력·독립 PDF·기존 대표 이미지 4장 모두 code head blob과 바이트 동일 | 충족 |

## 처리 계획

1. fresh WASM·TS·실제 Canvas 2D와 최종 PNG 판독을 완료하여 승인 판정을 기록했다.
2. review 문서를 local commit으로 보존한 뒤 작업지시자 승인에 따라 최신 head/CI/base를 재확인하여 승인·merge를 수행했다. archive와 오늘할일은 maintainer 직접 반영으로 보존한다.
3. post_merge 절차에 따라 아래 승인된 계획의 감사·첫 기여 환영 댓글을 운영 기록 반영 뒤 게시하고 이번 검토 작업공간만 정리한다. contributor fork는 보존한다.

## Merge 후 contributor PR comment 계획

작업지시자가 승인한 후속 게시 계획이다. 실제 merge SHA 확정 뒤 `--body-file`로 한국어 존댓말 문안을 게시하고 API로 한글과 실제 LF, asset URL을 재확인한다. [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결하고 `mydocs/pr/assets/issue_7593_dyad_arrow/{native-review,native-overlay,wasm-review,wasm-overlay}.png`를 merge SHA 고정 raw URL의 Markdown 이미지로 표시한다. 검토자가 새로 측정한 범위와 작성자 첨부 이미지의 기존 측정값을 구분한다. 첫 기여 감사, 붙여쓰기/양쪽 화살표 수용, 남은 글꼴·BOT 간격과 직접 CanvasKit 진단 한계를 함께 명시한다.

게시할 asset URL 형식은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue_7593_dyad_arrow/native-review.png`이며 overlay 및 WASM도 같은 형식이다. 대상 1쪽, flagged 0, 위 보조값과 실루엣 96.62219% 및 직접 판독의 남은 차이를 함께 게시한다. 작성자 이미지의 기존 96.21277%와 reviewer의 새 측정값은 구분한다.

## 병합과 후속 처리

- 사용자 승인 뒤 exact head `dbb87b98047ffb73ff038374a59501495a47bdf2`에 [GitHub APPROVED review](https://github.com/edwardkim/rhwp/pull/7600#pullrequestreview-5441419157)를 게시하고 본문의 한국어·LF를 API 재조회로 확인했다.
- 2026-10-07 20:11 KST에 [merge commit `22ddc2ab81c405b0c3e9b1f25a397a87de109948`](https://github.com/edwardkim/rhwp/commit/22ddc2ab81c405b0c3e9b1f25a397a87de109948)로 devel에 병합했다. `--match-head-commit`으로 검증 head를 고정했다.
- 후속 문서 경로: maintainer 직접 반영. 이 archive review와 `mydocs/orders/20261007.md`만 운영 기록 commit으로 반영한다. 원 fork의 제품 코드·테스트·baseline을 보정하지 않았다.
- 관련 [#7593](https://github.com/edwardkim/rhwp/issues/7593)은 [Close Issues on devel Push](https://github.com/edwardkim/rhwp/actions/runs/37612400718) success에 따라 20:11:35 KST에 CLOSED가 됐다. 수동 close는 불필요하다.
- [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/37612400702): success. `ready=true`, `successful-pr-worker-measurements`, source run `37317981931` attempt 2의 B/C/D 실측으로 46 target을 갱신하고 metrics branch의 `f423af8e` commit push까지 확인했다. 병합 뒤 CI·CodeQL·Adapter·Proptest·Oracle 재실행은 시작하지 않았다.
- 기준 작업공간의 devel을 upstream/devel로 fast-forward했고 merge commit 포함을 확인했다. 영구 입력/PDF 및 대표 이미지 4장도 이 merge SHA에 포함된다.
- issue와 PR에는 같은 증적의 maintainer comment가 아직 없어 후속 댓글을 게시한다. 게시 후 API로 한글·LF·merge SHA 고정 이미지 URL을 재확인한다.
- 이번 작업에서 만든 clean review worktree/branch와 임시 PR fetch ref는 후속 게이트 확인 뒤 정리한다. contributor `Phantomn/rhwp:fix/issue-7593`과 다른 작업공간·공유 target/pr-review·Docker cache는 보존한다. review-only local commit `014d4ef61`의 검토 내용은 이 archive로 이전했다.
