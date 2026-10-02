# Task #7280 Stage 25 — R2w TAC 문단 배치 후 높이 정산 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2v](task_m100_7280_stage24.md), 시작 head `c42e55643`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2w 구현·고정 제품 SHA 집중 검증 완료. R2 전체 완료나 PR 준비 완료가 아니다.

## 1. 책임 경계와 보존 계약

`typeset_table_paragraph` 끝의 TAC 높이 정산만 분리한다.
`controls/tac_reconcile.rs`는 높이·상한·저장 줄간격 누락 여부와 앵커·최종 하단을 조회한다.
`controls::reconcile_tac_height`는 조회 → 누락 표시/진단/저장 좌표 무효화 → 앵커 및 상한
조회 → 진단 → 최종 하단 조회 → 높이 확정 순서를 조정한다.
`state.rs`는 필요한 관측값과 두 상태 명령을 제공한다. 공개 API·IR·출력 타입은 변경하지 않는다.

- TAC 존재, 양수 문단 높이, 저장 LineSeg 존재, 배치 전후 페이지 수 동일이라는 바깥 guard는
  부모에 남긴다. 앞선 decoration-only host 텍스트 배치도 그대로 둔다.
- 저장 줄 높이와 실측 표 높이 선택, 줄간격 절반 가산, TAC 그림/도형의 선행 TAC 개수와 저장 줄
  대응을 유지한다. 기존 줄 소속 가정의 정확성을 새로 승인하는 변경이 아니다.
- owned rowbreak frame, 문단 앞 간격, 표 위 바깥여백, 저장 스텝·문단 간격 누락 서명과
  1.0/0.5/2.0 임계값을 그대로 둔다. 앵커의 전방 24px 조건도 바꾸지 않는다.
- `TacHeightCap`의 `tac_seg_total`, `cap`, `stored_step_px`, `ladder_total`,
  `ladder_omits_spacing`을 명시적으로 반환한다. 원래 조건과 합산 순서를 유지한다.
- effective TAC 판별에는 engine의 기존 `TacFlowQuery`를 사용하고, HWPX 저장 레이아웃
  판별에는 기존 state profile을 사용한다. 서로 같은 값이라고 가정하여 합치지 않는다.
- `RHWP_DIAG_TACSIB`는 해당 높이 가산 직후 callback으로 실행한다.
  줄간격 누락 명령은 `stored_ladder_spacing_omitted=true` → 진단 callback →
  `vpos_ladder_dirty=true` 순서를 보존한다. callback은 정적 디스패치이며 새 등록 체계가 아니다.
  조회에는 문서/조판 상태 쓰기 권한이 없지만 진단 callback의 외부 효과까지 없는 순수 함수는 아니다.
- 앵커 계산 후 줄간격 누락 상한을 선택하고, 편집 세션 성장량의 `max`를 적용한다.
  TACCAP 진단 뒤 해당 문단의 clearance를 합산하고, 전체 inline placement 유무에 따른
  flow bottom 보정을 유지한다. 현재 높이가 최종 하단보다 클 때만 낮추는 기존 조건을 보존한다.
- 이번 분리는 기존 cap/되감김의 타당성을 재판정하거나 조판 오류를 수정하는 작업이 아니다.
  표 컷·continuation과 나머지 표 문단 조정, 최종 상태 캡슐화는 후속에 남긴다.

조판 산식·상수·선택 순서, 테스트 source/assertion/ID, baseline/golden/ignore, CI 정책은 변경하지 않는다.

## 2. 검증 계획과 한계

`output/7280/stage25/verify-tac-reconcile.mjs`로 새 조회/상태 명령/진단을 기존 순서로 재구성하여
원본과 비교한다. 공백·주석·후행 쉼표와 rustfmt의 단일 표현식 closure 중괄호만 정규화한다.
coordinator 인자·호출 순서, 관측값 필드, 부모 guard/다른 경로/테스트와 나머지 명령의 불변도 검사한다.
정적 비교는 실행 검증이나 모든 진단 환경변수 조합의 실행 증거를 대신하지 않는다.

별도 review worktree에서 파생 suite 준비, 고정 baseline 대비 manifest/unit-tier, fmt,
native Clippy와 집중 nextest를 순차 실행한다. R2v 250건에 아래 기존 5건을 추가한다.

- `issue_2319_no_lineseg_tac_table_height` 2건: LineSeg가 없는 TAC 표의 높이를 잘못 되감지 않는 경로.
- `issue_1835_tac_stale_height` 2건: 선언 높이를 낮춘 합성 HWP의 실측 표 높이와 뒤 문단 비겹침.
  fixture에는 별도 한컴 PDF가 연결되어 있으나, 이번 실행은 기존 assertion 검사이며
  실제 편집 세션 성장 경로나 PDF 직접 대조의 실행 증거와는 구분한다.
- `issue_2220_tac_host_line_outer_margin` 1건: TAC host 바깥여백 이중 가산과 오른쪽 단 말미 위치.

기존 #6812 어울림/owned frame, #3738 TAC 동반 개체, #7049/#7062 host 줄 계약도 유지한다.
줄간격 누락 서명의 모든 경계값과 편집 세션 성장 분기의 독립 실행 추적은 이번 절편에 추가하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM과 직접 시각 대조는
통합 게이트에 남긴다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.

## 3. 고정 head 검증

- 제품 SHA: `df3f1744542bb5bcd31bfa314adc0872237b45ba`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2w`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 비교 통과: `output/7280/stage25/extraction-proof.json`.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- native Clippy `-D warnings` 통과(exit 0, 57.50초).
- 로그: `output/7280/stage25/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

정책 검사는 `node scripts/rust-test-suite-manifest.mjs --check --base-ref
722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 `node scripts/rust-unit-test-tiers.mjs
--check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`로 실행했다.
Clippy 명령은 `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir
/home/edward/mygithub/rhwp/target/pr-review -- -D warnings`다.

집중 테스트는 `bash output/7280/stage25/run-focused.sh`로 실행했다. 전체 명령은 다음과 같다.

```bash
cd /home/edward/mygithub/rhwp-review-7280-r2w
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_005 --test regression_suite_017 --test regression_suite_002 --test regression_suite_020 --test regression_suite_001 --test regression_suite_004 --test regression_suite_006 \
  --test regression_suite_007 --test regression_suite_008 --test regression_suite_009 \
  --test regression_suite_012 --test regression_suite_015 --test regression_suite_018 \
  --test regression_suite_019 --test regression_suite_027 \
  -E 'test(issue_2319_no_lineseg_tac_table_height::) | test(issue_1835_tac_stale_height::) | test(issue_2220_tac_host_line_outer_margin::) | test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_7103_tac_table_rewind::) | test(issue_7150_tac_line_owner_anchor::) | test(issue_6601_inline_tac_tables_share_a_line::) | test(issue_6812_square_picture_tac_table::) | test(maintainer_nested_table_lines::) | test(issue_5700_tac_reset_tail_above_flow::) | test(issue_5807_coanchored_float_tac_order::) | test(issue_7049_inline_tac_table_baseline::) | test(issue_7062_tac_object_host_line_height::) | test(tac_group_page_bottom_overflow::) | test(issue_6879_tac_sibling_float_anchor_line::) | test(issue_6929_float_table_para_offset::) | test(issue_1686::) | test(issue_6795_split_float_sibling_gets_its_own_page::) | test(issue_5906_float_stack_declared_tail::) | test(issue_1753_deferred_table_fill_ahead::) | test(=issue_3738_rowbreak_table_footnote_fragment::rowbreak_table_cell_footnotes_keep_the_pdf_fragment_boundary) | test(issue_6946_block_seated_float_sibling_gets_its_own_page::) | test(issue_7203_float_table_stored_anchor_top::) | test(issue_7203_float_table_top_uses_stored_anchor::) | test(issue_5585_sibling_table_anchor_offset::) | test(issue_3738_tac_sibling_shape_line_advance::) | test(issue_1156_chart_column_flow::) | test(issue_6146_page_tail_float_band_spill::)' \
  --no-fail-fast
```

결과: **255건 통과 / 실패 0건**, 19 binaries, 필터 비선택 6,929건, exit 0.
빌드 6분 12초, 테스트 1.523초. 실행 ID: `84625c36-bec5-4ecf-b2b2-c45bd46da584`.
로그: `output/7280/stage25/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행에서 동일 필터로
선택한 255개 PASS 이름과 일치함을 확인했다. 이번에 전체 회귀를 재실행하지 않았으며
필터 비선택은 기존 ignore 50건과 별개다. 출력의 픽셀 동일성이나 전체 시각 일치 판정은 아니다.

nextest 0.9.137(권장 0.9.140) 및 observation profile 설정 경고는 기준 실행과 동일하다.
review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 뒤에는 계획과 결과 기록만 수정했으며 제품 코드 변경은 없다.

## 4. 후속

표 문단의 decoration-only 텍스트 배치 등 잔여 흐름 조정 책임을 분리한다.
표 컷/continuation, 각주, 최종 상태 캡슐화와 전체 통합 게이트는 남아 있다.
