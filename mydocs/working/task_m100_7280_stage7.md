# Task #7280 Stage 7 — R2e 문단 분할 경계 보정 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2d](task_m100_7280_stage6.md), 시작 head `178667990`.
- 상태: R2e 구현·집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.

## 1. 책임과 범위

`typeset_paragraph`의 줄 스캔이 만든 후보를 실제 분할 경계로 보정하는 책임을
`paragraph/split.rs`의 `refine_split_boundary`로 분리한다. 문서 IR·문단 구성 결과와
다음 문단, 현재 줄 범위·예산·호환 flag·dpi를 받으며 TypesetState/TypesetEngine은 받지 않는다.
상태를 변경하거나 다음 페이지를 생성하지 않는다.

`SplitBoundary`는 exclusive 끝 줄과 해당 후보의 누적 advance를 함께 반환한다.
두 값을 따로 재추정하지 않는다. 기존 조건·상수·연산 순서·PageItem 의미·기대값은 유지한다.
기존 조건의 타당성 또는 한컴 피델리티 개선을 이번에 판정하는 것은 아니다.

보정 순서는 기존대로 최소 한 줄 보장 → 뒤따르는 RowBreak 앵커 표 앞의 한 줄 되돌림 →
저장 줄 위치 되감김 복원이다. 최소 한 줄 보장 시 cumulative를 다시 계산하지 않던 동작도
그대로 둔다. 표 앵커/되감김 보정에서만 기존 `line_advances_sum`으로 누적량을 다시 계산한다.

본문 높이의 기존 메서드는 읽기 전용 `layout.available_body_height()`이고 profile 조회도
불변 상태를 읽는다. 보정 block 내부에는 상태 변경이 없으므로 호출 시 값으로 전달한다.
각주/여백 등을 차감한 `avail_for_lines`와 기본 본문 높이를 하나로 합치지 않는다.

## 2. 생산·소비 경계

- 생산: 기존 `for li` 줄 스캔의 `end_line`, `cumulative`. 저장 꼬리 채택 flag도 기존 위치에서 생산한다.
- 보정: 새 Query가 같은 end/advance 쌍을 반환한다. 시작 줄은 조정자가 소유한다.
- 소비: 반환 end로 `part_line_height`와 마지막 spacing_after를 구하고 part_height를 조립한다.
  전체 수용 경로는 반환 cumulative로 trailing spacing 제외/overflow를 재확인한다.
- 확정: 기존 FullParagraph/PartialParagraph 추가 → trimmed spacing 초기화 → 높이 전진.
  끝 줄이면 종료하고, 아니면 단/쪽 전환 후 cursor를 end로 바꾸는 순서는 변경하지 않는다.

줄 스캔 자체의 saved-tail/각주/강제 경계 정책, 표 셀/rowspan/continuation은 이번 절편 밖이다.
새 Query가 전체 문단 분할을 소유한다고 주장하지 않는다. `is_synthetic_line_seg`와
`LADDER_FIT_EPSILON_PX`의 parent 의존은 명시적 import로 남아 있으며 최종 소유권은 후속 정리한다.

## 3. 검증

`output/7280/stage7/verify-split.mjs`는 입력/반환 매핑을 확인한 뒤 Query 본문을 원래 위치로
복원 비교한다. 상수·조건·주석·계산 순서 및 그 외 parent 코드가 일치한다.
formatter 공백/후행 쉼표와 짧아진 변수명 때문에 생긴 단일 식 closure 중괄호 차이만 정규화한다.
기존 테스트·golden·baseline·ignore·CI 정책과 public API/IR은 변경하지 않는다.

집중 계약은 기존 typeset/composer와 `issue_6542`, `issue_6718_native_hwp5_zero_vpos_rewind`,
`issue_6718_zero_rewind_in_split_paragraph`를 선택한다. 실제 render tree의 본문 하단,
여유 예산이 남은 되감김, sub-pixel fit을 포함한다. 최소 진행/다음 표 앵커의 모든 조건 조합을
새로 실행 검증했다고 주장하지 않는다. 정적 보존 증거나 자동 검사를 직접 시각 판정으로 승격하지 않는다.

고정 SHA review worktree에서 suite 준비, fmt·manifest/unit-tier 정책·native Clippy와 집중 검사를
순차 실행한다. 전체 회귀, 최종 WASM/workspace lint와 native build, Native Skia,
fresh Docker WASM·직접 시각 검증은 최종 통합 게이트에 남는다. 원격 쓰기는 하지 않는다.

### 고정 SHA 실행 기록

- 제품 SHA: `ca7534500cdd1b8410ac1e6b4c8fdb96e0185f8c`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2e`.
- 정적 보존 검사: `output/7280/stage7/extraction-proof.json` 통과.
- manifest 고정 baseline 검사: 1,382 sources / 5,965 static attrs / 48 integration targets 통과.
- unit-tier 고정 baseline 검사: 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support items 28 통과.
- `cargo fmt --all -- --check` 통과.
- `CARGO_BUILD_JOBS=4 cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings`
  통과(exit 0, 55.36초).
- 로그: `output/7280/stage7/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

review worktree에서 준비한 suite 매핑에 따라 다음 집중 검사를 실행한다.

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --lib --test regression_suite_011 --test regression_suite_019 --test regression_suite_028 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(issue_6542_mid_para_vpos_rewind_breaks_page::) | test(issue_6718_native_hwp5_zero_vpos_rewind::) | test(issue_6718_zero_rewind_in_split_paragraph::)' \
  --no-fail-fast
```

집중 테스트 결과: **153건 통과 / 실패 0건**, 필터 비선택 4,540건, 7 binaries, exit 0.
빌드 4분 56초, 테스트 0.369초. 실행 ID: `cbdc3c48-3d31-4194-aa08-b99626e5b89d`.
로그: `output/7280/stage7/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 결과에서
동일 필터로 선택한 153개 PASS 이름과 일치함을 확인했다. 필터 비선택 건수는 기존 ignore
50건과 다른 수치이며 이번에 전체 회귀를 재실행한 것은 아니다.

nextest 0.9.137(권장 0.9.140), 미사용 observation profile 설정 경고는 기준 실행과 같다.
현재 CI 도구 버전까지 동일한 검증으로 주장하지 않는다. review worktree의 tracked 변경은 없고
파생 suite/manifest는 stage하지 않았다. 제품 검증 후에는 완료 기록만 갱신했다.

## 4. 다음 절편

줄 스캔의 저장 꼬리/각주 판단과 실제 분할·배치 조정, 표 문단 흐름의 책임 분리를 이어간다.
이번 경계 Query는 상태를 변경하지 않지만, 전체 상태 캡슐화와 R2 전체 분해는 아직 남아 있다.
최종 전체 회귀·WASM·시각 증적 없이 #7280 완료 또는 제출 준비 완료로 보고하지 않는다.
