# Task #7280 Stage 46 — R3e 채택한 블록 조각의 범위·점유 높이 Query 분리

- Issue: #7280. 이전: [Stage45](task_m100_7280_stage45.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `5b3135ad7`. 제품: `cfbff473fbf0b4efe1f96b4ff575a86239be54a7`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3e 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/block_fragment.rs`의 `SelectedBlockCut`은 기존 컷과 밴드 재시도 컷 중
원래 우선순위로 선택한 불변 참조와 행 오프셋 사용 여부를 보유한다. 선택 자체는
내용 컷을 복사하거나 조각 높이를 미리 측정하지 않는다.

- `select`: 재시도 결과가 있으면 그 결과와 오프셋을 선택하고, 없으면 원래 컷을 선택한다.
- `end_row`: 기존 offset 컷의 끝 행 계산/허용치를 보존한다. 비offset 컷은 기존 블록 끝 행을 쓴다.
- `occupied_height`: offset 컷은 기존 행별 조각 합과 컷 소비 높이의 비교를, 비offset 컷은
  `row_block_content_height` 계산을 그대로 수행한다.

부모는 `끝 행 확정 → end_cut clone → end_limit → block_start → 점유 높이 조회 → 누적 → break`
순서를 그대로 소유한다. 불필요해진 단일 호출 closure만 제거하고, 그 호출은 동일한
`RowBlockQuery::fragment_height`로 연결했다. 선택 시점으로 높이 측정을 앞당기지 않는다.
Query에는 가변 입력·TypesetState/TypesetEngine·clone·source-side 테스트가 없다.

이는 읽기 전용 선택/기하 계산과 부모의 명시적인 쓰기를 분리하는 절편이다.
스캔 상태 전체를 캡슐화하거나 기존 모든 측정/paint 경로를 공통 알고리즘으로 교체한 것은 아니다.
기존 offset 높이 보정의 정책 타당성을 새로 승인하지 않으며, 상수·조건·기대값은 변경하지 않는다.

## 실제 호출·컷·예약·배치 연결

제품 `cfbff473f` 기준:

1. 일반 스캔 `typeset.rs:22359`와 각주 예약 후 refit `:22492`가 기존 스캔을 호출한다.
   원본 측정 표/행 기하 표, 시작 컷, 컷용 높이, 가용 예산 구분을 유지한다.
2. 요구 높이의 전체 fit 실패 뒤 컷 실행·분할 허용·재시도 판단을 기존 순서로 수행한다.
   실제 컷을 수용하는 기존 guard 안에서만 `:17726`의 `select`를 호출한다.
3. `:17732`가 선택된 컷의 끝 행을 소비한다. 뒤따르는 끝 컷 복사/limit/block_start 기록을
   보존하고, `:17737`에서 같은 시작 컷·끝 컷으로 점유 높이를 계산한다.
   `consumed += cs_before + split_total`도 그대로여서 내용 유닛 컷과 물리 점유를 합치지 않는다.
4. `:22766` PartialTable 발행과 `:22853` continuation 전진으로 전달된다.
   헤더/각주 예약, 렌더 높이 보정, 마지막 유닛 소비 후 종료 및 캡션/후행 내용 정산은 그대로다.

전체 fit 수용, source-complete 마지막 행 밴드, 분할 불가 강제 수용/이월, 시작 행의 빈 물리 밴드,
일반 행/rowspan 끝행·별도 TAC 경로는 이번 Query의 적용 범위 밖이며 원본을 유지한다.
두 호출부와 나머지 부모 전체를 정적으로 대조했다. 해당 분기들이 모두 실행됐다는 동적
커버리지나 한컴 시각 일치 증거로 해석하지 않는다.

## 검증 계약과 한계

- 정적 대조: `output/7280/stage46/{verify-block-fragment.mjs,extraction-proof.json}`.
  컷 선택/끝 행/점유 높이 계산식을 원본과 비교하고, 부모의 guard·컷 clone·진단·소비 순서와
  나머지 호출/continuation 본문에 허용한 치환 외 변화가 없음을 확인했다.
- 기존 집중 368건을 그대로 실행한다. 컷 유닛의 누락/중복·이어받기·본문/셀 하한과 인접 예산을
  검사하는 #6024/#6756/#6803/#7226 및 밴드/끝행의 #5714/#2097을 포함한다.
  정확한 의미와 기존 허용치·합성 입력/페이지 수 검사의 한계는
  [Stage44](task_m100_7280_stage44.md), [Stage45](task_m100_7280_stage45.md)를 따른다.
- 원래 높이만 fit하는 예산, 여러 rowspan 끝점 등 모든 내부 조합의 동적 실행을 주장하지 않는다.
  미검증 경계는 R3 책임 묶음 검증 시 확인한다. 이번 변경은 결함 수정이 아니므로
  기존 통과 계약을 수정 전 FAIL/후 PASS의 결함 검출 증거로 보고하지 않는다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3e`.
공유 `target/pr-review`를 보존하고 jobs 4 / test threads 8로 순차 실행한다.
새 source-side test/support, 테스트 기대값·baseline·ignore 변경은 없다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage46/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 1분 00초.
- 집중 nextest **368 passed / 0 failed**, 25 binaries, 필터 비선택 8,039건, exit 0.
  비선택은 전체 회귀의 기존 ignore 50건과 구분한다. 빌드 7분 02초, 테스트 7.645초.
  run ID: `a8e6c19a-4be8-4368-b95e-c6f486da7ae7`.
- `output/7280/stage46/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  같은 368개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀는 재실행하지 않았다.
- 실행 로그: `output/7280/stage46/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  Stage45 밴드 관련 입력 10개 해시의 불변을 `fixture-check.log`로 확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 코드 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크 및
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
이번 절편의 구조 보존과 기존 집중 계약은 충족했지만 R3 전체 회귀·Native/fresh Docker
WASM 시각 비교와 최종 제출 게이트는 아직 미실행이다. 원격 작업은 하지 않았다.

## 다음 절편

일반 행과 rowspan 끝행의 fit/잔여 밴드 조회를 분리하고, 스캔 조정·continuation 본체와
상태 소유 경계를 이어서 정리한다. R3 전체 회귀·Native/fresh Docker WASM 시각 비교와
최종 제출 게이트는 아직 남아 있다. 원격 push·PR·댓글은 이번 승인 범위가 아니다.
