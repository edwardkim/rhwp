# Task #7280 Stage 44 — R3c rowspan 블록 분류·요구 높이 Query 분리

- Issue: #7280. 이전: [Stage43](task_m100_7280_stage43.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `11da183aa`. 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3c 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`scan_block_table_split_rows`의 rowspan 보호 블록 분류, 행 오프셋, 컷 조각 높이 및
요구 높이 계산을 `table/scan.rs`의 `RowBlockQuery`로 분리한다. 입력은 원본 MeasuredTable,
실제 행 기하 Table, LayoutEngine, 스타일, 컷용 행 높이, rowspan 표식과 행 간격의 읽기 전용 값이다.
TypesetEngine/TypesetState나 가변 문서·페이지 상태를 전달하지 않는다.

분류는 기존 반복문의 같은 위치에서 수행한다. 블록 분기 진입 뒤에만 행 오프셋과 요구 높이를
계산한다. 소모한 유닛 컷과 물리 높이는 합치지 않는다. 시작 행의 빈 물리 밴드를 수용하는
`start_row_height_override`의 선행 continue, offset 컷의 재시도, 결과 반영은 부모에 남긴다.
기존 조건·허용치·float 연산/호출 순서와 반환을 보존하고 조판 정책의 타당성을 새로 승인하지 않는다.

## 호출·컷·높이·실제 배치 연결

1. continuation의 일반 스캔과 각주 예약량을 되돌린 refit 스캔 모두 기존
   `scan_block_table_split_rows`를 호출한다. 양쪽의 `row_geometry_table`, 원본 `mt`,
   `cut_row_h`/`whole_row_fit_h`, 시작 컷 및 가용 높이 구분은 유지한다.
2. Query의 분류 결과가 보호 블록/RowBreak rowspan 블록 경로와 행 오프셋 사용을 고른다.
   해당 블록의 첫 행이 아니거나 분류가 비해당이면 기존 일반 행 경로로 계속한다.
3. `required_height`는 시작 컷 유무·행 오프셋 여부에 따라 기존 행 합 또는 컷 내용 높이를 반환한다.
   이 높이를 부모가 `consumed + cs_before + block_h`로 예산과 비교하고 수용 시 누적한다.
4. 예산 실패 시 남은 예산으로 `advance_row_block_cut[_with_row_offsets]`를 호출한다.
   fresh 페이지 높이 비교, 밴드 재시도·강제 수용·다음 행 이월 판단은 이동하지 않는다.
5. 분할 수용 시 `fragment_height`의 행별 합과 컷 워크 높이를 기존 허용치로 대조한다.
   `split_end_cut`/`split_end_limit`/`split_block_start` 및 누적 높이는 기존 부모가 결정한다.
   이후 PartialTable 발행·커서 전진·각주·종료·paint의 별도 보정도 그대로다.

이번 분리는 컷 소유권·물리 밴드·요구 높이의 기존 관계를 보존하는 것이지 모든 경로를
단일 측정 결과로 교체하는 작업이 아니다. 블록 컷 이외 일반 행/거대 행·rowspan 끝행,
단일 행 컷의 판단은 후속 절편으로 남긴다.

제품 `8a32f7c34`의 연결 위치는 `table/scan.rs:32`(분류), `:95`(오프셋),
`:117`(컷 조각 높이), `:161`(요구 높이), `typeset.rs:17607`(요구 높이 소비),
`:17619`(남은 예산), `:17833`/`:17834`(끝 컷·누적 높이), `:22485`/`:22618`(일반/refit 호출),
`:22892`(PartialTable 컷 전달), `:22979`(커서 전진)다. 마지막 내용 소비 후 종료와
각주·캡션 등 후속 높이 정산도 원본 그대로이며 이번 절편의 정적 대조 범위에 포함한다.

## 검증 계획과 한계

고정 baseline의 기존 정식 계약을 독립적인 동작 기준으로 삼는다. 정적 대조에서 원본 분류식·
오프셋·컷 높이·요구 높이를 새 Query와 비교하고, 두 스캔 호출 및 나머지 부모 전체의 불변을 검사한다.
기존 집중 353건에 #6024, #6756, #6803, #7226의 rowspan 이어받기·중복·유닛 분할·본문 하한
계약을 추가한다. tests/cases 원본·기대값·baseline·ignore·source-side 테스트/support는 바꾸지 않는다.

추가 선택의 실제 검사 범위는 다음과 같다. 기존 계약을 그대로 재실행하며 기대값을 새로 정하지 않는다.

| 기존 계약 | 검사하는 의미와 한계 |
| --- | --- |
| #6024 1건 | RowBreak/CellBreak 이어받기 쪽에서 이미 그린 병합 셀 라벨을 재출력하지 않음 |
| #6756 2건 | 전체 글자 수의 기존 범위 및 2쪽 용지 하한 +0.5px. 문자 수만으로 완전한 소유권을 주장하지 않음 |
| #6803 3건 | 문단 인덱스가 조각 사이에서 누락·중복 없이 연속. 용지 하한 검사는 기존 40px 허용치를 보존하며 엄격한 overflow 0 검사가 아님 |
| #7226 5건 | rowspan 줄 소유·마지막 줄 보존, 이어받기 겹침, 쪽수, 마지막 줄/셀/뒤 행/본문 +0.5px, 인접 예산 대조 |

#7226 인접 예산 검사는 원본에서 구역/표를 골라 여백을 ±75HU 바꾼 합성 계약이다.
정상 저장 실물·별도 한컴 PDF와 동등한 근거로 승격하지 않는다. 일반/각주-refit 호출 보존은
정적으로 확인하지만 이번 집중 실행이 모든 내부 분기를 거쳤다는 동적 커버리지 주장은 하지 않는다.

전용 review worktree에서 suite 준비, 고정 baseline 명시 정책, fmt, native Clippy, 집중 nextest를
공유 `target/pr-review`에서 순차 실행한다. §7.2 R3 책임 묶음 전체 회귀·Native/fresh Docker WASM
시각 검증과 §7.3 최종 제출 게이트는 아직 미실행이다. 정적 동등성과 계약 통과를 모든 분기의
동적 실행이나 한컴 시각 일치로 해석하지 않는다. 원격 push·PR·댓글은 승인 범위가 아니다.

## 고정 head 검증 결과

- 제품 SHA: `8a32f7c340ac0b6e65f16bb51bb3daa104a5dc8b`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r3c`.
- host 16 logical CPUs / RAM 31 GiB, 가용 약 19 GiB. 실행 전 다른 Cargo/Rust 작업이 없음을 확인했다.
  공유 `target/pr-review` 약 196 GiB를 보존하고 빌드 jobs 4 / test threads 8로 순차 실행했다.
- 정적 대조: `output/7280/stage44/{verify-row-block.mjs,extraction-proof.json}`.
  분류·오프셋·조각/요구 높이 계산식, 두 스캔 호출·소비/예약·continuation을 포함한 나머지
  부모 본문이 동일하다. Query에는 TypesetEngine/TypesetState·가변 입력·source-side test가 없다.

review worktree에서 실행한 명령:

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage44/run-focused.sh
```

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 1분 00초.
- 집중 nextest **364 passed / 0 failed**, 24 binaries, 필터 비선택 7,845건, exit 0.
  비선택은 전체 회귀의 기존 ignore 50건과 구분한다. 빌드 6분 52초, 테스트 7.504초.
  run ID: `4a91b951-a5a2-4924-892b-a9cd605a6f73`.
- 로그: `output/7280/stage44/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  `compare-focused.mjs` / `regression-comparison.json`에서 고정 baseline의 같은 364개 PASS 이름과
  일치함을 확인했다. 추가 선택 4개 실물 입력의 SHA-256은 `fixture.sha256`에 기록했다.
- nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 기존과 동일하다.
  원격 CI와 도구 환경이 완전히 같다는 의미는 아니다.

검증 후 제품 코드 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
baseline·ignore·테스트 기대값 변경이나 원격 push·PR·댓글은 없다.
이번 절편의 구조 보존·기존 집중 계약은 충족했지만 R3 전체 회귀·Native/fresh Docker WASM
시각 비교 및 최종 제출 게이트는 미실행이다. 기존 계약 재검증이며 결함 수정 전 FAIL은 주장하지 않는다.

## 다음 절편

블록 예산 실패 뒤 컷 수용/이월 판단과 밴드 재시도의 조회 경계를 분리한다.
누적 예약·컷 확정·커서 전진은 순서를 유지하면서 별도로 관리하고, 일반 행·rowspan 끝행 및
continuation 본체 분리와 R3 책임 묶음 통합 검증으로 이어간다.
