# PR #1901 리뷰 구현 로그

## Stage 1. 메타 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1901
- base: `devel`
- head: `planet6897:pr/devel-1893`
- 최종 head SHA: `433fb908455d670808333a91c468f4331ff0bb58`
- merge commit: `ced8f37751be73191157982ca07a695a70c6b586`
- 관련 이슈: #1893
- reviewer assign: `jangster77`
- GitHub CI: CI/CodeQL/Render Diff/Canvas visual diff 통과, WASM Build skip

## Stage 2. 로컬 fetch 및 merge 시뮬레이션

완료.

```bash
git fetch upstream pull/1901/head:local/pr1901
git switch local/pr1901
git merge upstream/devel --no-commit --no-ff
```

결과:

- `Already up to date.`
- 충돌 없음

## Stage 3. 변경 내용 검토

완료.

확인한 변경 범위:

- `src/document_core/commands/document.rs`
- `src/serializer/hwpx/section.rs`
- `tests/issue_1893.rs`
- `samples/issue1893_clickhere_field_roundtrip.hwpx`
- `pdf/issue1893_clickhere_field_roundtrip-2022.pdf`
- task plan/report 문서 3건

중점 확인:

- `clear_initial_field_texts` 삭제 수술이 `text`/`field_ranges` 뿐 아니라 `char_offsets`/`char_count`/`char_shapes` 까지 함께 맞추는지 확인했다.
- 0-length field 의 begin/end 방출 순서가 자기 begin 직후 end 로 보존되는지 확인했다.
- 신규 테스트는 기준 PDF 직접 일치가 아니라 HWPX roundtrip self-consistency 를 pin 한다.
- 보정 근거는 문서의 field/control/offset 속성에 있고 특정 샘플명·페이지 번호·이슈 번호 기반 분기는 없다.

## Stage 4. 라운드트립 자기정합 검증

완료.

```bash
target/debug/rhwp render-diff samples/issue1893_clickhere_field_roundtrip.hwpx --via hwpx
target/debug/rhwp hwpx-roundtrip samples/issue1893_clickhere_field_roundtrip.hwpx -o output/pr1901-roundtrip
target/debug/rhwp export-hwpx samples/issue1893_clickhere_field_roundtrip.hwpx output/pr1901-roundtrip/issue1893_clickhere_field_roundtrip.saved.hwpx --verify --verify-pages
```

결과:

- `render-diff --via hwpx`: 페이지 수 A=1/B=1, 최대 변위 0.00px, 구조 불일치 0, PASS
- `hwpx-roundtrip`: PASS, diff=0, r2=0
- `export-hwpx --verify --verify-pages`: 1쪽, IR diff 없음

## Stage 5. visual sweep

완료.

원본 HWPX vs 기준 PDF:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1901-issue1893 \
  --hwp samples/issue1893_clickhere_field_roundtrip.hwpx \
  --pdf pdf/issue1893_clickhere_field_roundtrip-2022.pdf \
  --page 1 \
  --out output/pr1901-visual
```

저장 HWPX vs 기준 PDF:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1901-issue1893-saved \
  --hwp output/pr1901-roundtrip/issue1893_clickhere_field_roundtrip.saved.hwpx \
  --pdf pdf/issue1893_clickhere_field_roundtrip-2022.pdf \
  --page 1 \
  --out output/pr1901-visual-saved
```

산출물:

- 원본 compare: `output/pr1901-visual/pr1901-issue1893/compare/compare_1893.png`
- 원본 overlay: `output/pr1901-visual/pr1901-issue1893/overlay/overlay_1893.png`
- 원본 review: `output/pr1901-visual/pr1901-issue1893/review/review_1893.png`
- 원본 asset: `mydocs/pr/assets/pr_1901_issue1893_source_vs_pdf_review_p001.png`
- 저장본 compare: `output/pr1901-visual-saved/pr1901-issue1893-saved/compare/compare_1893.png`
- 저장본 overlay: `output/pr1901-visual-saved/pr1901-issue1893-saved/overlay/overlay_1893.png`
- 저장본 review: `output/pr1901-visual-saved/pr1901-issue1893-saved/review/review_1893.png`
- 저장본 asset: `mydocs/pr/assets/pr_1901_issue1893_saved_vs_pdf_review_p001.png`

결과:

- 원본/저장본 모두 SVG/PDF 페이지 수 1 / 1
- 원본/저장본 모두 자동 후보 `flagged=0/1`
- 원본/저장본 모두 `visual_accuracy_proxy_percent=6.89631%`

사람 판정:

- 기준 PDF는 누름틀 placeholder 표시를 보존하지 않아 빨간 안내문이 보이지 않는다.
- rhwp 쪽의 빨간 누름틀 안내문/placeholder 계열 텍스트는 PDF 출력 기준의 한계로 판단하며 실패로 보지 않는다.
- 표/필드 배치와 페이지 구성 기준으로 visual sweep 통과로 판정한다.
- 이번 PR 의 핵심인 roundtrip self-consistency 도 별도 명령에서 PASS 다.

## Stage 6. 로컬 검증

완료.

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

```bash
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1893
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_258_clickhere_form_mode
env CARGO_INCREMENTAL=0 cargo test --profile release-test --test hwpx_form_roundtrip
env CARGO_INCREMENTAL=0 cargo test --profile release-test serializer::hwpx::section::tests
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
cargo fmt --check
git diff --check
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- `cargo build`: 통과
- `issue_1893`: 1 passed
- `issue_258_clickhere_form_mode`: 13 passed
- `hwpx_form_roundtrip`: 1 passed
- `serializer::hwpx::section::tests`: 47 passed
- `cargo test --profile release-test --tests`: 통과, `svg_snapshot` 포함
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과

## Stage 7. 결론

merge 완료로 정리한다.

근거:

- PR 핵심인 HWPX roundtrip self-consistency 는 `render-diff --via hwpx`, `hwpx-roundtrip`, 신규 테스트에서 모두 통과했다.
- 관련 form/clickhere/serializer 회귀와 전체 integration test, clippy 가 통과했다.
- GitHub Actions 최종 head 기준 통과 상태다.
- admin merge 완료: `ced8f37751be73191157982ca07a695a70c6b586`

주의:

- 기준 PDF는 누름틀 표시를 보존하지 않으므로, PR/issue 후속 코멘트에는 해당 차이가 실패가 아니라 비교 기준의 표현 한계임을 짧게 설명한다.

후속:

- #1893 auto-close 완료를 확인했다. docs-only PR merge 후 asset 링크와 함께 후속 검증 코멘트를 남긴다.
- PR #1901 감사/검증 코멘트
- review 문서/asset docs-only PR 생성 및 merge
