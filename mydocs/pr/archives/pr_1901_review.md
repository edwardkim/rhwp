# PR #1901 리뷰 — #1893 빈 누름틀 HWPX 라운드트립 렌더 분기

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1901 |
| 제목 | Task #1893: 빈 누름틀(CLICK_HERE) 서식 HWPX 라운드트립 렌더 분기 수정 — 3중 결함 체인 |
| 작성자 | planet6897 |
| base | `devel` |
| head | `planet6897:pr/devel-1893` |
| 최종 head SHA | `433fb908455d670808333a91c468f4331ff0bb58` |
| merge commit | `ced8f37751be73191157982ca07a695a70c6b586` |
| 관련 이슈 | #1893 |
| 규모 | 8 files, +340 / -1 |
| mergeable | merge 전 최종 확인: `MERGEABLE`, `CLEAN` |
| CI | 최종 head 기준 CI/CodeQL/Render Diff/Canvas visual diff 통과, WASM Build skip |

## 변경 범위

- `src/document_core/commands/document.rs`
  - 빈 누름틀 초기 안내문 삭제 후 `char_offsets`, `char_count`, `char_shapes` 를 원본 오프셋 스냅샷 기준으로 함께 수술한다.
  - 삭제 구간 뒤 오프셋은 삭제 문자 UTF-16 폭만큼 감산하고, 삭제 구간 내부 `CharShape` 경계는 zero-width run 시작점으로 고정한다.
- `src/serializer/hwpx/section.rs`
  - 문단 끝 또는 같은 슬롯의 0-length `fieldBegin`/`fieldEnd` 순서를 begin 직후 end 로 맞춘다.
  - fieldEnd 가 다음 fieldBegin 슬롯의 8유닛 gap 을 소진해 LIFO 페어링이 교차하는 경로를 막는다.
- `tests/issue_1893.rs`
  - `samples/issue1893_clickhere_field_roundtrip.hwpx` 를 `Via::Hwpx` 로 라운드트립했을 때 페이지 수 1, 최대 변위 1px 이하, 구조 불일치 없음으로 고정한다.
- `samples/issue1893_clickhere_field_roundtrip.hwpx`
- `pdf/issue1893_clickhere_field_roundtrip-2022.pdf`
- task plan/report 문서 3건

## 렌더 영향 및 visual sweep 판정

PR 본문과 테스트가 HWPX 라운드트립 렌더 분기를 직접 다루고, HWPX 샘플과 한컴 2022 기준 PDF를 함께 추가했으므로 visual sweep 대상이다.

다만 기준 PDF는 누름틀 placeholder 표시를 보존하지 않는 출력물이다. 따라서 PDF 대조에서 rhwp 쪽에 빨간 누름틀 안내문이 보이는 것은 이 PR 의 실패로 보지 않고, `DocumentCore load -> export_hwpx_native -> reparse` 경로의 self-roundtrip 렌더 자기정합을 중심으로 판단했다.

### 원본 HWPX vs 기준 PDF

명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1901-issue1893 \
  --hwp samples/issue1893_clickhere_field_roundtrip.hwpx \
  --pdf pdf/issue1893_clickhere_field_roundtrip-2022.pdf \
  --page 1 \
  --out output/pr1901-visual
```

산출물:

- compare: `output/pr1901-visual/pr1901-issue1893/compare/compare_1893.png`
- overlay: `output/pr1901-visual/pr1901-issue1893/overlay/overlay_1893.png`
- review: `output/pr1901-visual/pr1901-issue1893/review/review_1893.png`
- 증적 asset: `mydocs/pr/assets/pr_1901_issue1893_source_vs_pdf_review_p001.png`

결과:

- SVG/PDF 페이지 수: 1 / 1
- 자동 후보: `flagged=0/1`
- `visual_accuracy_proxy_percent`: `6.89631%`

사람 판정 메모:

- 기준 PDF는 누름틀 placeholder 표시를 보존하지 않으므로 빈 누름틀 안내문이 보이지 않는다.
- rhwp 렌더의 빨간 안내문/placeholder 계열 텍스트는 PR 실패가 아니라 PDF 출력 기준의 표현 한계로 판단한다.
- 표/필드 배치와 1쪽 페이지 구성은 기준 PDF와 비교해 merge blocker 를 보이지 않았다.

### 저장 HWPX vs 기준 PDF

명령:

```bash
target/debug/rhwp export-hwpx \
  samples/issue1893_clickhere_field_roundtrip.hwpx \
  output/pr1901-roundtrip/issue1893_clickhere_field_roundtrip.saved.hwpx \
  --verify --verify-pages

python3 scripts/task1274_visual_sweep.py \
  --key pr1901-issue1893-saved \
  --hwp output/pr1901-roundtrip/issue1893_clickhere_field_roundtrip.saved.hwpx \
  --pdf pdf/issue1893_clickhere_field_roundtrip-2022.pdf \
  --page 1 \
  --out output/pr1901-visual-saved
```

산출물:

- compare: `output/pr1901-visual-saved/pr1901-issue1893-saved/compare/compare_1893.png`
- overlay: `output/pr1901-visual-saved/pr1901-issue1893-saved/overlay/overlay_1893.png`
- review: `output/pr1901-visual-saved/pr1901-issue1893-saved/review/review_1893.png`
- 증적 asset: `mydocs/pr/assets/pr_1901_issue1893_saved_vs_pdf_review_p001.png`

결과:

- `export-hwpx --verify --verify-pages`: 1쪽, IR diff 없음
- SVG/PDF 페이지 수: 1 / 1
- 자동 후보: `flagged=0/1`
- `visual_accuracy_proxy_percent`: `6.89631%`

사람 판정 메모:

- 저장 HWPX 도 원본 HWPX 와 동일하게 빨간 누름틀 안내문을 유지한다.
- 기준 PDF는 누름틀 표시를 보존하지 않으므로 이 차이는 visual sweep 실패로 보지 않는다.
- 저장본도 표/필드 배치와 페이지 구성 관점에서 통과로 판정한다.

## 라운드트립 자기정합 검증

```bash
target/debug/rhwp render-diff \
  samples/issue1893_clickhere_field_roundtrip.hwpx \
  --via hwpx

target/debug/rhwp hwpx-roundtrip \
  samples/issue1893_clickhere_field_roundtrip.hwpx \
  -o output/pr1901-roundtrip
```

결과:

- `render-diff --via hwpx`: 페이지 수 A=1/B=1, 최대 변위 `0.00px`, 임계 초과 0, 구조 불일치 0, PASS
- `hwpx-roundtrip`: PASS, diff=0, r2=0

## 로컬 검증

새 PR review 시작 전 cargo cache 비대화 영향을 줄이기 위해 `/Users/tsjang/rhwp/target` 하위 항목을 삭제한 뒤 순차 실행했다.

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
- `cargo test --profile release-test --tests`: 통과. `svg_snapshot` 8 tests 포함 통과
- `cargo fmt --check`: 통과
- `git diff --check`: 통과
- `cargo clippy --all-targets -- -D warnings`: 통과

## 검토 결과

### 1. PR 핵심인 HWPX self-roundtrip 렌더 분기는 해소됐다

`render-diff --via hwpx` 와 신규 `issue_1893` 테스트 모두 1쪽, 최대 변위 0.00px, 구조 불일치 없음으로 통과했다. `hwpx-roundtrip` 도 diff=0으로 통과해 `DocumentCore load -> export_hwpx_native -> reparse` 경로의 자기정합은 검증됐다.

### 2. 보정 근거는 문서 내부 속성에 기반한다

보정은 특정 파일명, 페이지 번호, 이슈 번호, 임의 계수로 맞추는 분기가 아니라 입력 문서에서 읽은 `field_ranges`, `char_offsets`, `char_shapes`, control slot 순서와 UTF-16 폭을 기준으로 한다. 렌더/파서 보정 PR 검토 기준상 하드코딩성 보정으로 보지 않는다.

### 3. 기준 PDF 대조 visual sweep 도 PR 범위 기준 통과다

한컴 2022 기준 PDF는 누름틀 placeholder 표시를 보존하지 않는다. rhwp 쪽에 빨간 누름틀 안내문/placeholder 계열 텍스트가 보이는 것은 PDF 출력 기준의 한계로 판단하며, 이번 PR 의 visual sweep 실패나 후속 후보로 보지 않는다. 이번 PR 은 #1893 에서 표면화된 roundtrip 렌더 분기를 닫는 변경이며, 표/필드 배치와 페이지 구성 기준으로 visual sweep 통과로 정리한다.

### 4. update branch 후 최종 CI 통과

최종 head `433fb908455d670808333a91c468f4331ff0bb58` 기준 GitHub Actions 는 CI/CodeQL/Render Diff/Canvas visual diff 모두 통과했다. 로컬 merge simulation 도 `Already up to date` 로 충돌이 없었다.

## 최종 권고

merge 완료됐다.

- merge commit: `ced8f37751be73191157982ca07a695a70c6b586`
- merge 방식: admin merge
- merge 시각: 2026-07-04T12:28:31Z

merge 전 최종 조건:

- 최종 head SHA `433fb908455d670808333a91c468f4331ff0bb58` 확인
- GitHub required checks 통과 확인

merge 후 후속:

- #1893 auto-close 완료를 확인했다. docs-only PR merge 후 asset 링크와 함께 후속 검증 코멘트를 남긴다.
- PR #1901 에 감사/검증 코멘트를 남긴다.
- 기준 PDF는 누름틀 표시를 보존하지 않는다는 점을 PR/issue 후속 코멘트에 짧게 설명한다.
- review 문서와 대표 visual asset 2건은 docs-only PR 로 반영한다.
