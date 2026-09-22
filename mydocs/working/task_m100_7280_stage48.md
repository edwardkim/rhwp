# Task #7280 Stage 48 — R3g 일반 행 저장 프레임·통째 수용 Query 분리

- Issue: #7280. 이전: [Stage47](task_m100_7280_stage47.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `67b2e3265`. 제품: `4a9804d0515f4e32f37113ce7d765c93b94fd34a`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3g 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/source_frame.rs`로 일반 행의 저장 프레임 선택과 통째 수용 판단을 옮겼다.
`SourceFrameQuery`는 읽기 전용 행 조회, 현재 행의 시작 컷, 행 수, continuation 여부와
호환성 profile을 받는다. `WholeRowBudget`은 앞서 수용한 높이·행 간격·현재 요구 높이와
예산/기존 허용치를 구분한다. 가변 TypesetState나 TypesetEngine 전체를 전달하지 않는다.

- `SourceFrameSelection`은 후속 컷에 필요한 저장 프레임과 첫/중간/이어받기/끝 행 조건을
  반환한다. `RowCutResult`는 clone 없이 이동한다.
- reflow 여부와 다음 행의 텍스트/컨트롤 유무 조회는 콜백으로 전달한다. 조회를 미리
  실행하지 않고 원래의 선행 guard와 단락 평가 순서대로 실행한다.
- 온전한 행 수용 실패 후의 저장 행 높이 재판정, 가로 용지 처리, 내용 컷 생성·확장·이월과
  누적 예약·커서 전진은 부모에 그대로 남긴다. 후속 계산을 선행 Query로 당기지 않는다.

기존 조건·허용 수치·출력은 보존 대상이다. 기존 source-frame 예외를 새로운 조판 규칙으로
승인하거나 한컴과의 차이를 수정한 작업이 아니다. 테스트·기준값·ignore는 변경하지 않았다.

## 실제 호출·컷·높이·배치 연결

제품 `4a9804d05` 기준:

1. `typeset.rs:22255` 일반 스캔과 `:22388` 각주 예약 후 refit은 같은
   `scan_block_table_split_rows`를 호출한다. 블록 수용 및 rowspan 전용 경로가 처리하지
   않은 일반 행에서만 `:17855`의 새 Query를 실행한다. 별도 TAC 경로는 비해당이다.
2. 직전 `RowScanQuery`가 생산한 `row_total`, 실제 누적 `consumed`, `cs_before`와 가용
   예산을 읽는다. 저장 profile → 다음 빈 행 → 저장 프레임 → 첫/중간 조각 reflow 검사 →
   기존 수용 허용치 → 선언 저장 프레임 검사 순서를 원본과 대조했다.
3. `whole_row_fits`이면 부모에서 기존 bleed 기록 초기화, 요구 높이 예약, 행 전진을
   수행한다. 실패할 때만 기존 저장 행 높이 수용 판단을 실행한다. 실제 예약은 두 경우
   모두 기존 `cs_before + row_total`이며 이를 새 값으로 대체하지 않는다.
4. 미수용 행은 기존 일반 컷을 먼저 생성한다. `:18125` 이후 source-frame tail 조건이
   반환된 flags를 소비한다. 기존 `stored_source_frame.or_else(...)`는 `:18156`에 남겨
   컷 소유권과 fallback의 지연 실행을 유지했다. 이후 경계 제한·요구 높이·예산 검사는
   바꾸지 않았다. 끝 행의 두 줄 프레임 높이도 기존 후속 수용 분기에서 소비한다.
5. `:22662` PartialTable 발행, `:22749` continuation 전진으로 기존 결과가 전달된다.
   내용 컷/물리 밴드·패딩·헤더/각주 예약, 종료 후 캡션·후행 본문 처리는 변경하지 않았다.

정적 검사는 부모 전체의 허용된 호출 치환 외 변경이 없음을 확인한다. 모든 분기의 동적
커버리지나 측정/paint의 새로운 공통화를 주장하는 것은 아니다.

## 검증 계약과 한계

`output/7280/stage48/{extract-source-frame.mjs,extraction-proof.json}`에 원본 추출과
정적 대조를 보존한다. 계산식·조건 순서, 부모의 후속 컷/상태 반영/호출자 보존과 새로운
clone·가변 상태·source-side 테스트가 없음을 확인했다.

기존 집중 380건에 다음 **기존 계약 7건**을 추가한다. 모두 기존 integration suite에 있으며
새 source-side 테스트나 공개 API를 추가하지 않는다.

| 계약 | 실제 검사 및 한계 |
| --- | --- |
| #5584 1건 | 저장 프레임 중간 행 뒤 한 유닛 꼬리 페이지 방지의 기존 4쪽 계약. 쪽수만 검사하므로 기하/피델리티 증거는 아님 |
| #6790 4건 | 예산을 넘는 source tail의 3쪽 분할, 첫 조각 표 하단 본문 초과 ≤0.5px, 예산 내 확장 대조군 4쪽, 확장 비적용 Square 대조군 2쪽 |
| #5057 2건 | 원본 HWP5 13쪽 및 export HWPX에서 출처 표식만 제거한 사본의 같은 쪽수. 첫 조각 whole-row 허용 경로이며 source-tail 확장 경로의 증거가 아님 |

#6790의 잔여 글자 결손/겹침 축은 기존 계약이 검사하지 않는다. #5057의 표식 제거 사본은
한컴이 직접 생성한 HWPX가 아니다. 기존 계약 통과를 한컴 시각 일치로 승격하지 않는다.
기존 380건의 한계는 [Stage47](task_m100_7280_stage47.md) 및 연결된 앞 절편을 따른다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3g`.
공유 `target/pr-review`를 보존하고 jobs 4 / test threads 8로 순차 실행한다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage48/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 59.44초.
- 집중 nextest **387 passed / 0 failed**, 25 binaries, 필터 비선택 8,020건, exit 0.
  비선택은 전체 회귀의 기존 ignore와 구분한다. 빌드 7분 01초, 테스트 11.242초.
  run ID: `963b4ec2-e777-477c-827c-9350622a0815`.
- `output/7280/stage48/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  같은 387개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀는 재실행하지 않았다.
- 실행 로그: `output/7280/stage48/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  추가 계약의 원본 입력 4개 SHA-256은 `fixture.sha256`에 기록하고 실행 뒤 재확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 코드 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했지만, R3 전체 회귀·Native/fresh Docker WASM
시각 비교와 최종 제출 게이트는 미실행이다. 기존 계약 재실행이며 결함 수정 전 FAIL/후 PASS를
주장하지 않는다. 원격 작업은 하지 않았다.

## 다음 절편

일반 행의 가로 용지 수용/분할 조건을 분리한다. 후속 행 내부 컷/저장 프레임 확장과
스캔 상태 소유·continuation 본체도 남아 있다. R3 전체 회귀·Native/fresh Docker WASM
시각 비교 및 최종 제출 게이트는 아직 미실행이다. 원격 push·PR·댓글은 승인 범위가 아니다.
