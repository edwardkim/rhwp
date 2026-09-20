# Task #7280 Stage 36 — R2ag 후속 어울림 문단 매칭·그림 anchor 조회 분리

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2af](task_m100_7280_stage35.md), 시작 head `50ac14a22`.
- 고정 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db`.
- 상태: Query 분리와 고정 제품 SHA 집중 검증 완료. R2 전체/PR 준비 완료가 아니다.

## 1. 이번 분리와 후속 경계

`typeset_wrap_around_paragraph` 앞부분의 읽기 전용 판별을 `controls/wrap_match.rs`로 분리한다.
입력 `WrapBand`는 활성 여부 확인 후 상태 쓰기가 없는 구간의 cs/sw·앵커 문단·any_seg만 담는다.
원본 문단/목록/PageDef는 차용한다. Query는 state·엔진 전체나 가변 참조를 받지 않는다.

- `classify`: 원래 순서의 cs/sw·빈 문단·저장 줄·그림 위치·cs-only 매칭과 결과 반환.
- `anchor_is_picture`: 매칭 성공 뒤에만 앵커 종류 조회.
- `picture_anchor`: 그림 또는 유도 밴드 분기에서만 기존 오른쪽 여백 조회 후 WrapAnchorRef 반환.
- state의 `following_wrap_band`는 읽기만, `register_following_wrap_anchor`는 기존 map insert만 수행한다.

원래 진입 guard(`cs >= 0 && !has_table`)와 `derived_band` 분기 위치를 유지한다.
snapshot 생성에서 plain 필드를 미리 복사하지만 이 구간에는 상태 쓰기나 외부 호출이 없다.
본문 `paragraphs.get` 의미를 보존하고 미주의 전역 인덱스 처리와 합치지 않는다.
매칭 실패의 밴드 종료/쪽 전환, 표 옆 흡수·소급 기록, prefix/tail 포맷·높이 계산·배치는
원래 본체에 그대로 남긴다. 이 구간은 상태 변경 뒤의 재조회가 있어 같은 snapshot으로 옮기지 않는다.
반환 true/false와 호출자의 continue도 변경하지 않는다.

이번 절편은 후속 어울림 전체 분리를 완료한 것이 아니라 그 앞쪽 Query 경계를 분리한 것이다.
수치·비교·조판 정책·IR/public API·Rust 테스트/assertion·baseline/golden/ignore·CI 변경은 없다.

## 2. 검증 계획

`output/7280/stage36/verify-wrap-match.mjs`는 조회를 원래 식으로 펼쳐 전체 함수 본체와 대조한다.
state getter/insert와 부모 나머지·기존 controls/state 불변도 확인한다.
정적 대조는 실행 분기 커버리지나 직접 시각 판정이 아니다.

고정 review worktree에서 suite 준비 → 고정 baseline 대비 manifest/unit-tier → fmt →
native Clippy → Stage35와 같은 323건 집중 nextest를 순차 실행한다.
기존 원본·변형 계약을 재사용하며 모든 caption/미주/다단 분기 검증을 주장하지 않는다.
전체 회귀·WASM/workspace lint·workspace build·Native Skia·fresh Docker WASM·직접 시각 대조는
구현계획 §7의 통합 게이트에 남긴다. 원격 push·PR·댓글은 범위 밖이다.

## 3. 고정 head 검증

- 제품 SHA: `646a5c4b80840cce6a41a0d60500852fa7820081`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2ag`.
- target: `/home/edward/mygithub/rhwp/target/pr-review`, `CARGO_BUILD_JOBS=4`.
- 정적 대조 통과: `output/7280/stage36/extraction-proof.json`.
- manifest: 1,382 sources / 5,965 static attrs / 48 targets 통과.
- unit-tier: 4,205 tests / 298 modules / ready 0 / support 87 / white-box 4,114 /
  cfg support 28 통과. source-side test 변경은 없다.
- fmt 통과, native Clippy `-D warnings` 통과(exit 0, 55.93초).
- 로그: `output/7280/stage36/{prepare,manifest,unit-tier,fmt,clippy-native}.log`.

위 고정 review worktree에서 아래 명령을 순차 실행했다.

```bash
node scripts/rust-test-suite-manifest.mjs --prepare
node scripts/rust-test-suite-manifest.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
node scripts/rust-unit-test-tiers.mjs --check --base-ref 722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db
cargo fmt --all -- --check
CARGO_BUILD_JOBS=4 cargo clippy --locked \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings
bash /home/edward/mygithub/rhwp/output/7280/stage36/run-focused.sh
```

정확한 집중 명령과 필터는 `run-focused.sh`에 보존했다. Stage35와 동일한 선택이며
테스트 원본·assertion·기준값은 변경하지 않았다.

결과: **323 passed / 0 failed**, 24 binaries, 필터 비선택 7,886건, exit 0.
빌드 6분 41초, 테스트 2.809초. run ID: `c41650b8-12b3-41c9-b0ec-07cf7bc86d10`.
`nextest-focused.log`, `compare-focused.mjs`, `regression-comparison.json`에 증적을 보존했다.
baseline 전수 로그에서 동일 필터로 고른 323개 PASS 이름과 일치한다.
비선택 수는 기존 ignore 50건과 별개다. 전체 회귀 재실행·출력 픽셀 동일성·한컴 시각 일치를
뜻하지 않는다. nextest 0.9.137 권장 버전 및 observation 설정 경고는 기존과 동일하다.

review worktree의 tracked 변경은 없고 파생 suite/manifest는 커밋하지 않았다.
검증 후 제품 코드를 변경하지 않았으며 구현계획과 결과 기록만 갱신했다.

## 4. 후속

표 anchor의 전체 문단 흡수·소급 기록과 prefix/tail 배치의 조회/명령 경계를 계속 분리한다.
밴드 종료가 높이를 바꾸는 점과 그 뒤 쪽 전환 재판정 순서를 보존한다.
