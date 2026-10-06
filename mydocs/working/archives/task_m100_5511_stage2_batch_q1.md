# #5511 Stage 2 기능군 배치 Q1 — thumbnail preview output 경계

- 이슈: [#5511](https://github.com/edwardkim/rhwp/issues/5511)
- 브랜치: `task_m100_5511`
- 통합 기준선: `1a6ce79fd56e3cdf5813c7938338fcb5b7d0a859`
- 계약 커밋: `02ebb6d3e`
- 구현 커밋: `171cb75c0`
- 수행일: 2026-08-19
- 상태: 완료 — Q2 승인 대기

## 1. 배치 범위와 CQRS 판정

`thumbnail`은 문서의 내장 미리보기를 읽지만 기본 모드에서 새 파일을 쓰고, 선택에 따라
base64 또는 data URI를 stdout으로 내보낸다. 따라서 부작용 없는 query adapter가 아니라
**preview output adapter**가 소유하는 것이 맞다.

이 판정에 따라 root의 `extract_thumbnail` handler를 `src/cli/outputs/preview.rs`로 물리 이동하고
`src/cli/outputs/mod.rs`를 출력 adapter 진입점으로 추가했다. top-level dispatch만 새 경로를
호출하도록 바꿨다. 인자 해석, preview 추출, MIME·확장자 결정, 파일 저장, JSON·사람용 출력과
종료 코드는 변경하지 않았다.

## 2. 선행 characterization 계약

기존 `tests/issue_3366_thumbnail_contract.rs`에 다음 네 계약을 추가하고 제품 코드 이동 전에
독립 커밋했다.

1. 파일 모드의 출력 바이트는 `rhwp::parser::extract_thumbnail_only` 결과와 정확히 같고 JSON
   metadata가 일치하며 입력 문서 바이트는 변하지 않는다.
2. base64와 data URI는 모두 내장 preview 원본 바이트로 정확히 decode되고 파일을 만들지 않는다.
3. 기본 출력 경로는 현재 작업 디렉터리 기준 `output/<stem>_thumb.<format>`이며 바이트가 같다.
4. 부모 경로가 일반 파일인 저장 불가 target은 종료 코드 1, 빈 stdout, 저장 실패 stderr를
   내고 기존 blocker를 바꾸지 않는다.

직접 thumbnail 계약은 7개에서 11개로 늘었다. 이동 전 관련 regression suite 두 개를 함께
실행해 228/228가 6.142초에 통과했다.

## 3. 이동과 구조 변화

구현 커밋은 새 output 경계에 handler 본문을 옮긴 move-only 변경이다. 새 wrapper, 상태 객체,
helper 복제나 parser·renderer·serializer 알고리즘 변경은 없다. 모듈 경계 호출에 필요한
`pub(crate)` 가시성과 import만 추가했다.

| 항목 | Q1 전 | Q1 후 | 변화 |
|---|---:|---:|---:|
| `src/main.rs` 줄 수 | 38,561 | 38,405 | -156 |
| `src/cli/outputs/mod.rs` | 0 | 5 | +5 |
| `src/cli/outputs/preview.rs` | 0 | 165 | +165 |
| Rust test source | 754 | 754 | 변화 없음 |
| static test attribute | 3,721 | 3,725 | +4 |
| 직접 thumbnail 계약 | 7 | 11 | +4 |

이동 후 같은 focused 범위 228/228가 6.060초에 통과했다. 새 모듈은 165줄로 1,200줄 상한보다
작고, root와 새 모듈 사이 양방향 참조나 기능군 helper 복제가 생기지 않았다.

## 4. 배치 최종 검증

| 검증 | 결과 |
|---|---|
| 이동 전 관련 regression suite | 228/228 통과, 6.142초 |
| 이동 후 관련 regression suite | 228/228 통과, 6.060초 |
| release-test 전체 nextest | 7,774/7,774 통과, 3 slow, 38 skipped, 160.596초 |
| `cargo check --all-targets` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `cargo clippy --all-targets --target-dir target/pr-review -- -D warnings` | 통과 |
| `cargo test --doc --target-dir target/pr-review` | 8/8 통과, 3 ignored |
| integration manifest 정책 자체 계약 | 16/16 통과 |
| prepare 후 `rust-test-suite-manifest --check --base-ref upstream/devel` | 통과, 754 sources / 3,725 static test attrs / 43 integration targets |
| unit-tier 정책 자체 계약 | 12/12 통과 |
| `rust-unit-test-tiers --check --base-ref upstream/devel` | 통과, 4,225 tests / 298 modules |
| CI impact Node 계약 | classifier+policy 62/62 통과 |
| `scripts/tests/test_ci_impact_workflow.py` | 30/30 통과 |
| `git diff --check` | 통과 |
| `check_markdown_links.py --changed-from upstream/devel` | 기존 capability 등록부 무결성 오류 16건, Q1 신규 오류 없음 |

검증 준비가 생성한 로컬 Cargo test target 두 개와 Cargo가 바꾼 verifier package의 lockfile
순서는 추적 변경에서 복원했다. 파생 target을 정리한 상태에서 `--prepare` 없이 manifest
`--check`만 실행하면 drift를 보고하는 것은 원본-only 제출 정책에 따른 예상 동작이다.

호스트 nextest는 `0.9.137`이고 저장소 권장은 `0.9.140`이라 경고가 있었지만 실행 결과에는
영향이 없었다. parser·renderer·serializer와 WASM 경계를 바꾸지 않은 move-only 변경이고,
추출 바이트 동등성을 계약으로 직접 검증했으므로 시각 검증과 WASM 빌드는 추가하지 않았다.

## 5. 원격 병합 위험 재검증

Q1 시작 시 `origin/devel`과 `upstream/devel`은 `1a6ce79fd`로 같았다. 최종 검증 뒤 원격을 다시
fetch하자 #5548이 병합되어 두 devel 모두 `73811a7bc`로 전진했다. 새 원격 변경은 equation
parser, 전용 test와 review/order 문서에 한정되어 Q1의 root·CLI output·thumbnail test 경로와
겹치지 않는다.

현재 구현 HEAD와 `73811a7bc`의 merge-tree는 충돌 없이 생성됐다. 열린 PR 14개의 변경 경로에도
Q1 대상 경로와 교집합이 없다. #5548 병합 및 새 PR #5567·#5569·#5570을 반영한 판정이다.
따라서 이 배치 안에서 rebase나 merge를 만들지 않았다. remote push 또는 PR 생성 직전에는
최신 devel과 열린 PR head를 다시 fetch해 exact SHA 기준으로 재검증해야 한다.

## 6. 다음 배치 관문

다음 승인 단위는 Q2 `SVG·render-tree·structure·PNG·GPU·PDF` 출력 기능군이다. renderer 자체를
고치는 배치가 아니라 기존 output adapter를 물리 분리하는 배치이며, 시작 전에 다음을 먼저
확정한다.

- 각 handler의 stdout·파일·JSON·종료 코드 계약과 시각 결과 보호 범위를 inventory한다.
- vector, raster/GPU, PDF가 하나의 1,200줄 초과 모듈이 되지 않도록 하위 책임을 나눈다.
- renderer 알고리즘 변경 필요, 시각 차이 또는 열린 PR 경로 중첩이 발견되면 이동을 중단한다.

Q2는 메인테이너의 배치 종료 승인과 다음 배치 진입 승인 전 시작하지 않는다. remote push도
별도 승인 전 수행하지 않는다.
