# Task #7280 Stage 5 — R2c inline 계획 조회와 상태 반영

- Issue: [#7280](https://github.com/edwardkim/rhwp/issues/7280)
- 구현계획: [task_m100_7280_impl.md](../plans/task_m100_7280_impl.md)
- 이전 절편: [R2b](task_m100_7280_stage4.md), 시작 head `7262bfa7e`.
- 상태: R2c 구현·집중 검증 완료. R2 전체 완료나 제출 준비 완료가 아니다.
- 제품 baseline: `722fb38af361ed3508aef7ca0ac3a8fdc5d3c0db` 유지.

## 1. 책임 분리와 보존 범위

기존 `typeset/inline_flow.rs`의 build closure 계산을 `inline_flow/plan.rs`로 분리한다.
`build_plan`은 읽기 전용 `InlineFlowInput`, 문단, 스타일, 측정 표, dpi만 받는다.
TypesetEngine/TypesetState 또는 가변 페이지 상태를 받지 않으며 기존 renderer inline planner를
재사용한다. 별도 규칙 엔진이나 새로운 줄 나눔 알고리즘을 만들지 않는다.

`state.rs`는 현재 단의 조회, 참조 입력 제공, fit된 계획의 5개 상태 변경을 소유한다.
배제 영역의 복사는 이전과 같이 스타일 조회 성공 후에만 수행한다. 입력 생성은 단 기하와
페이지 크기를 읽고 참조만 보관하며, 계산 중 원래 상태에 대한 쓰기/콜백은 없다.
현재 단의 기하 조회를 사용하는 다른 호출자도 같은 메서드를 사용한다.

상위 inline 흐름 조정자는 기존 eligibility → 계획 → carved 확인 → fit → 필요 시 다음 단
후보 → 단 전진 → 실제 단 재계산 → 확정 순서를 보존한다. 특히 전진 후 재계산 실패 시
이미 전진한 상태로 기존 문단 분할기에 fallback하는 동작도 변경하지 않는다.
이는 기존 동작의 보존이지 해당 fallback의 타당성을 새로 승인한 것이 아니다.

계획의 상대 좌표 변환은 한 번만 수행하고 같은 plan의 end/배치 metadata를 함께 반영한다.
조건, 상수, 연산 순서, public API/IR, 테스트·baseline·golden·ignore는 변경하지 않는다.
상위 `typeset_paragraph` 호출 순서와 이후 높이 보정, renderer 소비자는 그대로다.

## 2. 구조의 한계와 후속 항목

`inline_flow.rs`의 wildcard import를 명시적 의존으로 바꿨다. 계산 모듈에서는 상태 쓰기가
불가능하지만, TypesetState 전체의 필드는 아직 parent 소유이며 조정자는 fit/전진을 위해
이를 조회한다. 다른 메서드의 상태 직접 쓰기까지 차단한 완전한 캡슐화는 아니다.
문단 fit/분할, table paragraph와 다른 컨트롤 흐름의 책임 분리는 R2 후속 절편에 남는다.

## 3. 검증 계획과 증거

원래 build closure, 확정 block, 단 기하 조회의 본문을 복원 비교하고 그 외 호출 순서를 확인한다.
검증 스크립트와 실행 로그는 `output/7280/stage5/`에 둔다.

고정 제품 SHA review worktree에서 suite 준비 후 fmt, manifest/unit-tier 정책 검사,
native Clippy와 기존 inline/어울림/문단 상태 관련 집중 계약을 순차 실행한다.
이전 R2b 전체 10,096 PASS를 이번 제품 SHA의 실행 결과로 재사용하지 않는다.
전체 회귀·WASM/workspace lint·Native Skia·fresh Docker WASM·직접 시각 검증은 최종 통합
게이트에 남는다. 이번에 시각 통과나 PR 제출 준비 완료를 선언하지 않는다.
원격 push·PR·댓글은 수행하지 않는다.

### 고정 제품과 검사 진행

- 제품 SHA: `c774746de7e25cd348cc6c70d26e7aeb10958f93`.
- review worktree: `/home/edward/mygithub/rhwp-review-7280-r2c`.
- 구현 커밋 `2b9ad517b`에서 fmt가 하위 plan 모듈 경로를 해석하지 못했다.
  기존 부모의 `#[path]` 선언을 유지하고 하위 모듈도 명시 경로로 지정한 `c774746de`에서 재검사했다.
  최초 오류는 `fmt-initial-path-error.log`에 보존한다. 테스트 실패로 집계하지 않는다.
- `verify-inline-flow.mjs` / `extraction-proof.json`: 계산 입력 매핑, 원래 closure/확정 block 복원,
  단 조회 본문 및 나머지 상위 코드 동일성 통과. 정적 보존 증거이며 직접 시각 검증은 아니다.
- manifest 고정 baseline 검사 통과: 1,382 sources / 5,965 static attrs / 48 integration targets.
- unit-tier 고정 baseline 검사 통과: 4,205 tests / 298 modules / ready 0 / support 87 /
  white-box 4,114 / cfg support items 28.
- `cargo fmt --all -- --check` 통과.
- native `cargo clippy --locked --target-dir /home/edward/mygithub/rhwp/target/pr-review -- -D warnings`
  통과(exit 0, 55.34초). `CARGO_BUILD_JOBS=4` 사용.

집중 테스트는 기존 `renderer::typeset`, `renderer::composer`, `renderer::float_placement`와
`issue_6812` 계약을 선택한다. 후자는 그림 옆 공간, 그림 하단 회피, inline 앞뒤 텍스트 순서,
이전 문단 그림의 영향, 새 페이지에서 앞 페이지 배제 영역 해제를 검사한다.
그림 경계를 넘는 배치와 실제 paint 기하를 검사하는 기존 계약을 유지하며 기대값을 바꾸지 않는다.
전진 후 재계산 실패의 모든 조건 조합을 별도로 실행했다고 주장하지 않는다.

```bash
CARGO_BUILD_JOBS=4 cargo nextest run --locked --cargo-profile release-test \
  --target-dir /home/edward/mygithub/rhwp/target/pr-review \
  --lib --test regression_suite_027 \
  -E 'test(renderer::typeset::) | test(renderer::composer::) | test(renderer::float_placement::) | test(issue_6812)' \
  --no-fail-fast
```

실행 결과: **181건 통과 / 실패 0건**, 필터 비선택 4,102건, 5 binaries, exit 0.
빌드 4분 42초, 테스트 0.410초. 실행 ID: `662d7393-25aa-4d3e-8ee0-844d59d0d6ff`.
로그: `output/7280/stage5/nextest-focused.log`.
`compare-focused.mjs` / `regression-comparison.json`으로 고정 baseline 전수 결과에서
동일 필터로 선택한 181개 PASS 이름과 일치함을 확인했다. 이번 필터 비선택 건수는
기존 회귀 ignore 50건과 다른 수치다. 전체 회귀를 다시 실행한 것으로 집계하지 않는다.

nextest 0.9.137(프로젝트 권장 0.9.140)과 미사용 observation profile 설정 경고는
기준 실행과 동일하다. 현재 CI 도구 버전까지 동일한 검증은 아니다.
review worktree tracked 변경은 없고 generated suite/manifest는 stage하지 않았다.
제품 검증 뒤에는 이 완료 기록만 수정했다.

## 4. 다음 절편

R2의 문단 fit/분할과 table paragraph의 흐름 조정 책임을 이어서 분리한다.
계획 계산의 읽기 경계가 생긴 것과 TypesetState 전체 쓰기 권한의 캡슐화는 구분한다.
실제 조판 변화가 나타나면 이슈 수정으로 섞지 않고 해당 구조 이동부터 검토한다.
