# Task #7280 Stage 16 — R2n 강제 쪽 경계 조회·선택 책임 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2m](task_m100_7280_stage15.md), 시작 head `82fdbbcf6`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2n 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

`paragraph/boundary.rs`는 컨트롤의 inline metadata 판정과 저장 줄 기반 문단 내부 쪽 경계,
HWPX 명시적 다음 쪽 앞의 tail 경계 조회를 소유한다. 기존 세 helper의 본문·상수·비교 조건은
보존한다. 별도 좌표/높이 계산이나 저장 LineSeg 수용 범위 변경은 하지 않는다.

`paragraph::prepare_forced_page_boundary`는 기존 각주 경계를 **먼저** 계산하고, 현재 쪽
vpos 기준을 조회한 뒤 후보를 다음의 기존 순서로 선택한다.

1. 내부 저장 경계와 HWPX 현재 흐름 일치 필터
2. HWPX 명시적 쪽나누기 앞 tail
3. 첫 각주와 겹치는 본문 경계
4. 저장 줄 누락의 trailing line 경계
5. 큰 TAC TopAndBottom 그림 앞 본문 경계
6. 앞서 계산해 둔 기존 각주 reset 경계

`or_else`/`then`/`filter`의 지연 평가와 마지막 `or`를 유지한다. 모든 후보를 미리 계산한
목록으로 바꾸지 않는다. 각주 조회는 composer와 진단을 호출할 수 있으므로 평가 시점을 보존한다.
조정자는 `&TypesetState`만 읽고 상태를 쓰지 않는다. 기존 각주/그림 helper의 전체 상태 읽기
의존은 이행 항목이며 R4 및 R5의 좁은 관측·상태 경계 분리에서 다룬다.

결과 세 값은 서로 합치지 않는다. 선택된 `forced_page_break_line`은 전체 fit/넘침/분할로,
`native_hwp5_existing_footnote_reset_line`은 별도로 후속 줄 스캔으로,
`current_page_vpos_base`는 fit의 저장 bounds와 분할 경계로 그대로 전달한다.
`scan_lines`의 줄 컷 선택 → 기존 경계 보정 → 기존 조각 높이/각주 예약 → 페이지 전환 및
실제 배치 경로는 수정하지 않는다. 문단 전후의 fit 예산, 빈 줄/다단 조기 반환, trim 계산도 유지한다.

이번 절편은 기존 구현의 구조 이동이지 heuristic의 타당성 승인이나 조판 결함 수정이 아니다.
IR/API·테스트 assertion/ID·baseline/golden/ignore·CI 정책은 변경하지 않는다.
기존 private 테스트는 parent의 좁은 import로 연결해 모듈 ID와 검사를 보존한다.

## 2. 검증 계획

원래 helper 본문과 조정 block으로 복원 비교하여 조건·우선순위·지연 호출·반환값을 확인한다.
별도 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 실행한다. 기존 typeset/composer·저장 경계·각주 계약을
선택하며, 후보 충돌의 모든 조합이나 최종 출력의 한컴 일치를 입증했다고 주장하지 않는다.

전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM/직접 시각 대조는 통합 게이트에
남긴다. 이번 절편에서는 원격 push·PR·댓글을 수행하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `b6ff27f5d3ecacf81a5a25c4b193626d0720da62`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2n`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 복원 비교: `output/7280/stage16/verify-boundary.mjs` / `extraction-proof.json` 통과.
  세 helper의 본문, 기존 각주 경계의 선행 계산과 후보의 지연 평가 순서, 반환된 세 값,
  나머지 parent/소비자/기존 테스트와 paragraph 조정자의 불변을 확인했다.
- manifest: 고정 baseline 대비 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 고정 baseline 대비 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support 28 통과.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review
  -- -D warnings` 통과(exit 0, 55.37초).
- 로그: `output/7280/stage16/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_003 --test regression_suite_005 --test regression_suite_007 \
  --test regression_suite_011 --test regression_suite_012 --test regression_suite_013 \
  --test regression_suite_015 --test regression_suite_019 \
  --test regression_suite_022 --test regression_suite_026 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6568_para_start_first_line_fit::) | test(issue_5755_rewind_overflow_page_break::) | test(issue_5921_neartop_reset_fits::) | test(issue_6753_lazy_base_keeps_trimmed_spacing_before::) | test(issue_6793_tac_host_page_tail_padding::) | test(issue_5870_empty_host_float_flow_advance::) | test(issue_6854_empty_para_orphan_page::) | test(issue_6970_multicolumn_synth_ladder_fill::) | test(issue_6855_float_band_page_fill_rewind::) | test(issue_6031_ladder_sb_omitted_tail_overrun::) | test(issue_6718_native_hwp5_zero_vpos_rewind::) | test(issue_5128_spec_page_roundtrip::) | test(issue_6034_footnote_area_reflow_width::) | test(issue_5700_tac_reset_tail_above_flow::)' \
  --no-fail-fast
```

선택한 기존 계약:

- 기존 typeset의 `hwpx_explicit_page_break_tail_splits_only_last_stored_line` 및
  `hwpx_explicit_page_break_tail_requires_all_stored_layout_evidence`: tail 후보의 적용·비적용 근거.
- #6718 3건: 0-vpos 저장 되감김의 페이지 본문 넘침과 경계값 관련 기존 계약.
- #5128 5건: 원본/HWPX 왕복 프로필·문단 수·쪽수 및 p15/p16의 항목 종류/문단 소속.
- #5700 1건: TAC 표 host의 reset 꼬리가 앞 문단보다 아래에 남는 기존 흐름 계약.
- #6034 1건: 각주 영역이 본문 하단 아래에 있고 각주 줄 수가 기존 범위인 대조 계약.

앞 절편의 169개에 위 추가 10개를 더해 선택했다. 테스트 source는 수정하지 않았다.
#6034 통과만으로 기존/첫 각주 reset 후보의 모든 적용 경로를 입증할 수는 없다.
여러 후보가 동시에 생기는 모든 조합 및 진단 호출 횟수의 실행 추적은 이번 절편에서 새로
검사하지 않았으며, 정적 순서 보존과 선택된 기존 계약의 무회귀 증거를 구분한다.
결과: **179건 통과 / 실패 0건**, 14 binaries, 필터 비선택 5,989건, exit 0.
빌드 5분 47초, 테스트 0.780초. 실행 ID: `7415b24e-64e9-42e5-adfb-320343d552c5`.
로그: `output/7280/stage16/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행 중 같은 필터의
179개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과 별개이며,
이번 절편에서 전체 회귀를 재실행하지 않았다.

nextest 0.9.137(권장 0.9.140)과 observation profile 미사용 설정 경고는 기준 실행과 동일하다.
CI 도구 버전 일치나 전체 CI 통과를 주장하지 않는다. review worktree의 tracked 변경은 없고
파생 suite·manifest를 커밋하지 않았다. 제품 검증 후에는 계획과 결과 기록만 변경했다.

## 4. 후속

다단 분기와 trim/흐름 메트릭 등 남은 문단 조정, 표 문단/컨트롤 흐름 분리를 계속한다.
각주 측정 helper와 전체 상태 읽기 의존, 최종 상태 캡슐화 및 전체 통합 검증은 남아 있다.
이번 절편은 원격 push·PR 없이 로컬 커밋으로 닫는다.
