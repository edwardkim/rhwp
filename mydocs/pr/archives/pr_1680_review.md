# PR #1680 검토 - HWPX PUA 글리프와 줄바꿈 매핑

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1680](https://github.com/edwardkim/rhwp/pull/1680) |
| 제목 | `[경험적 렌더링] HWPX PUA 글리프와 줄바꿈 매핑` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-pua-linebreaks` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 5 파일, +199/-10 |
| 원 PR 커밋 | `207a2b6e1fe6` |
| 로컬 적용 커밋 | `0da18bbc9` (`git cherry-pick -x`) |

## 2. 변경 범위

HWPX private-use glyph를 표시 가능한 글리프로 매핑하고, 치환 문자 주변의 line break 처리를 보정하는 경험적 렌더링 PR이다.

- fixture에서 관찰된 알려진 PUA codepoint를 표시 글리프로 변환한다.
- filler 성격의 PUA codepoint가 visible text로 나오지 않게 한다.
- replacement와 paragraph condense spacing 주변 line-break composition을 보존한다.
- `src/renderer/composer.rs`, `src/renderer/composer/line_breaking.rs`, `src/renderer/style_resolver.rs`, `tests/issue_937.rs` 등이 변경된다.

## 3. 검토 의견

PUA 매핑은 실제 HWPX 출력에서 빈 네모나 알 수 없는 문자 대신 의도된 기호를 보여주는 효과가 있다. 다만 private-use 영역은 문서/폰트/템플릿별 의미가 다를 수 있으므로, 관찰된 codepoint에 좁게 적용되어야 한다.

line break 보정은 텍스트 조판 전체에 영향을 줄 수 있다. #1677의 font metric 보존과 결합하면 줄바꿈 위치가 더 많이 변할 수 있으므로, 이번 검토에서는 둘을 함께 적용한 최신 브랜치에서 페이지 수와 visual snapshot을 확인했다.

`tests/issue_937.rs`와 composer tests가 추가된 것은 좋다. 그래도 PR 본문에 첨부된 release asset은 참고 증거일 뿐이며, canonical visual gate를 대체하지 않는다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 1개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. 일괄 브랜치에서는 #1677의 font metric 변경 뒤에 적용되어 두 텍스트 보정이 함께 존재한다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `issue_937` 포함)
- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 공통 검증 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 5.1 PR 내용별 targeted 검증

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1680 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| `U+F012B` signature PUA를 표시 텍스트 `(인)`으로 확장 | `cargo test --profile release-test --test issue_937` 중 `issue_937_f012b_display_text_should_be_signature_seal`, `issue_937_svg_renders_f012b_as_signature_seal` 통과 |
| `U+F081C` filler PUA가 visible text로 나오지 않음 | 같은 명령의 `issue_937_f081c_filler_should_not_render_as_text` 통과 |
| `U+F02FC`, `U+F031C`가 각각 callout/toc 표시 글리프로 매핑 | 같은 명령의 `issue_937_f02fc_callout_bullet_should_render_as_pointer`, `issue_937_f031c_toc_bullet_should_render_as_square` 통과 |
| 실제 fixture에서 signature cell source PUA가 존재함 | 같은 명령의 `issue_937_bokhakwonseo_signature_cell_contains_f012b` 통과 |
| paragraph condense spacing 주변 reflow가 space width를 줄여 반영 | `cargo test --profile release-test --lib test_reflow_condense_shrinks_measured_space_width` 통과 |
| 한국어 break word tokenization 보정 | `cargo test --profile release-test --lib test_tokenize_korean_break_word_chars` 통과 |

단, 이 검증은 관찰된 PUA codepoint와 관련 line-break 단위 동작을 확인한다. 다른 문서/폰트의 private-use 의미까지 일반화하지 않는다.

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `cargo test --profile release-test --tests page_count`: 필터 매칭 26개 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. PUA 매핑/line-break 변경과 결합 가능성은 있으나, 현재 자동 sweep에서는 전체 페이지 수 불일치가 없다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보이나 경험적 매핑 범위를 좁게 유지해야 한다. PUA 표시와 line-break 보정의 직접 주장은 targeted 검증으로 확인했고, snapshot/page-count/visual sweep도 페이지 수 불일치 없이 통과했다. 향후 line break나 페이지 수가 흔들리면 #1677과 #1680의 결합 영향을 먼저 분리해 확인한다.
