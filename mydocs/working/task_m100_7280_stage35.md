# Task #7280 Stage 35 — R2af 표 없는 host의 어울림 영역 준비 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 점검: [Stage34](task_m100_7280_stage34.md), 시작 head `c1f1b8255`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: R2af 구현·고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 책임과 동작 보존

`typeset_no_table_paragraph_tail`을 기존 시그니처의 얇은 진입점으로 유지한다.
구역 내 호출 위치·계측·미사용 인자도 변경하지 않는다.

- `controls/host_wrap.rs`: Picture/Shape의 저장 영역 후보, 전폭 여부, 해당 host의 anchor 조회.
  원본 IR과 PageDef만 읽는다. state·엔진 전체·가변 참조를 받지 않는다.
- `controls::prepare_no_table_host_wrap`: 조회와 적용 순서 조정.
- `state.rs`: 단 너비·비활성 여부 조회, 저장 영역 적용·anchor 등록·유도 영역 적용.
  의미 있는 명령으로 기존 필드 쓰기 순서를 보존하며 범용 setter를 만들지 않는다.
- 기존 `square_float_left_lane_width` 계산과 모든 호출 외 코드, 후속 문단 어울림·페이지 전이·
  미주 흐름은 변경하지 않는다. 기존 root Query 의존은 후속 구조 작업에 남는다.

보존할 계약:

1. `has_table`이면 새 조회·쓰기 없이 종료한다.
2. 비-TAC Square Picture 후보가 있는 경우에만 저장 cs/sw를 읽고 단 너비를 조회한다.
   전폭 판별의 `< 3000`, 양수 조건, 기본값을 그대로 둔다.
3. 저장 밴드의 다섯 상태 필드를 먼저 적용한 뒤 caption-style/오른쪽 여백으로 host anchor를 판단한다.
   후보 계산 전에 미리 기존 상태를 비우거나 anchor를 삭제하지 않는다.
4. 원래 저장 영역 적용 뒤의 `wrap_around_cs < 0`일 때만 단 너비를 다시 조회하고 유도 lane을 계산한다.
   저장 후보가 없어도 기존 영역이 활성화돼 있으면 보존한다.
5. 저장 밴드는 `any_seg=true, derived=false`, 유도 밴드는 `any_seg=false, derived=true`를 유지한다.
   유도 lane 실패도 기존 상태를 지우지 않으며 host anchor를 새로 등록하지 않는다.

조판 규칙의 타당성을 새로 판단하거나 호환 상수·조건을 개선하지 않는다.
Rust 테스트/assertion·baseline/golden/ignore·IR/public API·CI 변경은 없다.

## 2. 검증 계획

`output/7280/stage35/verify-host-wrap.mjs`가 Query와 Command를 원래 식으로 펼쳐 전체 본체,
조정 순서와 부모 나머지·기존 controls/state 불변을 대조한다. import 별칭만 정규화한다.
정적 대조가 모든 분기 실행이나 직접 시각 판정을 의미하지는 않는다.

고정 review worktree에서 suite 준비 → baseline 대비 manifest/unit-tier → fmt → native Clippy →
집중 nextest 순으로 검증한다. 기존 313건에 Stage34 어울림 10건을 더한 323건을 선택한다.
일부 그림 지연·각주 계약은 기존 선택에 있으나 caption/미주/다단의 모든 경로 커버리지를 주장하지 않는다.
새 fixture나 기준값 변경은 하지 않는다.

전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM·직접 시각 대조는
구현계획 §7의 책임 묶음/제출 전 통합 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 3. 고정 head 검증

- 제품 SHA: `fa836740c49deb30e8db2a05778641f30b7d90fa`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2af`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage35/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support 28 통과. source-side test 변경은 없다.
- fmt 통과, native Clippy `-D warnings` 통과(exit 0, 55.01초).
- 로그: `output/7280/stage35/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 고정 review worktree에서 아래 명령을 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage35/run-focused.sh
```

정확한 집중 명령과 필터는 `run-focused.sh`에 보존했다. Stage33의 313건 선택에
Stage34의 어울림 6개 module / 10건을 추가했다. 테스트 원본·기준값은 변경하지 않았다.

결과: **323 passed / 0 failed**, 24 binaries, 필터 비선택 7,886건, exit 0.
빌드 6분 42초, 테스트 2.782초. run ID: `7ae61599-1490-4e73-9dcf-c48205d2ef67`.
`nextest-focused.log`, `compare-focused.mjs`, `regression-comparison.json`에 증적을 보존했다.
baseline 전수 로그에서 동일 필터로 고른 323개 PASS 이름과 일치한다.
비선택 수는 기존 ignore 50건과 별개다. 전체 회귀 재실행·출력 픽셀 동일성·한컴 시각 일치를
뜻하지 않는다. nextest 0.9.137 권장 버전 및 observation 설정 경고는 기존과 동일하다.

review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 후 제품 코드를 변경하지 않았으며 구현계획과 결과 기록만 갱신했다.

## 4. 후속

후속 문단의 어울림 매칭·흡수와 prefix/tail 배치 경계를 분리한다.
표 분할 본체·미주 연결·구역 반복문·페이지 수명은 R3/R4/R5 경계를 유지한다.
