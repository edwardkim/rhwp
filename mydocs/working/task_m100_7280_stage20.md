# Task #7280 Stage 20 — R2r 컨트롤 배치 순서 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2q](task_m100_7280_stage19.md), 시작 head `56139c729`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2r 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 PR 준비 완료가 아니다.

## 1. 분리한 책임과 보존 계약

`controls/order.rs`가 문단 컨트롤의 배치 인덱스와 그 순서상 첫/마지막 표를 계산한다.
입력은 기존 Paragraph, FormattedParagraph, 읽기 전용 TacFlowQuery이며 페이지 상태나
가변 IR은 받지 않는다. 원본 컨트롤 배열은 수정하지 않고 인덱스 벡터만 반환한다.

음수 float 오프셋, 저장 줄의 문단 내부 리셋, TAC 호스트 줄 높이와 양수 float의 앵커 상대
겹침, 가시 텍스트 조건, 두 정렬 키와 안정 정렬을 원래 계산·평가 순서대로 보존한다.
`should_sort_para_float_tables=false`는 세로 오프셋 키를 0으로 만드는 조건일 뿐이다.
그때도 **비-TAC/TAC 보조 키는 계속 적용**한다. 기존 주석의 “배열 순서 보존”이라는
표현을 이유로 정렬 전체를 생략하지 않는다. 동일 키의 순서는 기존 stable sort가 보존한다.

### 결과의 실제 소비

`order::for_paragraph` → 기존 컨트롤 순회가 `ctrl_order`와 `order_pos`를 소비 →
`first_placed_table/last_placed_table`이 TAC/블록 표 배치의 앞뒤 텍스트·간격 플래그로 전달된다.
지연 이월하는 형제 표 목록도 동일 인덱스의 `skip(order_pos + 1)`과 첫/마지막 플래그를 사용한다.
반환값은 부모에서 원래 지역변수 이름으로 풀어 기존 두 소비 경로를 그대로 유지한다.

이번 절편은 좌표·높이·컷 산식이나 배치/측정 결과를 바꾸지 않는다. 상태 예약, 실제 표 배치,
지연 큐, 각주 및 metadata/paint 소비자도 변경하지 않는다. 기존
`layout::stored_float_anchor_offset_hu` 읽기 의존은 그대로 남기며 완전한 단방향화로 보고하지 않는다.
공통 TAC 판별은 R2q의 query를 재사용한다. 별도 캐시·새 규칙·profile 선조회는 추가하지 않는다.

IR/API, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책은 변경하지 않는다.
기존 조판 heuristic의 정당성을 새로 승인하거나 조판 결함을 해결하는 작업이 아니다.

## 2. 검증

`output/7280/stage20/verify-control-order.mjs`는 새 조회 본문을 이전 위치의 표현으로 복원해
조건·평가 순서·정렬 키·first/last 선택을 대조한다. 부모의 나머지 코드와 테스트,
기존 query/state의 불변도 검사한다. 이 정적 검사는 런타임 회귀 검사를 대신하지 않는다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier, fmt,
native Clippy, 집중 nextest를 순차 수행한다. R2q의 211건에 #6879와 #6929를 추가한다.
#6879는 TAC/float 앵커 소속·실제 순서·그림 위치를, #6929는 float 위치와 가시 제목 대조군을
보호한다. 한컴 기준을 인용한 기존 실물 계약과 합성 대조군을 구분하며 직접 시각 판독으로
격상하지 않는다. 모든 부호·동률·리셋 조합의 별도 실행 추적은 이번에 새로 만들지 않는다.

전체 회귀, WASM/workspace lint, workspace build, Native Skia, fresh Docker WASM 및 직접
시각 대조는 통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위에 포함하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `ae983e461f37ba9eaefff354e6e29af7ab9bc495`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2r`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 제품 SHA를 입력한 정적 복원 비교 통과: `extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 56.13초).
- 로그: `output/7280/stage20/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

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
  --test regression_suite_009 --test regression_suite_015 --test regression_suite_018 \
  --test regression_suite_019 --test regression_suite_027 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_7103_tac_table_rewind::) | test(issue_7150_tac_line_owner_anchor::) | test(issue_6601_inline_tac_tables_share_a_line::) | test(issue_6812_square_picture_tac_table::) | test(maintainer_nested_table_lines::) | test(issue_5700_tac_reset_tail_above_flow::) | test(issue_5807_coanchored_float_tac_order::) | test(issue_7049_inline_tac_table_baseline::) | test(issue_7062_tac_object_host_line_height::) | test(tac_group_page_bottom_overflow::) | test(issue_6879_tac_sibling_float_anchor_line::) | test(issue_6929_float_table_para_offset::)' \
  --no-fail-fast
```

결과: **218건 통과 / 실패 0건**, 12 binaries, 필터 비선택 5,514건, exit 0.
빌드 5분 31초, 테스트 0.470초. 실행 ID: `b09a6fd8-b286-4124-9d56-bb527672fa6c`.
로그: `output/7280/stage20/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서
동일 필터로 선택한 218개 PASS 이름과 일치함을 확인했다. 이번 절편에서 전체 회귀를
재실행하지 않았으며, 필터 비선택은 기존 ignore 50건과 별개다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
제품 검증 뒤에는 계획과 결과 기록만 수정했다. 전체 CI·WASM 및 직접 시각 판독 완료로
해석하지 않는다.

## 4. 후속

표 컨트롤의 실제 배치·float/지연 이월 조정과 표 컷/continuation 책임 분리를 이어간다.
R2 전체 및 최종 통합 검증은 아직 남아 있다.
