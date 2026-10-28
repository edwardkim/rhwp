# Task #7280 Stage 18 — R2p 저장 TAC 표 문단 경로 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2o](task_m100_7280_stage17.md), 시작 head `eae87f9be`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2p 구현·고정 SHA 집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.

## 1. 책임과 보존 범위

표 문단의 기존 `진입 진단 → float 배제 영역 소비 → 호스트 문단 구성` 뒤에 위치한
저장 TAC 줄 수용 경로만 분리한다. 일반 TAC 카운트/pre-flush, float 및 지연 배치,
표 행/셀 컷과 continuation은 이번 절편에서 변경하지 않는다.

- `controls/stored_tac.rs`: profile/side-wrap gate, 기존 composer의 줄 소속 조회,
  흐름·저장 원점의 max 선택, 선언 높이와 실측 높이 대조, 가용 높이 fit 및 표별 배치 계산.
- `controls::try_place_stored_tac_paragraph`: 계획 수용 시 표별 계산·확정을 원래 순서로 수행.
  불수용은 기존 일반 컨트롤 경로로, 수용은 기존 문단 조기 반환으로 연결한다.
- `state`: 좁은 페이지 관측값 제공 및 확정 metadata → PageItem → current_height 반영.

`composer::stored_tac_lines`가 서로 다른 물리 줄의 표만 수용하는 기존 계약을 재사용한다.
TAC라는 이유로 모든 표를 같은 줄로 보거나 높이를 합산하지 않는다. 편집 세션, dirty/합성 저장 줄,
같은 줄의 복수 표, 다른 컨트롤과의 혼재를 다루는 기존 수용/배제 조건은 바꾸지 않는다.
기존 `measured_fits` 검사 뒤에 `fits`를 **별도로** 실행하며, 앞 검사가 false라는 이유로
뒤 검사를 생략하지 않는다. `available_height`의 환경 변수 기반 진단과 호출 횟수를 유지하려고
기존 `all` 내부에서 closure로 조회한다. 계산이 끝나기 전에는 상태를 쓰지 않는다.

### 좌표·높이 소비 경로

`composer::StoredTacLine(top/end/occupied_end)` → `stored_tac::prepare`의
`max(flow_origin, saved_origin)` 및 `max(occupied_end, end)` fit →
`StoredTacPlan::placement`의 top/advance_end → `state::commit_stored_tac_control`의
`inline_placements`와 `current_height` → 기존 LayoutEngine의 inline metadata 소비.
composer/측정/LayoutEngine/paint 소비자는 변경하지 않는다. 마지막 소유 표에만 문단 아래 간격을
더하는 순서와 음수 저장 간격을 그대로 유지한다. 표별 계산과 확정은 계속 교대로 수행한다.

제품 SHA 기준 소비 지점은 `layout.rs:10188`의 metadata 조회와 `:10199`의
`col_area.y + placement.y + outer_margin_top` 변환, `:10487`의 가로 정렬 분기,
`:11296`의 `col_area.y + advance_end` 반환이다. `advance_end=Some`인 이 경로는
기존 가로 정렬을 보존하고 뒤의 표 하단 기반 줄간격 재계산으로 들어가지 않는다.
이들 소비 지점의 계약을 새로 고친 것이 아니라 생산 결과의 의미와 전달 경로를 보존했다.

이번 작업은 기존 동작의 구조 분리이며 조판 오류 수정이나 기존 heuristic의 타당성 승인이 아니다.
IR/API·테스트 source/assertion/ID·baseline/golden/ignore·CI 정책은 변경하지 않는다.
controls의 나머지 경로와 최종 상태 캡슐화는 후속 이행 항목이다.

## 2. 검증 계획

`output/7280/stage18/verify-stored-tac.mjs`로 기존 parent 분기와 query/계획/상태 명령을
복원 비교한다. 수용 조건, 두 all의 평가 순서, 원점/끝점 산식, 항목/상태 쓰기 순서와
나머지 parent·기존 state 명령·테스트의 불변을 확인한다.

별도 review worktree에서 파생 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → 집중 nextest를 순차 실행한다. typeset/composer/float_placement와 기존
#7103, #7150, #6601, #6812, maintainer_nested_table_lines 계약을 선택한다.
원본 저장 문서의 좌표 계약과 수동 변경한 입력의 합성 경계 계약은 구분한다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM/직접 시각 대조는
통합 게이트에 남긴다. 이번 절편은 원격 push·PR·댓글이나 시각 통과 선언을 포함하지 않는다.

## 3. 고정 head 검증

- 제품 SHA: `1f07dc95745554bed6e41421e1d929bf19d072b4`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2p`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 복원 비교: `verify-stored-tac.mjs` / `extraction-proof.json` 통과.
- 파생 suite 준비 후 manifest 고정 baseline 비교 통과:
  1,382 sources / 5,965 static attrs / 48 targets.
- unit-tier 고정 baseline 비교 통과:
  4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 / cfg support 28.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir
  /home/edward/mygithub/rhwp/target/pr-review -- -D warnings` 통과(exit 0, 55.60초).
- 로그: `output/7280/stage18/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

정책 검사는 각각 `node scripts/rust-test-suite-manifest.mjs --check --base-ref
722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`와 `node scripts/rust-unit-test-tiers.mjs
--check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`로 실행했다.

집중 테스트(review worktree):

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review --lib \
  --test regression_suite_004 --test regression_suite_006 --test regression_suite_007 \
  --test regression_suite_009 --test regression_suite_027 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_7103_tac_table_rewind::) | test(issue_7150_tac_line_owner_anchor::) | test(issue_6601_inline_tac_tables_share_a_line::) | test(issue_6812_square_picture_tac_table::) | test(maintainer_nested_table_lines::)' \
  --no-fail-fast
```

검사 의미와 한계:

- #7103 4건은 이번 저장 줄 경로의 핵심 계약이다. 원본 문서의 표 경계/PDF 기준 좌표와,
  저장 입력을 수동 변경한 양수/음수 간격 보존 계약을 구분한다. 후자는 한컴 출력 증거가 아니다.
- #6601, #7150은 같은 줄의 복수 표 및 줄 소유자/이웃 표의 기하를 보호하는 대조 계약이다.
  #7150의 cross-line fixture는 합성 계약으로 분류한다.
- #6812는 어울림 공간과 TAC 혼재, 앞 문단 배제 영역의 영향을 보호한다.
- maintainer_nested_table_lines는 저장 슬롯·명시적 개행·너비 부족·재조판 대조군을 보호한다.
  중첩 표 전체 알고리즘을 이번에 변경하거나 검증 완료한 것으로 해석하지 않는다.
- 선언/실측 불일치, 각주/배제 영역 예산 경계, 모든 profile 조합의 경로별 실행 추적은
  이번 절편에서 새로 작성하지 않았다. 정적 순서 보존과 기존 계약 실행의 증거를 구분한다.

결과: **197건 통과 / 실패 0건**, 9 binaries, 필터 비선택 4,918건, exit 0.
빌드 5분 06초, 테스트 0.420초. 실행 ID: `78012853-0522-4f7f-8315-0534513a5b4a`.
로그: `output/7280/stage18/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 실행 중
동일 필터의 197개 PASS 이름과 일치함을 확인했다. 필터 비선택은 기존 ignore 50건과
별개이며, 이번 절편에서 전체 회귀를 다시 실행하지 않았다.

nextest 0.9.137(권장 0.9.140)과 observation profile 설정 경고는 기준 실행과 동일하다.
전체 CI 통과나 한컴 직접 시각 판독 완료를 주장하지 않는다. review worktree의 tracked 변경은
없으며 파생 suite/manifest는 커밋하지 않았다. 제품 검증 후에는 계획과 결과 기록만 수정했다.

## 4. 후속

일반 TAC 문단의 카운트·첫 줄·편집 후 높이·pre-flush 판단과 float/지연 배치 등 나머지
컨트롤 흐름 분리를 계속한다. 표 컷/continuation, 각주 의존과 최종 상태 캡슐화,
전체 통합 검증은 별도 절편으로 남아 있다.
