# Task #7280 Stage 51 — R3j 저장 프레임 꼬리 진입·후보 컷 Query 분리

- Issue: #7280. 이전: [Stage50](task_m100_7280_stage50.md).
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md) R3, §7.1.
- 시작 head: `27ac65ff9`. 제품: `f7a5179c877177022b4a01cfa6e6b9e8be205552`.
- 고정 동작 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R3j 구조 분리·정적 대조·집중 검증 완료. R3 전체·최종 제출 게이트 완료는 아니다.

## 이번 경계

`table/scan/source_tail.rs`에 저장 프레임 꼬리의 진입 조건과 후보 컷 조회를 분리했다.
`SourceTailQuery`는 읽기 전용 행 Query·시작 컷·호환 profile·선행 프레임 분류만 받는다.
전체 TypesetState/TypesetEngine이나 가변 페이지 상태는 전달하지 않는다.

- `gate`: 일반 예산 컷 뒤 plain-text 저장 reset → 꼬리 계약 → 중간 프레임 전용 여부 →
  profile/TAC guard 순서로 조회한다. 기존 단락별 short-circuit을 유지한다.
- `candidate`: gate가 참일 때만 호출한다. continuation 또는 소비 높이 `<= 0.5`이면
  다음 가시 유닛 컷, 그 외에는 저장 컷 또는 지연 평가한 문단 꼬리 컷을 선택한다.
  컷은 clone 없이 소유권을 전달하며 조회하지 않던 fallback을 미리 계산하지 않는다.
- 후속 mirrored rewind 보정, 확장량/잔여 높이/쪽 예산 수용, `budget/res` 대입과
  `uses_source_frame_tail` 변경은 부모에 남겼다. 후보 조회와 실제 수용을 구분한다.

새 예외·허용치·공개 API·source-side 테스트는 추가하지 않았다. 기존 조건의 구조 분리이지,
현재 저장 프레임 정책이나 숫자 허용치의 정확성을 새로 승인한 것은 아니다.

## 실제 호출·컷·높이·배치 연결

제품 `f7a5179c8` 기준:

1. `typeset.rs:22157` 일반 스캔과 `:22290` 각주 예약 뒤 refit이 같은 행 스캔을 호출한다.
   블록 전용 분기/통째 수용/분할 불가 경로를 지난 일반 행에서만 이번 Query에 진입한다.
   별도 TAC 배치와 선행 조기 반환 경로의 기존 동작은 이동하지 않았다.
2. `:18044` mixed-nested reserve가 일반 `res/budget`을 만든 후 `:18064` gate,
   `:18066` guard 안에서 `:18067` 후보 컷을 조회한다. 시작 컷과 일반 끝 컷은 동일하다.
3. 후속 mirrored rewind 보정은 부모에서 계속 수행한다. `:18166` 확장량,
   `:18189` 중간 프레임/어울림 상한 분류와 잔여 높이, `:18238` 쪽 예산 판정을 거쳐
   `:18245`에 수용한 후보의 높이와 컷을 함께 대입한다. 후보만 조회했다고 예약하지 않는다.
4. `:18409` 실제 선택 컷의 높이 재측정과 이후 fit/retry·예약·이월은 원본 그대로다.
   내용 컷에 없는 물리 밴드/패딩, 종료 후 캡션·각주·후행 본문의 처리를 변경하지 않았다.
5. `:22564` PartialTable 발행과 `:22651` continuation 전진이 기존 결과를 소비한다.
   최종 paint 경로에 새 좌표 보정이나 출력 억제를 추가하지 않았다.

`output/7280/stage51/{extract-source-tail.mjs,extraction-proof.json}`은 시작 head에서 추출한
조건식·후보 선택식과 변경 후 Query를 대조하고, 부모 전체가 허용된 호출 치환 외에는 동일함을
검사한다. 공백/포맷을 정규화한 정적 증거이며 모든 분기의 실행 커버리지를 의미하지 않는다.

## 검증 계약과 한계

기존 390건에 기존 계약 4건을 추가했다. 테스트 원본·기대값·ignore는 변경하지 않는다.

| 계약 | 실제 검사 및 한계 |
| --- | --- |
| 추가 plain-text saved reset 1건 | 합성 입력에서 텍스트 저장 reset은 참, 컨트롤 문단의 로컬 reset은 거짓인지 확인. helper 판별 계약이며 end-to-end gate 전체를 입증하지 않음 |
| 추가 #6973 3건 | `83818` 9쪽 핀, 8쪽 표 바닥의 본문 초과 `< 30px`, 두 셀의 저장 rewind `[9]` 확인. 완전한 무초과/한컴 일치나 유닛 누락·중복 전수 검사는 아님 |
| 기존 #5584·#6790·#5057 | 단일 유닛 꼬리, 확장 프레임 예산 및 정상 대조군, 첫 프레임 수용 계약 유지. 상세 범위는 Stage48과 연결된 기록 참조 |
| 기존 #2308·#3820·#3738 등 | 앞 절편의 내용 소유·분할·각주/본문 경계 계약 유지. 나머지 범위/한계는 Stage50 참조 |

#6973의 기존 문서에는 8쪽 17.7px 잔여 초과가 기록되어 있으며 테스트도 `< 30px` 상한만
검사한다. 이번 통과를 그 잔여 문제 해결이나 시각 일치로 격상하지 않는다. 후보 뒤의 보정 코드는
이번에 변경하지 않았지만 소비 경로 보호를 위해 해당 기존 integration 계약을 함께 선택했다.
신규 reset 분기 전수 조합·컷 소유권 전수 커버리지나 수정 전 FAIL/후 PASS를 주장하지 않는다.

review worktree: `/home/edward/mygithub/rhwp-review-7280-r3j`.
공유 `target/pr-review`를 보존한다. host 16 CPU / RAM 31GiB 확인 후 jobs 4 / test threads 8로
순차 실행한다. 실제 준비된 suite 013에서 #6973/#2308의 위치를 확인하고 runner를 고정했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage51/run-focused.sh
```

## 고정 제품 검증 결과

- manifest 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- source-side 정책 통과: 4,205 tests / 298 modules / cfg support 28 유지.
- fmt·native Clippy 통과(exit 0). Clippy 58.44초.
- 집중 nextest **394 passed / 0 failed**, 27 binaries, 필터 비선택 8,450건, exit 0.
  비선택은 기존 ignore와 구분한다. 빌드 7분 17초, 테스트 11.242초.
  run ID: `bae05e1c-1aca-48b5-af00-c8f8b7627ee0`.
- `output/7280/stage51/{compare-focused.mjs,regression-comparison.json}`에서 고정 baseline의
  동일 394개 PASS 이름과 일치함을 확인했다. 전체 회귀 재실행 결과는 아니다.
- 실행 로그: `output/7280/stage51/{prepare,manifest,unit-tier,fmt,clippy-native,nextest-focused}.log`.
  추가 integration 입력 SHA-256은 `fixture.sha256`에 기록하고 실행 후 재확인했다.
  nextest 0.9.137/권장 0.9.140 및 observation 설정 경고는 기존과 동일하다.

검증 후 제품 변경 없음, review worktree tracked 변경 없음, 문서 상대 링크와
`git diff --check` 통과를 확인했다. 파생 suite/manifest·진단 로그는 stage하지 않는다.
이번 절편의 구조 보존·기존 집중 계약은 충족했다. R3 전체 회귀·Native/fresh Docker WASM
시각 비교·최종 제출 게이트는 미실행이며, 한컴 출력과의 신규 시각 일치는 주장하지 않는다.
원격 push·PR·댓글은 수행하지 않았다.

## 다음 절편

저장 프레임 후보의 mirrored rewind 경계 선택·컷 보정 및 확장 예산 조회를 다음 경계로 검토한다.
수용/예약/이월의 상태 쓰기는 부모 소유로 유지한다. 스캔 상태/continuation 본체와 R3 전체
회귀·Native/fresh Docker WASM 시각 비교·최종 제출 게이트가 남아 있다. 원격 작업은 하지 않는다.
