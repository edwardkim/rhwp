# Task #7280 Stage 52 — R3k 저장 프레임 후보 보정·확장 적합성 Query 분리

- Issue: #7280. 이전: [Stage51](task_m100_7280_stage51.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `ab1b41eed`. 제품: `904829bdc683ad461f4e47c30d9dba27a7f7ec20`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3k 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/source_tail/extension.rs`에 후보 컷의 mirrored rewind 보정과 확장 적합성 계산을
분리했다. 기존 `SourceTailQuery`의 읽기 전용 행/시작 컷을 재사용하며, 전체 TypesetState나
TypesetEngine·가변 페이지 상태를 새 인자로 전달하지 않는다.

- `mirrored_correction`: 저장 rewind와 가시 셀 목록을 원래 순서로 조회한다. 각 가시 셀에서
  일반 컷 이후·후보 컷 이전의 첫 경계를 고르고, 둘 이상이 같은 경계를 가질 때만 원래와 같이
  끝 컷을 한 번 clone한다. 실제 줄어든 컷이 있을 때만 해당 컷 높이를 조회한다.
- `SourceTailCorrection`: 보정한 끝 컷과 패딩을 제외한 소비 높이를 반환한다. 부모가
  끝 컷·소비 높이·`fully_consumed=false`를 원래 순서로 반영한다. 원래 `hit_hard_break` 등
  나머지 후보 속성은 그대로 두며, 보정할 경계가 없으면 기존 후보를 변경하지 않는다.
- `extension_fit`: 보정이 반영된 후보를 받아 확장량 → 중간 프레임/어울림 분기 → 조건부
  잔여 높이 → 근소 확장 상한 → 쪽 예산 순서로 계산한다. 숫자 조건·단락 평가를 바꾸지 않는다.
  원래 `avail_for_rows`를 사용하며 이를 `avail_for_rows - consumed`로 재해석하지 않는다.
- `SourceTailFit`: 계산 결과만 반환한다. 양의 확장/상한/쪽 소유의 최종 결합과
  `budget/res/uses_source_frame_tail` 쓰기는 부모에 남긴다.

새 예외·허용치·공개 API·source-side 테스트는 추가하지 않았다. 이번 작업은 기존 조판의 구조
보존이며, 현재 경계 추론이나 숫자 허용치의 정확성을 새로 승인하는 것이 아니다. 기존 설명과
이슈 이력 주석도 계산 구현과 함께 이동했다.

## 실제 호출·컷·높이·배치 연결

제품 `904829bdc` 기준:

1. `typeset.rs:22003` 일반 스캔과 `:22136` 각주 예약 뒤 refit이 같은 행 스캔을 호출한다.
   선행 블록/rowspan 전용·일반 행 통째 수용·분할 불가 경로를 지난 후에만 이번 경계에 도달한다.
   저장 꼬리 gate가 거짓이거나 후보가 없을 때는 새 두 메서드를 호출하지 않는다.
2. `:18070` 후보 보정 Query가 일반 끝 컷과 후보 끝 컷을 받는다. `extension.rs:68` 저장
   경계와 가시 셀을 조회하고, `:88` 동일 경계 판정, `:114` 실제 보정 컷의 내용 높이 조회를
   수행한다. 부모가 보정안을 반영한 다음에만 `typeset.rs:18080` 적합성을 조회한다.
3. `extension.rs:149` 확장량, `:172` bounded 분기, `:174` 조건부 잔여 높이, `:215` 쪽
   예산 판정은 동일 수식을 쓴다. 내용 컷·패딩·물리 행 예산을 한 값으로 합치지 않는다.
4. `typeset.rs:18091` 수용 시 후보 높이를 budget에 넣고 같은 후보를 res로 이동한다.
   미수용 시 일반 컷/예산을 유지한다. `:18255` 이후 실제 컷 높이 재측정과 fit/retry·누적
   예약·이월은 그대로다. 후보가 계산됐다는 이유만으로 물리 공간을 예약하지 않는다.
5. `:22410` PartialTable 발행과 `:22497` continuation 전진은 기존 컷/높이를 소비한다.
   종료 뒤 캡션·각주·후행 본문, 빈 밴드/남은 행 높이 소유와 최종 paint는 변경하지 않았다.

`output/7280/stage52/{extract-tail-extension.mjs,extraction-proof.json}`에 시작 head의 원본
경계 선택·clone·높이 조회·수용 식과 새 메서드의 정적 대조를 보존한다. 부모 전체도 허용된 호출
치환 외에 변경하지 않았음을 검사했다. 이 증거를 모든 경계의 동적 커버리지로 해석하지 않는다.

## 검증 계약과 한계

기존 394건에 기존 #6549 계약 1건을 추가했다. 테스트 원본·기대값·ignore는 바꾸지 않는다.

| 계약 | 실제 검사 및 한계 |
| --- | --- |
| 추가 #6549 1건 | `16418295_square_rowbreak_table.hwp`의 2쪽 핀. 어울림 분기의 기존 수용 결과 보호이며, 표 외곽의 무초과나 모든 줄 소유를 직접 검사하지 않음 |
| 기존 #6973 3건 | 두 셀 rewind `[9]`, 총 9쪽, 8쪽 표 바닥 초과 `< 30px`. 보정 경계의 소비 결과를 일부 보호하지만 반복 경계/중첩 유닛 투영의 모든 조합은 미검증 |
| 기존 #5584·#6790·#5057 | 단일 유닛 꼬리·프레임 예산·대조군/첫 프레임 계약. #5057을 이번 꼬리 확장 갈래의 실행 증거로 쓰지 않음 |
| 기존 #2308·#3820·#3738 등 | 앞 절편의 내용 소유·분할·각주/본문 경계 계약을 유지. 상세 범위/한계는 Stage51과 연결된 기록 참조 |

#6973의 기존 `< 30px` 상한은 완전한 무초과를 뜻하지 않는다. 테스트 통과를 잔여 결함 해결이나
한컴 시각 일치로 보고하지 않는다. 같은 조각에서 여러 경계가 생기는 모든 경우나 유닛
누락·중복의 전수 검증은 이번 집중 실행의 증거가 아니며, 새 결함 수정 전 FAIL/후 PASS도
주장하지 않는다. 원본 조건/호출 순서 대조와 기존 계약 재실행을 구분한다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3k`.
공유 `target/pr-review`를 보존한다. 16 CPU / RAM 31GiB 환경에서 jobs 4 / test threads 8로
순차 실행한다. 실제 `--prepare`한 suite 023의 #6549 위치를 확인한 뒤 runner를 고정했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage52/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 58.48초.
- 집중 nextest **395 passed / 0 failed**, 27 binaries, 필터 비선택 8,449건, exit 0.
  비선택은 기존 ignore와 구분한다. 빌드 7분 18초, 테스트 11.231초.
  run ID: `44a1cba7-5efb-4862-9944-1246705a4193`.
- `output/7280/stage52/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  동일 395개 PASS 이름과 일치함을 확인했다. 전체 회귀 재실행 결과는 아니다.
- 실행 로그: `output/7280/stage52/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  이번 경계와 연결한 입력 2개의 SHA-256을 `fixture.sha256`에 기록하고 실행 뒤 재확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 기존과 동일하다.

검증 후 제품 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 로그는 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했다. R3 전체 회귀·Native/fresh Docker WASM
시각 비교·최종 제출 게이트는 미실행이며, 한컴 출력과의 신규 시각 일치는 주장하지 않는다.
원격 push·PR·댓글은 수행하지 않았다. 기존 worktree와 공유 target은 보존했다.

## 다음 절편

일반 컷 이후 wrapper prefix의 이월/안전 컷과 실제 점유 높이 재판정 경계를 검토한다.
상태 수용·예약·이월은 부모 소유로 유지한다. 스캔 상태/continuation 본체와 R3 전체 회귀·
Native/fresh Docker WASM 시각 비교·최종 제출 게이트가 남아 있다. 원격 작업은 하지 않는다.
