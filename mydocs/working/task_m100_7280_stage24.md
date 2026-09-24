# Task #7280 Stage 24 — R2v 표 문단 동반 개체 흐름 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2u](task_m100_7280_stage23.md), 시작 head `f88bdc150`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2v 구현·고정 제품 SHA 집중 검증 완료. R2 전체 완료나 PR 준비 완료가 아니다.

## 1. 책임 경계와 보존 계약

일반 `typeset_table_paragraph`의 Shape/Picture/Equation match arm을 대상으로 한다.
`controls/shape_flow.rs`는 기존 TAC 줄 높이·비TAC 자리차지 높이 및 이월 조건을 조회한다.
`controls::place_table_host_shape`는 이미 배치된 개체 제외 → 높이 조회 → 필요 시 단/쪽 이동 →
항목·높이 확정 → 어울림 등록 순서만 조정한다. `state.rs`는 조회용 관측값과 항목·높이 확정을 맡는다.
공개 API·IR·출력 타입은 늘리지 않으며 새 타입의 가시성은 typeset 내부로 제한한다.

### 실제 호출과 소비

- match arm의 `continue`는 coordinator의 `return`으로 대응한다. arm 뒤 loop 말미에 추가 명령이
  없으므로 동일 개체의 재배치만 건너뛰며 전체 문단 처리를 중단하지 않는다.
- 원본 컨트롤 인덱스로 앞 TAC 개체 수를 세고 해당 저장 LineSeg를 선택하는 기존 계산을 유지한다.
  이 가정의 정확성을 새로 승인하거나 재조판 알고리즘으로 대체하는 절편이 아니다.
- 두 `Option<f64>`를 하나의 높이로 합치지 않는다. TAC 높이가 있으면 먼저 사용하고, 없을 때만
  비TAC TopAndBottom/Para 조건으로 선언 높이+아래여백을 계산한다. `None`과 `Some(0)`도 보존한다.
- TAC는 현재 항목 존재, 비TAC는 `current_height < 1.0` 단 상단 여부를 선행 조건으로 사용한다.
  이를 같은 조건으로 통일하지 않는다. `available_height`는 closure로 기존 단락 평가 시점에 조회한다.
- 이월 후 새 페이지에 `PageItem::Shape`를 추가하고, 계획의 우선순위대로 현재 높이를 증가시킨다.
  다음 `register_side_wrap_picture`에는 기존 `Some(para_start_height)`를 전달한다.
  페이지 이동 뒤 문단 원점을 재계산하지 않는다.
- Equation은 기존처럼 두 높이 모두 없는 경로를 유지하지만 항목 추가·어울림 등록 호출은 생략하지 않는다.
- 실제 높이 소비 뒤에 수행되는 TAC 문단 높이 보정과 중복된 줄 높이 계산은 부모에 그대로 남긴다.
  이번 조회 결과를 그 보정에 새로 연결하거나 cap을 변경하지 않는다.
- 페이지 전환과 `register_side_wrap_picture` 구현은 기존 TypesetState에 남아 있다.
  이 절편은 모든 상태 필드 캡슐화나 모든 그림/도형 배치 경로의 통합 완료가 아니다.

조판 산식·상수·선택 순서, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책을 바꾸지 않는다.
표 컷·continuation, 줄 소속 규칙 개선과 시각 결함 수정도 범위 밖이다.

## 2. 검증 계획과 한계

`output/7280/stage24/verify-shape-flow.mjs`로 높이 식, 서로 다른 이월 guard와 지연 예산,
항목→높이→어울림 순서, coordinator 인자와 기존 부모/상태/테스트의 불변을 원본과 대조한다.
정적 비교는 실행 검증의 대체물이 아니다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier,
fmt, native Clippy, 집중 nextest를 순차 실행한다. R2u 245건에 기존 계약 5건을 추가한다.

- `issue_3738_tac_sibling_shape_line_advance` 2건: 합성 저장 줄 입력의 후속 본문 18줄 보존과 SVG 쪽 밖 출력 방지.
  정상 한컴 생성본/출력 일치 증거와는 구분한다.
- `issue_1156_chart_column_flow` 2건: 실제 2단 문서의 차트 단 이동·뒤 텍스트 비겹침과 배경 투명도 대조군.
- `issue_6146_page_tail_float_band_spill` 1건: 떠나는 쪽 밴드 잔류와 다음 쪽 상단 제목 영역의 비겹침.

기존 #6812 어울림 그림/TAC 표 계약도 유지한다. 수식 전용 입력이나 모든 guard 경계의 독립 실행 추적을
새로 추가하지 않는다. 실제 backend 전체 일치나 직접 시각 판독 완료를 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM 및 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `91248918b528f110007916125d79055170d3eb81`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2v`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 비교 통과: `output/7280/stage24/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 58.10초).
- 로그: `output/7280/stage24/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

정책 검사는 `node scripts/rust-test-suite-manifest.mjs --check --base-ref
722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 `node scripts/rust-unit-test-tiers.mjs
--check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`로 실행했다.
Clippy 명령은 `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir
/home/edward/mygithub/rhwp/target/pr-review -- -D warnings`다.

집중 테스트는 `bash output/7280/stage24/run-focused.sh`로 실행했다.
위 review worktree에서 실행하는 전체 명령은 다음과 같다.

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_005 --test regression_suite_017 --test regression_suite_002 --test regression_suite_020 --test regression_suite_001 --test regression_suite_004 --test regression_suite_006 \
  --test regression_suite_007 --test regression_suite_008 --test regression_suite_009 \
  --test regression_suite_012 --test regression_suite_015 --test regression_suite_018 \
  --test regression_suite_019 --test regression_suite_027 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_7103_tac_table_rewind::) | test(issue_7150_tac_line_owner_anchor::) | test(issue_6601_inline_tac_tables_share_a_line::) | test(issue_6812_square_picture_tac_table::) | test(maintainer_nested_table_lines::) | test(issue_5700_tac_reset_tail_above_flow::) | test(issue_5807_coanchored_float_tac_order::) | test(issue_7049_inline_tac_table_baseline::) | test(issue_7062_tac_object_host_line_height::) | test(tac_group_page_bottom_overflow::) | test(issue_6879_tac_sibling_float_anchor_line::) | test(issue_6929_float_table_para_offset::) | test(issue_1686::) | test(issue_6795_split_float_sibling_gets_its_own_page::) | test(issue_5906_float_stack_declared_tail::) | test(issue_1753_deferred_table_fill_ahead::) | test(=issue_3738_rowbreak_table_footnote_fragment::rowbreak_table_cell_footnotes_keep_the_pdf_fragment_boundary) | test(issue_6946_block_seated_float_sibling_gets_its_own_page::) | test(issue_7203_float_table_stored_anchor_top::) | test(issue_7203_float_table_top_uses_stored_anchor::) | test(issue_5585_sibling_table_anchor_offset::) | test(issue_3738_tac_sibling_shape_line_advance::) | test(issue_1156_chart_column_flow::) | test(issue_6146_page_tail_float_band_spill::)' \
  --no-fail-fast
```

결과: **250건 통과 / 실패 0건**, 19 binaries, 필터 비선택 6,934건, exit 0.
빌드 6분 15초, 테스트 1.461초. 실행 ID: `b5a949ac-9cee-43f0-b7ec-bd92b86d6901`.
로그: `output/7280/stage24/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 동일 필터로
선택한 250개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를 재실행하지 않았으며
필터 비선택은 기존 ignore 50건과 별개다. 출력의 픽셀 동일성이나 전체 시각 일치 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

일반 표 문단의 배치 후 높이 보정·흐름 정산 책임 분리를 이어간다.
표 컷/continuation과 각주 책임 분리, 최종 상태 캡슐화 및 전체 통합 게이트는 남아 있다.
