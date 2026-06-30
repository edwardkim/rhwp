# PR #1682 검토 - master-page 장식 요소 plane별 리플레이

## 1. PR 정보

| 항목 | 값 |
|------|-----|
| PR | [#1682](https://github.com/edwardkim/rhwp/pull/1682) |
| 제목 | `[부분 Ghidra 근거] master-page 장식 요소를 plane별 리플레이` |
| 작성자 | `humdrum00001010` |
| base <- head | `devel` <- `codex/pr1573-master-furniture` |
| 문서 작성 시점 참고값 | `MERGEABLE` / `BEHIND`, draft 아님, maintainer 수정 가능 |
| 규모 | 4 파일, +865/-49 |
| 원 PR 커밋 | `6ab4e464bffa` |
| 로컬 적용 커밋 | `4652a7b38` (`git cherry-pick -x`) |

## 2. 변경 범위

master-page/header 장식 요소를 render plane별로 replay하고, 섹션을 넘어 odd/even master-page 선택을 이어받도록 하는 렌더 pipeline 변경 PR이다. PR 본문은 render-plane, `zOrder`, paper-origin 처리를 부분 Ghidra/Frida 근거가 있는 동작으로 분류한다.

- 다음 섹션이 특정 parity master-page ref를 생략할 때 이전 odd/even ref를 이어받는다.
- paper-background master control을 감지해 body text 뒤쪽 plane에 배치한다.
- master-page render node를 plane, `zOrder`, stable source index 기준으로 정렬한다.
- paper-relative header/master control을 page origin 기준으로 배치한다.
- `src/document_core/queries/rendering.rs`, `src/renderer/layout.rs`, `src/renderer/layout/tests.rs`, `src/renderer/page_layout.rs`가 변경된다.

## 3. 검토 의견

이 PR은 변경량이 크고 page furniture ordering에 직접 관여한다. master/header/footer 장식, 본문 뒤 배경, body overlay 사이의 순서를 바꾸므로 visual impact가 넓다.

부분 Ghidra/Frida 근거를 명시한 지점은 장점이다. 다만 page-selection carryover와 render-plane replay가 한 PR 안에 묶여 있어, 실패 시 원인 분리가 어려울 수 있다. local batch에서는 #1681 matrix/group 변경 뒤에 적용되어 visual diff가 겹칠 가능성이 있다.

`src/renderer/layout/tests.rs`에 테스트가 크게 추가된 것은 긍정적이다. 이 변경은 snapshot/visual 계층 확인이 필요하므로, 이번 검토에서는 단독 `svg_snapshot`과 visual sweep을 함께 실행했다. 특히 odd/even master-page가 있는 문서와 body text 뒤 장식 요소는 필요 시 수동 비교에서 집중 확인한다.

## 4. 로컬 적용 상태

`upstream/devel` 기준 로컬 일괄 검토 브랜치 `local/humdrum-pr-batch-review`에 원 커밋 1개를 `-x`로 체리픽했다. 충돌은 없었다.

원 PR은 `BEHIND` 상태다. 일괄 브랜치에서는 #1681 뒤에 적용했다.

## 5. 검증 상태

- 완료: `cargo build --release`
- 완료: `cargo test --release --lib` (2037 passed, 0 failed, 7 ignored)
- 완료: `cargo test --profile release-test --tests` (통합 테스트 전체 통과, `svg_snapshot` 포함)
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

2026-06-30 로컬 일괄 브랜치 `local/humdrum-pr-batch-review`에서 #1682 주장별로 다음을 확인했다.

| 주장 | 검증 |
|------|------|
| odd/even master-page ref carryover와 fallback selection | `cargo test --profile release-test --lib master_page` 중 `master_page_selection_uses_final_carried_page_number_parity`, `missing_even_master_page_inherits_previous_even_master`, `single_base_master_page_applies_when_matching_parity_is_absent` 통과 |
| master-page control을 plane, `zOrder`, stable index 순서로 정렬 | 같은 명령의 `master_page_controls_sort_by_render_layer_z_order` 통과 |
| paper-sized master background는 body text 뒤 plane으로 replay | 같은 명령의 `master_page_paper_sized_background_replays_behind_body_text` 통과 |
| 작은 front master control은 body text 앞 plane 유지 | 같은 명령의 `master_page_smaller_front_control_stays_in_front_of_body_text` 통과 |
| master-page paper-relative shape/picture는 page origin 기준 배치 | 같은 명령의 `master_page_paper_relative_shape_uses_page_origin`, `master_page_paper_relative_picture_uses_page_origin` 통과 |
| header paper-relative shape/picture도 page origin 기준 배치 | `cargo test --profile release-test --lib header_paper_relative` 통과 |

단, 이 검증은 synthetic/unit render tree contract다. 이번 snapshot/visual sweep에서는 page count mismatch가 없었지만, PR 본문 release asset의 실제 문서 시각 정합은 저장소 fixture 밖의 수동 비교 범위로 남는다.

### 5.2 시각/브라우저 검증

2026-06-30 로컬 일괄 브랜치에서 자동 시각 gate를 추가 확인했다.

- `cargo test --test svg_snapshot`: 8개 golden snapshot 통과
- `cargo test --profile release-test --test visual_roundtrip_baseline`: 3개 visual roundtrip baseline 통과
- `python3 scripts/task1274_visual_sweep.py --target all`: 15개 target 모두 SVG/PDF page count 일치, page count mismatch 0건
- 자동 sweep flagged 후보: 5개 target. master-page/header furniture ordering 변경과 직접 맞물린 page count mismatch는 없었지만, PR 본문 release asset 자체는 저장소 fixture가 아니므로 필요 시 수동 비교로 확인한다.
- browser/WASM 경로: `rhwp-studio` TypeScript/test와 `wasm-pack build --target web --out-dir pkg` 통과

## 6. 잠정 판단

조건부 수용 후보. 부분 역공학 근거가 있는 변경의 주요 contract는 targeted 검증으로 확인했고, snapshot/visual sweep도 page count mismatch 없이 통과했다. 다만 렌더 ordering 변경 폭이 커서 PR 본문 release asset까지 확인하려면 수동 비교가 필요하며, 실패가 나오면 #1682 단독 revert/분리 가능성을 열어 둬야 한다.
