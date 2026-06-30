# PR #1677 검토 - 렌더러 폰트 메트릭과 굵기 보존

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1677](https://github.com/edwardkim/rhwp/pull/1677) |
| 제목 | `[경험적 렌더링] 렌더러 폰트 메트릭과 굵기 보존` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-font-metrics` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 4 파일, +164/-15 |
| 원 PR 커밋 | `cb651bfaa851` |
| 로컬 적용 커밋 | `935e036ca` (`git cherry-pick -x`) |

## 2. 변경 범위

텍스트 측정과 HTML 출력에서 renderer font hint와 font weight를 더 구체적으로 보존하는 렌더링 보정 PR이다. PR 본문은 Ghidra/Frida 근거가 아니라 경험적 text-metric 맞춤이라고 분류한다.

- renderer font hint를 style resolution에서 text measurement 경로까지 전달한다.
- 모든 굵은 글씨를 generic `bold`로 접지 않고 구체적인 CSS font weight 값을 출력한다.
- `src/renderer/layout/text_measurement.rs`, `src/renderer/style_resolver.rs`, `src/renderer/mod.rs`, `src/renderer/html.rs`가 변경된다.
- paragraph condense와 pagination 자체는 이 PR 범위 밖으로 둔다.

## 3. 검토 의견

폰트 hint와 굵기 보존은 실제 HWPX 출력에서 자간, 줄 높이, line break 위치에 직접 영향을 준다. 값 손실을 줄이는 방향은 타당하지만, 변경 표면은 작아 보여도 모든 텍스트 레이아웃에 파급될 수 있다.

특히 `text_measurement` 경로의 변경은 페이지 분할, table cell 내 텍스트 흐름, shape 내부 텍스트와 같이 서로 다른 호출자가 공유한다. 단일 fixture 이미지의 시각 개선만으로는 충분하지 않으므로, 이번 검토에서는 기존 regression set의 페이지 수와 line break 안정성을 같이 확인했다.

PR 본문에 첨부된 release asset 이미지는 참고 자료다. 저장소에 fixture/golden truth가 들어온 것은 아니므로, 이번 검토에서는 deterministic snapshot/visual sweep을 수행했고 canonical 출력과의 최종 비교는 작업지시자 시각 판정 범위로 남긴다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 1개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. 개별 merge보다는 일괄 브랜치에서 #1676 이후 변경과 함께 검증하는 방식이 적합하다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `svg_snapshot`/`visual_roundtrip_baseline` 포함)
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

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1677 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| 중고딕/태고딕 계열 medium weight hint 식별 | `cargo test --profile release-test --lib weight` 중 `test_medium_weight_face` 통과 |
| Light/Bold face weight를 CSS font-weight로 구체화 | `cargo test --profile release-test --lib weight` 중 `test_explicit_face_weight_hints` 통과 |
| HTML/SVG 텍스트 출력에 medium weight가 반영됨 | `cargo test --profile release-test --lib weight` 중 `test_html_draw_text_medium_weight`, `test_svg_draw_text_medium_weight` 통과 |
| 언어별 font family가 style resolver에서 보존되고 fallback 동작이 유지됨 | `cargo test --profile release-test --lib test_resolve_char_style_font_families`, `cargo test --profile release-test --lib font_family_for_lang` 통과 |

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. 텍스트/수식 겹침 후보와 tail/question/large ink 후보가 일부 남아 있어 수동 시각 판정 대상이지만, font weight/hint contract 또는 page count mismatch 실패는 확인되지 않았다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보이나 렌더링 회귀 가능성이 있는 경험적 보정이다. font hint/weight 보존 자체는 targeted 검증으로 확인했고, 단독 렌더 snapshot 및 WASM/browser 측정 경로도 통과했다. 기존 문서/표/shape 텍스트 페이지 수가 흔들리면 이 PR 단독 원인인지 #1680/#1681/#1683과 함께 재분리해야 한다.
