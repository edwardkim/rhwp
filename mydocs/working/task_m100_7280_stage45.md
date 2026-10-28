# Task #7280 Stage 45 — R3d 블록 컷 수용·밴드 재시도 Query 분리

- Issue: #7280. 이전: [Stage44](task_m100_7280_stage44.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `3711f2996`. 제품: `2cfa3c13b05f6b6ddd742da8a16ab358bfc8bd0d`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3d 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/block_fit.rs`의 `BlockCutQuery`로 세 읽기 전용 판단을 분리한다.

- 저장 줄 내용이 완결되는 기존 label-response 분기의 마지막 행 밴드 높이 조회.
- RowBreak rowspan/hard-break와 fresh 쪽 초과 여부에 따른 블록 분할 허용 판단.
- plain 컷이 기각된 뒤 행 오프셋 기반 컷을 재시도할 조건 조회.

입력은 기존 `RowBlockQuery`, 후보 블록, 현재/시작 행 번호, 시작 컷, 원래 컷 결과의
불변 참조다. TypesetEngine/TypesetState·가변 페이지 상태를 받지 않는다. 원래 컷의 실행,
fresh 쪽 높이 조회, 진단 로그, 재시도 컷 생성과 채택, 누적 높이·끝 컷·커서 변경은
부모가 기존 순서대로 수행한다. 원래 컷 결과와 재시도 결과도 합치거나 덮어쓰지 않는다.

저장 줄 완결 분기는 기존 guard를 통과한 때만 내용 높이를 조회하고, 모든 기존 높이 조건을
만족하면 `Some(last_row_band)`를 반환한다. 부모는 기존과 동일하게 남은 밴드 높이를
누적하고 마지막 행 높이만 기록한 뒤 continue한다. 실패하면 후속 분할/재시도/이월로 진행한다.
기존 조건·허용치·연산 순서를 보존하며 이 분기의 압축 정책이나 기존 주석의 설명을
새로운 조판 규칙으로 승인하는 변경은 아니다. 테스트·baseline·ignore는 변경하지 않는다.

## 실제 호출과 결과 소비

제품 `2cfa3c13b` 기준:

1. `typeset.rs:22401` 일반 스캔과 `:22534` 각주 예약 후 refit이 같은
   `scan_block_table_split_rows`를 호출한다. 원래 측정 표/행 기하 표, 컷용 높이,
   시작 컷과 가용 예산의 구분을 그대로 보존한다.
2. 블록 분류·요구 높이의 전체 fit 실패 뒤 부모가 plain/offset 컷을 실행하고 진단한다.
   `:17670`의 저장 줄 완결 조회는 이 컷 결과를 소비한다. source 없는 재조판,
   내부 컨트롤, hard-break 등 기존 비적용 경계는 그대로다.
3. 조회한 마지막 행 밴드는 `end_row_height_override`로 전달되고, 예약 높이는
   `consumed += cs_before + remaining_band`로 확정된다. 시작 컷의 유닛 수와
   물리 밴드 높이를 하나의 값으로 대신하지 않는다.
4. 미수용 시 `:17681` 분할 허용, `:17695` 밴드 재시도 조건을 원래 순서로 조회한다.
   재시도 오프셋 생성과 `advance_row_block_cut_with_row_offsets` 실행, 재시도 로그,
   `fully_consumed`/최소 높이 검사 및 `band_fill` 채택은 부모 그대로다.
5. 분할 수용 시 `:17747` 끝 컷 복사, `:17780` 실제 조각 높이 누적을 거쳐
   `:22808` PartialTable 발행과 `:22895` 커서 전진·종료·각주/캡션 정산으로 이어진다.
   분할 불가 시 시작 행 강제 수용 또는 다음 쪽 이월도 변경하지 않는다.

블록 진입 전의 빈 물리 밴드, 블록 외 일반 행/거대 행/rowspan 끝행, 별도 TAC 경로와
최종 paint는 이동하지 않았다. 해당 부모 전체와 두 호출부의 불변을 정적으로 확인한다.
실제 내부 분기별 동적 커버리지 또는 모든 렌더 경로가 동일 결과를 공유하게 됐다는 주장은 아니다.

## 검증과 증거의 범위

- 정적 대조: `output/7280/stage45/{verify-block-fit.mjs,extraction-proof.json}`.
  시작 head와 새 Query의 guard·높이 계산·분할/재시도 조건을 비교한다.
  부모 전체에 허용한 호출 치환 외의 변화가 없는지도 검사하여 컷 실행·진단·상태 효과·
  나머지 소비 경로 보존을 확인했다. rustfmt의 단일 표현식 closure 중괄호 생략은 정규화한다.
- 기존 집중 364건에 4건을 추가한다. 새 테스트나 source-side support를 만들지 않는다.
  고정 baseline 실행의 같은 테스트 이름·결과와 대조하며 새 결함 수정 전 FAIL을 주장하지 않는다.

| 추가 기존 계약 | 확인하는 의미와 한계 |
| --- | --- |
| `issue_5714_completed_row_tail_band` 2건 | 저장 내용 완결 샘플의 첫 행 전체 높이/마지막 행 축소, 압축된 앞 행의 tail이 다음 쪽 새 행에 잘못 전달되지 않는 셀·글줄 하한 |
| `issue_2097_band_fill` 1건 | 6개 샘플의 기존 페이지 수 유지. 컷 소유권·외곽·후행 문단·시각 일치 전체를 입증하지 않음 |
| `issue_2097_squeeze` 1건 | 3개 샘플의 기존 페이지 수 유지. 압축 정책의 타당성을 새로 승인하는 근거가 아님 |

기존 #6024/#6756/#6803/#7226의 이어받기·유닛·기하 계약과 한계는
[Stage44](task_m100_7280_stage44.md)의 검증 기록을 따른다. 이 선택이 모든 조건 조합을
실행했다는 주장은 하지 않는다. 미실행 경계는 책임 묶음 검증 시 계속 확인해야 한다.

review worktree `/home/edward/mygithub/rhwp-review-7280-r3d`에서 파생 suite를 준비하고,
공유 `target/pr-review`를 보존하여 jobs 4 / test threads 8로 순차 검증한다.
manifest·unit-tier는 위 고정 baseline을 명시한다. 이번 절편은 R3 전체 회귀·Native/fresh
Docker WASM 비교 및 최종 native/WASM/workspace lint·제출 게이트 완료가 아니다.
원격 push·PR·댓글은 승인 범위가 아니다.

## 고정 제품 검증 결과

review worktree에서 실행한 순서:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage45/run-focused.sh
```

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과. Clippy 1분 01초.
- 집중 nextest **368 passed / 0 failed**, 25 binaries, 필터 비선택 8,039건, exit 0.
  비선택은 전체 회귀의 기존 ignore 50건과 구분한다. 빌드 7분 00초, 테스트 7.607초.
  run ID: `be606f42-c728-44cd-8069-d0b0bb94278c`.
- `compare-focused.mjs` / `regression-comparison.json`에서 고정 baseline 실행의 같은
  368개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀를 이번 절편에서 다시 실행한 것은 아니다.
- 로그: `output/7280/stage45/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  추가 계약이 읽는 실물 10개 입력의 SHA-256은 `fixture.sha256`에 기록했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전 절편과 동일하다.

검증 후 제품 코드 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
baseline·ignore·기대값 변경이나 원격 push·PR·댓글은 없다. 구조 보존과 기존 집중 계약은
충족했지만, R3 전체 회귀·Native/fresh Docker WASM 시각 비교 및 최종 제출 게이트는
아직 미실행이다. 기존 계약 재검증이며 결함 수정 전 FAIL/후 PASS 증거로 보고하지 않는다.

## 다음 절편

블록 컷의 채택 결과와 누적 예약/끝 컷 반영을 분리하고, 이어서 일반 행·rowspan 끝행 및
continuation 본체의 책임을 분리한다. 기존 수치·예외를 바꾸지 않으며 R3 책임 묶음 완료 후
전체 회귀와 Native/fresh Docker WASM 시각 대조를 진행한다.
