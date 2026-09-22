# Task #7280 Stage 47 — R3f 행 요구 높이·rowspan 잔여 밴드 Query 분리

- Issue: #7280. 이전: [Stage46](task_m100_7280_stage46.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `390b05e63`. 제품: `c8436d6b5eed31313f55668c67bbc480389ed181`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3f 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/row.rs`에 행 요구 높이와 rowspan 잔여 밴드의 조회를 분리했다.
`RowScanQuery`는 기존 읽기 전용 표 조회, 현재/시작 행, 시작 컷, 시작 행의 물리 높이
override만 받는다. TypesetState/TypesetEngine이나 가변 페이지 상태는 전달하지 않는다.

- `required_height`: rowspan 행과 일반 행에 반복되던 `straddle_continuation_demand` 호출과
  실제 수용 누적량 차감을 공유한다. 두 원본 식은 변수명만 달랐음을 정적으로 확인했다.
- `whole_row_height`: 일반 행의 온전한 paint footprint와 시작 컷 이후 내용 높이를 구분한다.
  `cut_row_h`, `whole_row_fit_h`, 원본 `mt.row_heights`를 하나로 대체하지 않는다.
- `band_shape`: 이전 행에서 시작한 rowspan과 현재 행 내부 중첩 표 유무를 원래 순서로 조회한다.
- `probe_band`: 기존 guard 안에서 패딩 → 남은 내용 예산 → 컷 후보 → 실제 표시 높이 순서로 조회한다.
- `retains_blank_tail`: 진단 뒤 기존 최소 blank/완결 여부 조건을 평가한다.

실제 수용/이월, 누적 높이, 끝 행, 컷 move, 마지막 행 높이 상한, 진단 순서는 부모에 남긴다.
저장 프레임·가로 용지 예외와 일반 행의 후속 컷 선택은 이동하지 않았다. 현재 정책의
타당성을 새로 승인하거나 예외를 제거하는 변경이 아니라 책임 분리이며 수치·조건은 보존한다.

## 실제 호출·높이·컷·배치 연결

제품 `c8436d6b5` 기준:

1. 일반 스캔 `typeset.rs:22306`과 각주 예약 뒤 refit `:22439`가 같은 스캔을 호출한다.
   원본 측정 표와 행 기하 표, 컷용/온전한 행 높이와 가용 예산의 구분은 그대로다.
2. 블록 경로가 처리하지 않은 행에서 Query를 준비한다. rowspan 행의 `:17770`과
   일반 행의 `:17854`가 요구 높이를 소비한다. 후자는 `:17850`에서 시작 컷별 기본 높이를
   먼저 조회한다. `need - consumed - cs_before`는 실제 수용한 앞 행 높이를 그대로 차감한다.
3. rowspan 통째 수용 실패 후 `:17787` 형상 조회, 기존 guard 안의 `:17800` 컷/표시 높이 조회,
   원래 진단 로그, `:17813` blank tail 수용 판단 순서를 보존한다.
4. 수용 시 기존 `consumed += cs_before + rest`, 행 전진, 높이 override, `probe.end_cut` move와
   `split_end_limit = rest`가 실행된다. 내용 컷과 남은 물리 밴드를 합치거나 재계산하지 않는다.
   실패 시 기존 정지 진단 후 이월하며, 일반 행은 기존 저장 프레임/예산/행 내부 컷 분기를 따른다.
5. `:22713` PartialTable 발행과 `:22800` continuation 전진으로 전달된다.
   이어받기 빈 밴드, 헤더/각주 예약, 최종 유닛 소비 후 종료·캡션/후행 내용 처리는 변경하지 않았다.

선행 보호 블록/블록 컷, 시작 행의 물리 override 조기 continue 및 별도 TAC 경로는 이번 조회에
들어오지 않는다. 부모 전체와 일반/refit 호출의 정적 동등성을 확인하지만 모든 분기의
동적 커버리지 또는 모든 paint 경로의 신규 공통화를 주장하지 않는다.

## 검증 계약과 한계

정적 대조는 `output/7280/stage47/{extract-row.mjs,extraction-proof.json}`에 남긴다.
원본 계산식을 추출해 Query와 비교하고, 두 요구 높이 식의 동등성 및 부모의 허용한 호출
치환 외 변화가 없음을 검사한다. 최초 패치의 hunk 순서 오류는 파일 변경 없이 중단되었으며,
소스 순서로 다시 생성한 패치를 적용한 뒤 이 검사를 통과했다.

기존 집중 368건에 다음 기존 계약 12건을 추가한다. 테스트 원본·기대값·ignore는 변경하지 않는다.

| 기존 계약 | 의미와 한계 |
| --- | --- |
| #6981 8건 | 걸친 셀의 모든 목표 줄/해당 쪽의 셀 하한, 늘리면 안 되는 거대 컷과 재분할, 시작 컷 소비·빈 물리 밴드 재예약 방지, 종료 후 후속 본문 보존 |
| #6981 예산 변화(위 8건 중 포함) | 원본의 구역 28/표를 추출한 IR 입력에서 하단 여백을 ±1500HU, 75HU 간격으로 변경. 표/줄 하한·유일한 내용 소유와 페이지 전이를 검사. 한컴 재저장 출력은 아님 |
| #3820 4건 | `76076`의 4/5쪽 소유·표 하단, 35/36쪽 blank band/후속 내용 좌표와 중복 방지, 18/19쪽 pseudo-tail 비수용, 저장 LineSeg 없는 중첩 표 보존 |

#6981 재분할 대조군은 기존 셀 밖 글줄 최대 3건을 허용한다. 이 기존 계약의 유지를
overflow 0 또는 한컴 일치로 보고하지 않는다. #3820은 기존 PDF 유래 좌표 범위를 유지하며,
이번 실행에서 PDF를 다시 직접 판독한 증거로 승격하지 않는다. 기존 368건의 한계는
[Stage44](task_m100_7280_stage44.md)~[Stage46](task_m100_7280_stage46.md)을 따른다.
이 테스트 선택으로 모든 내부 조건 조합을 실행했다는 주장은 하지 않는다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3f`.
공유 `target/pr-review`를 보존하고 jobs 4 / test threads 8로 순차 실행한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage47/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 1분 01초.
- 집중 nextest **380 passed / 0 failed**, 25 binaries, 필터 비선택 8,027건, exit 0.
  비선택은 전체 회귀의 기존 ignore 50건과 구분한다. 빌드 7분 05초, 테스트 11.068초.
  run ID: `db4ab8ba-d3f1-4b7a-9081-0fc7c8941695`.
- `output/7280/stage47/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  같은 380개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀는 재실행하지 않았다.
- 실행 로그: `output/7280/stage47/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  추가 계약의 원본 입력 4개 SHA-256은 `fixture.sha256`에 기록했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 코드 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했지만, R3 전체 회귀·Native/fresh Docker WASM
시각 비교와 최종 제출 게이트는 미실행이다. 기존 계약 재실행이며 결함 수정 전 FAIL/후 PASS를
주장하지 않는다. 원격 작업은 하지 않았다.

## 다음 절편

일반 행의 저장 프레임 선택·whole-row fit 판단을 분리한다. 가로 용지/행 내부 컷,
스캔 조정·continuation 본체와 상태 소유 경계도 남아 있다. R3 전체 회귀·Native/fresh Docker
WASM 시각 비교 및 최종 제출 게이트는 아직 미실행이다. 원격 push·PR·댓글은 승인 범위가 아니다.
