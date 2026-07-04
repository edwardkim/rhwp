# PR #1681 검토 - matrix 위치 그룹 텍스트박스 재조판

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1681](https://github.com/edwardkim/rhwp/pull/1681) |
| 제목 | `[경험적 렌더링] matrix 위치 그룹 텍스트박스 재조판` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-matrix-textbox` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 3 파일, +371/-66 |
| 원 PR 커밋 | `7f9a10263908`, `dff4695898dc` |
| 로컬 적용 커밋 | `c279f7d93`, `3500d8e64` (`git cherry-pick -x`) |

## 2. 변경 범위

group transform이 matrix-positioned child textbox에 두 번 적용되는 문제를 줄이고, matrix-scaled dimension 때문에 overflow가 나는 textbox line을 재구성하는 경험적 렌더링 PR이다.

- HWPX rendering matrix placement를 group child에 대해 이미 배치된 값으로 취급한다.
- matrix-scaled dimension으로 overflow가 생기는 textbox를 다시 조판한다.
- 새 shape layout signature에 맞춰 table cell call-site에 default 인자를 추가한다.
- `tests/visual_roundtrip_baseline.rs`에서 기존 xfail 성격 sample을 고정 visual roundtrip 대상으로 승격한다.

## 3. 검토 의견

group/matrix 처리의 double-apply 문제는 실제 페이지 좌표가 크게 틀어지는 원인이 될 수 있어 수용 가치가 있다. 다만 `shape_layout.rs` 변경량이 크고, group child 좌표 체계는 이미지, connector, text box, table 등 여러 shape에 파급될 수 있다.

table cell 쪽 변경은 1줄 default 전달이지만 layout signature 변경에 따른 호출자 의미가 맞는지 봐야 한다. matrix textbox 전용 보정이 일반 table cell layout에 섞이지 않는지 확인이 필요하다.

visual roundtrip baseline 승격은 좋은 방향이다. 이번 검토에서는 저장소 자동 시각 gate를 실행했지만, 저장소에 release asset 이미지는 들어오지 않았으므로 PR의 이미지 증거는 참고 자료다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 2개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. 일괄 브랜치에서는 #1679 connector IR 복원 뒤에 적용되어 diagram 계열 변경이 함께 존재한다.

추가 maintainer 보강: clippy `items_after_test_module` 실패를 해소하기 위해 `src/renderer/layout/shape_layout.rs`의 새 test module을 파일 끝으로 이동했다. 테스트 내용과 런타임 로직은 바꾸지 않았다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `visual_roundtrip_baseline` 포함)
- 완료: `cargo fmt --check`
- 완료: `git diff --check`
- 완료: `cargo clippy --all-targets -- -D warnings` (최초 1회 `items_after_test_module` 실패 후 test module 위치 보정, 재실행 18m 25s warning 0, TIFF 보강 후 최종 재실행 17m 28s warning 0)
- 완료: `cargo test --doc` (0 passed, 0 failed, 1 ignored)
- 완료: `cargo test --test svg_snapshot` (8 passed)
- 완료: `cd rhwp-studio && npx tsc --noEmit`
- 완료: `cd rhwp-studio && npm test` (153 passed)
- 완료: `wasm-pack build --target web --out-dir pkg` (1m 25s)
- 중단/무효: `cargo check --all-targets --message-format=short`는 workflow 문서에 없는 명령이라 중단했으며 검증 결과로 기록하지 않는다.

### 5.1 PR 내용별 targeted 검증

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1681 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| matrix-positioned textbox에서 imported line이 height overflow를 만들면 재조판 | `cargo test --profile release-test --lib matrix_textbox_para_collapses_imported_lines_that_overflow_height` 통과 |
| PR에서 승격한 visual roundtrip sample이 PASS 목록과 xfail 목록 정합성을 유지 | `cargo test --profile release-test --test visual_roundtrip_baseline` 통과 (`visual_grade_lists_are_consistent`, `visual_xfail_entries_still_fail`, `visual_baseline_all_samples`) |

단, 이 검증은 repository visual roundtrip 기준이다. PR 본문 release asset과 canonical editor/PDF 시각 일치 여부는 저장소 fixture 밖의 수동 시각 판정 범위로 남는다.

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. matrix/group textbox PR 본문 release asset은 저장소 fixture가 아니므로, 해당 asset 자체의 canonical 일치는 수동 시각 판정 대상으로 남긴다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

수용 후보이나 렌더 좌표계 변경 폭이 커서 full 검증 대상이다. matrix textbox reflow와 visual baseline 승격은 targeted 검증으로 확인했고, snapshot/visual sweep도 page count mismatch 없이 통과했다. #1679, #1682와 함께 diagram/master plane 계열 visual diff는 필요 시 비교 이미지 기반 수동 판정으로 이어간다.
