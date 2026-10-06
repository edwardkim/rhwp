# Task #7280 Stage 49 — R3h 가로 용지 행 수용·분할 조건 Query 분리

- Issue: #7280. 이전: [Stage48](task_m100_7280_stage48.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `e1cc64ea2`. 제품: `a9a8d49bcb6d7caa9194c3fb8d53c93a064a10a6`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3h 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/landscape.rs`에 일반 행의 기존 가로 용지 수용 조건을 분리했다.
`LandscapeRowQuery`는 읽기 전용 행 조회·profile·시작 컷·continuation/분할 조건과
직전 수용 행 높이를 받는다. `LandscapeRowBudget`은 누적 수용 높이·행 간격·현재 행의
요구 높이·가용 예산을 별도 값으로 유지한다. 가변 TypesetState/TypesetEngine은 전달하지 않는다.

- `whole_row_shape`: 기존 전체 행 허용 범위와 같은 높이 연속 수용 금지를 조회한다.
- `short_row_shape`: 첫 수용 실패 뒤 짧은 행 조건을 조회한다. rowspan과 profile별 저장
  reset 검사 순서를 유지하며 native reset 조회 콜백은 기존 guard 안에서만 실행한다.
- `boundary_splittable`: 세 호출 지점에서 기존 분할 허용·행 분할 가능성·저장 분할 선언
  조건을 조회한다. 중복 조회를 선행 계산으로 합치지 않아 실행 시점과 횟수를 보존한다.

부모는 실제 수용, `bleed_absorbed_row_height` 기록, 누적 높이·행 전진, 경계 분할 flag와
진단을 소유한다. 입력 snapshot 이후 높이/행이 바뀌는 두 수용 분기는 곧바로 `continue`하므로
같은 Query가 변경 후 상태를 읽을 필요가 없다. 실패 경로에서는 예산 값이 변하지 않는다.

기존 `landscape_rowbreak_bleed`는 호출자가 `body_area.height < 700.0`으로 계산하며
profile별 whole/short 허용치도 호출자에 남겼다. 이름을 근거로 정확한 용지 방향 판정이라
주장하거나 이 예외의 타당성을 새로 승인하지 않는다. 수치·조건·출력을 보존하는 구조 작업이며
조판 수정, baseline·ignore 완화, 테스트 추가/삭제는 하지 않았다.

## 실제 호출·컷·높이·배치 연결

제품 `a9a8d49bc` 기준:

1. `typeset.rs:22225`의 일반 스캔과 `:22358` 각주 예약 후 refit이 같은
   `scan_block_table_split_rows`를 호출한다. 블록/rowspan 전용 경로와 일반 저장 프레임
   통째 수용 경로가 처리하지 않은 행에서 `:17911` 새 Query를 만든다. 별도 TAC는 비해당이다.
2. `:17929` whole-row 형상을 먼저 평가한다. 형상이 맞을 때만 `:17937` 분할 가능성을
   조회한다. 분할 불가이면 기존 높이를 예약하고 다음 행으로 이동한다.
3. 그 경로에서 수용하지 않은 경우 `:17945` short-row 조건을 평가하고, 맞을 때만
   `:17952` 분할 가능성을 조회한다. 수용 시 기존 기록/예약/전진을 수행한다. 분할 가능이면
   부모의 `landscape_boundary_splittable`만 켠다. `:17961` whole-row 재확인도 원래 위치에 남겼다.
4. 후속 일반 컷과 `row_cut_content_height` 측정은 그대로다. `:18521`의 경계 밴드 보존
   조건이 위 flag, 실제 소비 높이/컷 유닛, 남은 밴드를 읽어 content-only 고아 가드 적용을
   결정한다. 이후 요구 높이/예산 실패 시 재-cut·이월·예약도 바꾸지 않았다.
5. `:22632` PartialTable 발행과 `:22719` continuation 전진으로 기존 컷·점유 결과를
   전달한다. 내용 컷과 물리 밴드·패딩·헤더/각주·마지막 유닛 뒤 후속 내용은 변경하지 않았다.

`output/7280/stage49/{extract-landscape.mjs,extraction-proof.json}`은 원본 두 조건식과
반복된 분할 판단, 부모 전체의 허용한 호출 치환 외 변경이 없음을 대조한다. 모든 분기의
동적 커버리지나 paint 경로의 신규 공통화를 뜻하지 않는다.

## 검증 계약과 한계

기존 집중 387건에 다음 기존 integration 계약 **2건**을 추가한다. source-side 테스트나
테스트용 공개 API를 추가하지 않는다.

| 계약 | 실제 검사 및 한계 |
| --- | --- |
| #5828 | 절단 fixture가 13쪽 이상인지, 앞 최대 6쪽 SVG의 `<line>` y1 최댓값이 732px 이하인지 검사. 본문 밖 모든 요소·모든 쪽·정확한 행 수를 검사하는 것은 아님 |
| #6307 | 11쪽에서 ShadowOffsetX 행 부재, UnderlineColor 셀 조각 존재/높이 30px 미만, 표 하단 ≤737.5px. 12쪽 이후 전체 유닛 보존·누락 없는 소유를 전수 검사하지는 않음 |

#5828 입력은 원본 HWP5 구역3 일부를 잘라 스텁을 넣은 기존 fixture다. #6307 기대값은
기존 한컴 2022 PDF 관측과 본문 기하를 근거로 작성된 계약이며 이번 실행에서 PDF를 새로
직접 판독한 것은 아니다. 기존 387건의 범위·한계는 [Stage48](task_m100_7280_stage48.md)와
연결된 앞 절편을 따른다. 기존 계약 재실행을 한컴 시각 일치나 결함 수정 전 FAIL/후 PASS로 보고하지 않는다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3h`.
공유 `target/pr-review`를 보존하고 jobs 4 / test threads 8로 순차 실행한다.
초기 실행 목록 작성에 참고한 주 작업 디렉터리의 generated suite는 오래된 배치였다.
검증 worktree에서 `--prepare`한 실제 목록과 baseline PASS 이름을 대조하여 #5828이
suite 003, #6307이 suite 008에 있음을 확인했다. 누락된 suite 003을 추가한 최종 실행을
판정 대상으로 삼는다. 파생 suite 번호를 테스트 원본의 고정 소속으로 취급하지 않는다.
초기 nextest는 388건 통과했지만, 실행 중 runner 파일을 보완한 탓에 Bash의 후속 읽기에서
`--test-threads: command not found`(exit 127)가 발생했다. 이는 제품 실패가 아니라 검증
스크립트 변경 절차의 오류이며 최종 통과 증거로 사용하지 않는다. 해당 로그는
`nextest-initial-incomplete-selection.log`로 보존하고, `bash -n` 통과 뒤 고정한 runner로
다시 실행한다. 이후 실행 중인 스크립트는 수정하지 않는다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage49/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 59.51초.
- 최종 집중 nextest **389 passed / 0 failed**, 26 binaries, 필터 비선택 8,228건, exit 0.
  비선택은 전체 회귀의 기존 ignore와 구분한다. 최종 추가 빌드 8.19초, 테스트 11.389초.
  앞선 공통 바이너리 빌드는 7분 01초였으며 최종 실행에서 같은 제품의 빌드 캐시를 재사용했다.
  run ID: `9ff9639a-0d9f-4fe8-88fe-9e0d68857bde`.
- `output/7280/stage49/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  동일 389개 PASS 이름과 일치함을 확인했다. 선택 밖 전체 회귀는 재실행하지 않았다.
- 실행 로그: `output/7280/stage49/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  추가 입력 2개 SHA-256을 `fixture.sha256`에 기록하고 실행 뒤 재확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 이전과 동일하다.

검증 후 제품 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 산출물은 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했지만 R3 전체 회귀·Native/fresh Docker WASM
시각 비교·최종 제출 게이트는 미실행이다. 원격 작업은 하지 않았다.

## 다음 절편

일반 행의 분할 진입·마지막 주석 행 수용 조건을 분리한다. 후속 컷/저장 프레임 확장,
스캔 상태 소유·continuation 본체와 R3 통합 검증도 남아 있다. R3 전체 회귀·Native/fresh
Docker WASM 시각 비교 및 최종 제출 게이트는 아직 미실행이다. 원격 작업은 승인 범위가 아니다.
