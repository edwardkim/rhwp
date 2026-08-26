---
kind: report
status: completed
canonical: mydocs/plans/task_m100_4966.md
last_verified: 2026-08-23
---

# Task M100 #4966 — Stage W7-6 통합 검증·운영 인계

## 1. 판정

Stage W7-6의 제품·생성·렌더링 검증은 통과했다. canonical registry의 830개 semantic rule과 다섯
projection hash는 유지됐고, Rust·Studio consumer와 native/Docker WASM의 출력 차이는 발생하지 않았다.

통합 검사에서 새 Rust generated source가 schema version `"1.0"`을 두 파일에 직접 정의해 #4329의
스키마 단일 출처 계약을 위반한 사실을 발견했다. `src/schema_registry.rs`에
`FONT_RULE_PROJECTION_SCHEMA_VERSION`을 한 번 정의하고 generator가 이를 참조하게 정정했다. generated
file content와 generator hash는 바뀌었지만 registry 입력과 다섯 semantic projection hash는 바뀌지
않았다.

## 2. 검증 환경과 준비

- W7-6 정정 전 task head: `f0057d419`
- W7-6 정정 commit: `855fbf738`
- 최종 통합 기준: `upstream/devel@343ed2c013606319b6418dd8c637c5e04047e304`
- 최신 devel 통합 검증 head: `bb8482263`
- CPU 16개, RAM 31 GiB, Docker Server 29.7.2
- cargo-nextest 0.9.137; 저장소 권고 0.9.140 경고는 비차단
- review 전용 detached worktree에서 integration suite 890 source·4,156 static test attribute,
  32 suite + 9 exception을 준비했다. generated suite와 manifest는 source diff에 포함하지 않았다.
- W7-6 정정 자체는 `Cargo.lock`을 바꾸지 않았다. 이후 최신 devel 병합이 workspace package 정렬만
  가져와 최종 SHA-256은 `1eeaa945d41544d5c6172acb95c62a5b0d799dbc995642b46f01e5ad5919cec5`다.
  병합 뒤 전체 gate를 새 integration source로 다시 실행했다.

## 3. 최초 실패와 정정

첫 full nextest는 8,174건 중 8,173건이 통과했고 다음 한 건만 실패했다.

```text
schema_registry_contract::no_version_literals_outside_registry
src/renderer/font_rule_projections/layout_metric.rs: "1.0"
src/renderer/font_rule_projections/layout_name.rs: "1.0"
```

원인은 생성기가 각 Rust projection에 `*_SCHEMA_VERSION: &str = "1.0"`을 복제한 것이었다. 생성
산출물을 수동 편집하지 않고 generator template과 중앙 schema registry를 고쳤다. focused 계약을 다시
통과한 뒤 전체 nextest를 처음부터 재실행해 8,174/8,174를 확인했다.

### 3.1 PR-base unit-tier 실패와 완료 판정 철회

PR #5950 최초 head `4a7c0f431`의 CI lint는 `font_metrics_data.rs` 2개와 `style_resolver.rs` 4개를
PR base에 없던 `#[cfg(test)]` support item으로 거부했다. 로컬에서는 base 인자 없는 inventory check만
실행해 현재 정책 수치만 확인했고 PR-base 증가를 놓쳤다.

기존 hand-written mapping 함수의 `#[cfg(test)]`를 제거해 runtime scope로 되돌리는 최초 대응은 exact
base check를 통과했지만, canonical registry 이관 뒤에도 중복 수기 표를 제품 source에 남기는 우회였다.
메인테이너 지적 뒤 해당 미커밋 source 변경을 철회하고 완료 판정을 취소했다.

추가 재감사에서 unit-tier는 정책대로 신규 support 6개를 정확히 탐지한 것으로 판정했다. 별도로 W3의
current-source 의미 유지 계약 10건 중 1건이 실패했다. 600개 metric candidate의 ID·metric tuple은
유지됐지만 `sourceLocation.selector`가 W6의 고정 배열에서 composed view로 이동한 것을 의미 회귀로 센
계측 오판이다. W7 검증 목록이 W3 계약을 포함하지 않았기 때문에 Stage W7-6의 77/77만으로 이 누락을
발견하지 못했다. 근본 정정은 [Stage W7-R 계획](../plans/task_m100_4966.md#stage-w7-r--pr-ci-실패-원인-귀속과-소유권-이전-완결)으로
분리한다. 후속 구현과 focused 검증 결과는 [Stage W7-R2·R3 기록](task_m100_4966_w7_rework_stage2.md)에
있다.

## 4. 검증 결과

### 4.1 schema·계보·생성

| gate | 결과 |
| --- | --- |
| registry / projection / pre-migration baseline check | 통과 |
| W6 metric baseline·lineage manifest check | 통과 |
| W1·W2·W6·W7 Node contract | 77/77 |
| Rust unit-tier inventory | 4,224 tests / 299 modules, drift 없음 |
| projection generator | 830 rules / 5 outputs, deterministic check 통과 |

### 4.2 Rust

| gate | 결과 |
| --- | --- |
| `cargo build --locked --release` | 통과, 최신 통합 native build 9분 6초 |
| release library | 4,074 pass / 13 ignore |
| release-test nextest | 8,201/8,201 pass, 41 skip, slow 4 |
| native-skia library | 통과 |
| missing picture placeholder | 2/2 |
| direct PDF export | 4/4 |
| Clippy `--all-targets -- -D warnings` | 통과 |
| rustdoc | 8 pass / 3 ignore |
| `cargo fmt --all -- --check` | 통과 |

release library 수량은 root 3,892, contracts 15, OOXML chart 165, password crypto 2의 합이다.
최신 devel 병합 전 8,174/8,174도 통과했으며, 병합 뒤 늘어난 integration source를 준비해
8,201/8,201을 처음부터 다시 실행했다. 장시간 security corpus·IR field sweep·HWP5 baseline도
전부 통과했다.

### 4.3 Studio·WASM·backend

| gate | 결과 |
| --- | --- |
| `npx tsc --noEmit` | 통과 |
| Studio 전체 Node test | 1,070 pass / 1 skip |
| Studio production build | 통과, 223 modules |
| Canvas2D·CanvasKit focused | 38/38 |
| Docker optimized WASM | 통과, 최신 통합 head 5분 43초 |
| fresh WASM Decision Trace E2E | 3/3 |

focused 검사는 CSS glyph face 비관찰, local permission·probe, CanvasKit SFNT source record와 TTC face,
KoPub SFNT, 정부상징 successor, unavailable surface의 fail-closed와 generated `ruleId` 존재를 함께
확인했다. Canvas2D supply를 CanvasKit load 성공으로 승격하지 않았다.

### 4.4 native/WASM SVG parity

동일 source에서 다시 만든 `target/release/rhwp`와 Docker `pkg`를 사용했다.

| 묶음 | 문서 | 비교 페이지 | mismatch |
| --- | ---: | ---: | ---: |
| W1 공개 HWP | 7 | 167 | 0 |
| W2 trace 대표 HWP/HWPX | 6 | 6 | 0 |

private corpus·host 절대 경로·로컬 font bytes는 이 검증이나 보고서에 사용하지 않았다.

## 5. 생성 hash

| 항목 | SHA-256 |
| --- | --- |
| registry rules | `34838af25531327b9e697b065ed5771a11f310c970a9923c83a0b6e1235a68bd` |
| generator | `db6cc28ab3b21ea2aff40e352f6601b388207a6b8da767977e044b170b011373` |
| semantic projection bundle | `533c1ea77d70658be513b62bd77fb631c5099703bef4c4fdfb8629fd477c1ac8` |
| generated content bundle | `497a249d2e363d7289fe96df40e74780f857f807a7900de61c4fe8e416b4269e` |
| Rust layout-name | `595cdcc1c8d81441c9e4585acb393e734f52e6da3e822babf0f722df2c791cee` |
| Rust layout-metric | `c4659fc40246c5d4ad903578a61807c646681638cb4c8f9b7c802fb3f0c37cc2` |
| Canvas2D paint | `c959e68087f6928edcafc74a1d3f9cd3885dd7540faf22b7663a49b6ad8835e4` |
| Canvas2D webfont | `730cab042d68ffb019d5867102ee8b2b8e5be41c48170ca5fc75422005e3fbee` |
| CanvasKit SFNT | `d9019fc756d4fd9334252704309bb2020c251d6a7d04dc0f5a6b2efb0f017668` |

중앙 schema constant 정정 전후 semantic projection bundle과 backend별 projection hash는 같다.
generator와 Rust file bytes가 바뀌어 content bundle만 의도적으로 갱신됐다.

## 6. 운영 인계 판정

registry schema 1.0은 830개 active migration population을 봉인한다. `check`도 동결 W1/W6/W7 입력에서
registry를 다시 만들기 때문에 현 판에서 JSON 직접 추가·수정·삭제는 지원하지 않는다. 이를 일반 CRUD가
가능한 것처럼 기록하지 않았다.

생성기·직렬화 정정은 registry check → projection generate/check → baseline check → focused/full gate
순서로 처리한다. 실제 mapping 추가·수정·폐기는 별도 이슈에서 다음 schema 판을 먼저 승인해야 한다.
폐기는 행 삭제가 아니라 evidence와 successor를 가진 `retired` 상태로 설계한다. W8의 첫 mapping 보정은
이 변경 가능한 registry 판을 마련하기 전에는 시작하지 않는다.

## 7. 잔여 절차

Stage W7-6 구현·문서 diff와 최신 devel 통합 검증 기록을 로컬 commit했고 메인테이너가 최종 결과를
승인했다. remote push, PR 생성, self-review, merge와 issue close는 각각 별도 승인 게이트다.
