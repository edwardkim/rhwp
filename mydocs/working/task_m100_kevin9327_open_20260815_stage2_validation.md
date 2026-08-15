# Task M100 Kevin9327 공개 PR 통합 검토 Stage 2 - 전체 회귀 검증 계획

## 목적

Stage 1의 최신 head 정합과 유지보수 보정 커밋을 기준으로, #4818부터 #4878까지 누적한
통합본이 기본 기능 회귀 없이 함께 동작하는지 확인한다.

## 실행 계획

1. 고정 재사용 target인 `target/pr-review`에서 전체 Rust 테스트를 `nextest`로 실행한다.
2. 실행 중 또는 직후 열린 Kevin PR의 head SHA를 다시 조회해 검증 대상이 최신 상태인지 확인한다.
3. 결과는 이 문서에 추가하고, 개별 PR 검토 기록과 통합 PR 준비 여부를 다음 단계에서 결정한다.

## 실행 명령

```bash
cargo nextest run --cargo-profile release-test --target-dir target/pr-review \
  --tests --test-threads 12 --no-fail-fast
```

## 실행 결과

### 최초 전체 실행에서 발견한 정합 누락

2026-08-16에 위 명령을 처음 실행했을 때 6,340개 중 2개가 실패했다.

1. `agent_profile_router_contract::every_stateless_tool_belongs_to_some_specific_profile`
   - `hwp_threat_scan`이 어느 업무 프로필에도 속하지 않았다.
2. `knowledge_map_field_dictionary_contract::every_declared_record_field_is_in_the_dictionary`
   - `threat-scan`이 선언하는 `highestSeverity`와 `notes`가 지식지도 §2-2 전수 사전에 없었다.

### 유지보수 보정

- `아카이브검색` 프로필에 `hwp_threat_scan`을 추가하고, 출처가 불분명한 문서는 파싱 전에
  컨테이너·레코드 위협 신호를 확인하도록 레시피 순서를 명시했다.
- 지식지도에 `threat-scan`의 호출 용도, MCP 도구 매핑, `highestSeverity`·`notes` 필드와
  `scanScopes` 의미를 추가했다.
- §2-2 전수 사전 수를 `recordFields` 269개와 실측 전용 3개, 합계 272개로 갱신했다.

### 보정 직후 검증

```text
cargo test --profile release-test --target-dir target/pr-review --test agent_profile_router_contract every_stateless_tool_belongs_to_some_specific_profile -- --exact --nocapture
1 passed

cargo test --profile release-test --target-dir target/pr-review --test knowledge_map_field_dictionary_contract every_declared_record_field_is_in_the_dictionary -- --exact --nocapture
1 passed

RHWP_BIN=target/pr-review/release-test/rhwp python3 tools/gen_agent_codex.py --check
명령 89 · 실측 표본 19 · 계약만 70 · 변경 0

cargo clippy --all-targets --target-dir target/pr-review -- -D warnings
통과
```

### 전체 회귀 재실행

```text
cargo nextest run --cargo-profile release-test --target-dir target/pr-review --tests --test-threads 12 --no-fail-fast
Summary [347.177s] 6340 tests run: 6340 passed (7 slow), 38 skipped
```

최초 실패의 원인이었던 프로필·자기서술 문서 정합은 보정 후 전체 Rust 회귀에서 재현되지 않았다.
