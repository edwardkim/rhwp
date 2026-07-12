# PR #1963 리뷰 - 교차 문단 고아 fieldEnd 슬롯 순서 보정

## 메타

| 항목 | 내용 |
|---|---|
| PR | https://github.com/edwardkim/rhwp/pull/1963 |
| 제목 | Issue #1948: 교차 문단 고아 fieldEnd 갭을 말미 슬롯이 가로채는 순서 재배열 수정 |
| 작성자 | planet6897 |
| base | `devel` |
| head | `fix/1948-fieldend-order` |
| 문서 작성 시점 head SHA | `70fb3eaee1a8801d0af2f4cbaba98cad6e120251` |
| merge commit | `2e1c6929acc2b0f02bb6a2473597c45900fe70f4` |
| mergedAt | `2026-07-05T17:03:43Z` |
| 규모 | 2 files, +24 / -0 |
| 변경 파일 | `src/serializer/hwpx/section.rs`, `samples/hwpx/issue1948_cross_para_fieldend.hwpx` |
| mergeable | merge 전 최종 확인: `MERGEABLE` / `CLEAN` |
| CI | 문서 작성 시점 head 기준 GitHub Actions CI/CodeQL 통과 |
| closingIssuesReferences | 비어 있음. 다만 #1948은 GitHub Actions auto-close로 `2026-07-05T17:03:55Z`에 CLOSED 확인 |

## 관련 이슈 요약

#1948은 HWPX 저장 시 다른 문단에서 시작된 field의 고아 `fieldEnd`와 같은 위치의 말미 slot이 충돌해,
말미 표 slot이 `fieldEnd`의 8 UTF-16 unit gap을 먼저 가로채는 문제다. 이로 인해 재직렬화 후
`char_offsets`와 `char_shapes.pos`가 +8 드리프트하고, IR diff가 발생한다.

## 변경 범위

- `src/serializer/hwpx/section.rs`
  - `render_runs`의 slot 방출 루프에서 현재 gap 소유자가 아직 방출되지 않은 `orphan_field_end`이면
    일반 slot보다 먼저 `fieldEnd`를 방출하도록 순서를 보정했다.
  - 보정 근거는 문서 모델에서 읽은 `para.orphan_field_ends`, `char_idx`, `expected_utf16_pos`다.
  - 특정 파일명, 페이지 번호, issue 번호, 임의 계수로 결과를 맞추는 하드코딩은 보이지 않는다.
- `samples/hwpx/issue1948_cross_para_fieldend.hwpx`
  - 회귀 검증용 축소 샘플을 추가했다.

## 렌더 영향 및 visual sweep 판정

PR은 serializer/roundtrip IR 보존을 직접 수정하고, 샘플과 한컴 기준 PDF가 코멘트로 제공되었으므로
visual sweep 대상이다. 다만 핵심 판단 기준은 렌더링 개선이 아니라 HWPX 재직렬화 후 fieldEnd/slot 순서와
IR offset 보존이다. 따라서 visual sweep은 페이지 수와 큰 배치 회귀 여부를 확인하는 참고 근거로 사용했다.

기준 PDF:

- `pdf/issue1948_cross_para_fieldend-2024.pdf`
- GitHub comment 첨부 파일: `issue1948_cross_para_fieldend-2024.pdf`
- PDF 정보: Hancom PDF, 1쪽, A4

실행 명령:

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1963-issue1948-cross-para-fieldend \
  --hwp samples/hwpx/issue1948_cross_para_fieldend.hwpx \
  --pdf pdf/issue1948_cross_para_fieldend-2024.pdf \
  --page 1 \
  --out output/pr1963-visual
```

산출물:

| 항목 | 경로 |
|---|---|
| compare | `output/pr1963-visual/pr1963-issue1948-cross-para-fieldend/compare/compare_1948.png` |
| overlay | `output/pr1963-visual/pr1963-issue1948-cross-para-fieldend/overlay/overlay_1948.png` |
| review | `output/pr1963-visual/pr1963-issue1948-cross-para-fieldend/review/review_1948.png` |
| 대표 asset | `mydocs/pr/assets/pr_1963_issue1948_cross_para_fieldend_review_1948.png` |

visual sweep 결과:

- SVG/PDF 페이지 수: 1 / 1
- 자동 후보: `flagged=0/1`
- pixel match: `92.97862%`
- 내용 픽셀 중심 자동 일치율 보조값: `21.15087%`

사람 판정 메모:

- 기준 PDF와 rhwp 출력 모두 1쪽이며 큰 페이지/프레임/라인 후보는 없다.
- 자동 일치율 보조값은 폰트/raster/logo/회색 placeholder 차이를 크게 반영해 낮지만, PR의 핵심인
  교차 문단 `fieldEnd` 순서와 roundtrip offset 보존 판단에는 blocker가 아니다.

## 직접 CLI 검증

```bash
target/debug/rhwp hwpx-roundtrip \
  samples/hwpx/issue1948_cross_para_fieldend.hwpx \
  -o output/pr1963-roundtrip

target/debug/rhwp render-diff \
  samples/hwpx/issue1948_cross_para_fieldend.hwpx \
  --via hwpx \
  -o output/pr1963-render-diff
```

결과:

- `hwpx-roundtrip`: `[ PASS] diff=0 r2=0`
- `render-diff --via hwpx`: pages A/B `1/1`, max displacement `0.00 px`, status `PASS`

## 로컬 검증

검토 시작 시 cargo cache 비대화 영향을 줄이기 위해 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다. 이후
아래 명령을 순차 실행했다.

```bash
gh pr edit 1963 --repo edwardkim/rhwp --add-reviewer jangster77
git fetch upstream devel pull/1963/head:local/pr1963
git switch -C review/pr1963 local/pr1963
git merge upstream/devel --no-commit --no-ff
git diff --check upstream/devel...HEAD
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo build
env CARGO_INCREMENTAL=0 cargo test --lib serializer::hwpx::section
env CARGO_INCREMENTAL=0 cargo test --test hwpx_roundtrip_baseline
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

결과:

- reviewer assign: `jangster77`
- `git merge upstream/devel --no-commit --no-ff`: `Already up to date`, 충돌 없음
- `git diff --check upstream/devel...HEAD`: 통과
- `cargo fmt --check`: 통과
- `cargo build`: 통과
- `cargo test --lib serializer::hwpx::section`: 49 passed
- `cargo test --test hwpx_roundtrip_baseline`: 4 passed
- `cargo test --profile release-test --tests`: 통과
  - lib tests: 2126 passed, 6 ignored
  - `tests/svg_snapshot.rs`: 8 passed
  - command exit 0
- `cargo clippy --all-targets -- -D warnings`: 통과

## 검토 결과

### 1. 보정 위치는 PR의 원인과 직접 맞다

문제는 `fieldEnd`가 모델에는 고아 fieldEnd로 남아 있는데 slot 방출 루프가 먼저 말미 slot을 배치해
8-unit gap을 선점하는 순서 문제다. 변경은 slot 루프 진입부에서 같은 `char_idx`의 미방출 orphan fieldEnd를
먼저 방출하므로 원인 지점과 대응된다.

### 2. 문서 속성 기반 보정이다

분기는 PR 샘플명을 보지 않고 `para.orphan_field_ends`와 logical UTF-16 위치만 본다. 이는 serializer가
이미 관리하는 field/control 위치 정보에 근거한 보정이므로 하드코딩성 보정으로 보지 않는다.

### 3. 회귀 검증 범위가 적절하다

신규 축소 샘플, 직접 `hwpx-roundtrip`, `render-diff --via hwpx`, serializer 단위 테스트, HWPX baseline,
전체 release-test, clippy, GitHub CI가 모두 통과했다.

## 최종 권고

merge 완료로 정리한다.

- merge commit: `2e1c6929acc2b0f02bb6a2473597c45900fe70f4`
- merge 방식: admin merge
- merge 시각: `2026-07-05T17:03:43Z`

후속:

- #1948은 GitHub Actions auto-close로 CLOSED 상태다. 옵션 2 docs-only PR merge 후 #1963에서 처리됐다는
  수동 후속 코멘트를 남긴다.
- review 문서, 첨부 기준 PDF, 대표 visual asset은 옵션 2 docs-only PR로 archive/asset 경로에 반영한다.
