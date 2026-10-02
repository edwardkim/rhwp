# Task #7280 Stage 21 — R2s 형제 표 지연 후보 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2r](task_m100_7280_stage20.md), 시작 head `3ef227511`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2s 구현·고정 제품 SHA 집중 검증 완료. R2 전체 완료나 PR 준비 완료가 아니다.

## 1. 분리한 책임과 보존 계약

`controls/deferred.rs`의 읽기 전용 `CoanchoredTableQuery`로 같은 문단에 매달린
RowBreak 표의 지연 배치 판별과 후행 후보 선택을 이동한다. Paragraph·FormattedParagraph와
공유 TacFlowQuery만 보유한다. 판별에 필요한 현재 쪽 수와 항목 슬라이스를 받지만
TypesetEngine/TypesetState 전체나 가변 IR은 받지 않는다.

이월 트리거는 기존 표 조건 → 물리 페이지 증가 → 마지막 항목의 continuation 순서로 판별한다.
후행 후보에만 적용하는 **양수 세로 오프셋** 조건을 트리거와 합치거나 완화하지 않는다.
가시 텍스트와 줄 점유를 새로 등치시키지 않으며 기존 조건의 정당성을 승인하는 작업도 아니다.
profile은 기존 TAC 판별의 단락 평가 지점에서 읽는다. 새 조회 객체 생성 시에는 참조만 보유한다.
쪽 수와 항목 슬라이스는 부수효과 없는 관측값이며 판별 전에 상태를 바꾸지 않는다.

### 실제 소비 경로

기존 block 표 배치 → 이월 여부 조회 → 기존 정렬 순서의 `skip(order_pos + 1)`에서 후보 선택 →
기존 큐 extend → 각주 등록 → 기존 loop break 순서를 유지한다. 결과에는 원래 문단/컨트롤 인덱스,
첫/마지막 표 플래그와 **원 배치 시점의 para_start_height**가 그대로 담긴다.

flush에서는 현재 열 너비로 문단을 다시 format한 뒤 같은 후보 판별을 재사용한다.
큐 drain/유지 시점, Before/AfterTableParagraph·SectionEnd 선택, 실제 블록 배치,
각주와 vpos 상태 반영은 부모에 그대로 남는다. 지연 배치의 렌더 원점은 현재 흐름 높이를,
분할 예산 원점은 저장한 최초 문단 높이를 사용하는 기존 #1860 계약도 변경하지 않는다.
컷·요구 높이·예약 높이·paint 산식은 이번 절편의 변경 대상이 아니다.

IR/API, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책은 변경하지 않는다.
지연 큐와 결과 타입의 상태 캡슐화 및 실제 표 배치 분리는 후속 책임으로 남긴다.

## 2. 검증 계획

`output/7280/stage21/verify-deferred.mjs`로 세 판별 본문과 후보 선택을 원래 표현으로 복원해
대조하고, 부모의 큐/flush/각주/반복 종료와 기존 테스트·state·공유 TACquery 불변을 확인한다.
정적 비교는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier,
fmt, native Clippy, 집중 nextest를 순차 실행한다. R2r의 218건에 기존 #1686 4건,
#6795 5건, #5906 1건을 추가한다. #1686은 HWP/HWPX 이월 표·후속 제목 순서와 쪽 경계,
#6795는 0-offset 형제 표 비이월·겹침·순서와 통짜 배치 대조군,
#5906은 지연된 형제 표의 실제 행 경계·하단을 보호한다.
기존 한컴 근거를 인용한 계약이며 이번에 직접 시각 판독을 완료했다는 의미는 아니다.
각주가 있는 이월 후보의 모든 조합이나 큐 flush 지점별 실행 추적은 새로 추가하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM과 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위에 포함하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `860c6f67c1c9982c0d0c40bf7ce4d88bb3457efe`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2s`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 제품 SHA 기준 정적 복원 비교 통과: `output/7280/stage21/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 55.58초).
- 로그: `output/7280/stage21/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

정책 검사는 `node scripts/rust-test-suite-manifest.mjs --check --base-ref
722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 `node scripts/rust-unit-test-tiers.mjs
--check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`로 실행했다.
Clippy 명령은 `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir
/home/edward/mygithub/rhwp/target/pr-review -- -D warnings`다.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_004 --test regression_suite_006 --test regression_suite_007 \
  --test regression_suite_008 --test regression_suite_009 --test regression_suite_012 \
  --test regression_suite_015 --test regression_suite_018 --test regression_suite_019 \
  --test regression_suite_027 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_7103_tac_table_rewind::) | test(issue_7150_tac_line_owner_anchor::) | test(issue_6601_inline_tac_tables_share_a_line::) | test(issue_6812_square_picture_tac_table::) | test(maintainer_nested_table_lines::) | test(issue_5700_tac_reset_tail_above_flow::) | test(issue_5807_coanchored_float_tac_order::) | test(issue_7049_inline_tac_table_baseline::) | test(issue_7062_tac_object_host_line_height::) | test(tac_group_page_bottom_overflow::) | test(issue_6879_tac_sibling_float_anchor_line::) | test(issue_6929_float_table_para_offset::) | test(issue_1686::) | test(issue_6795_split_float_sibling_gets_its_own_page::) | test(issue_5906_float_stack_declared_tail::)' \
  --no-fail-fast
```

결과: **228건 통과 / 실패 0건**, 14 binaries, 필터 비선택 5,950건, exit 0.
빌드 5분 43초, 테스트 0.562초. 실행 ID: `740c611d-6983-4ce7-8230-a6276ce70225`.
로그: `output/7280/stage21/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서
동일 필터로 선택한 228개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를
재실행하지 않았으며 필터 비선택은 기존 ignore 50건과 별개다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
제품 검증 뒤에는 계획과 결과 기록만 수정했다. 전체 CI·WASM 및 직접 시각 판독 완료로
해석하지 않는다.

## 4. 후속

컨트롤 실제 배치·float 조정 및 지연 큐 적용/flush의 책임 분리를 이어간다.
R2 전체, 표 컷/continuation 분리와 최종 통합 검증은 아직 남아 있다.
