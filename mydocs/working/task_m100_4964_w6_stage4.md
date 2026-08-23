# Task M100 #4964 — Stage W6-4 generator 소유권·metadata

- **수행계획**: [`../plans/task_m100_4964.md`](../plans/task_m100_4964.md)
- **선행 분리**: [`task_m100_4964_w6_stage3.md`](task_m100_4964_w6_stage3.md)
- **기준 source**: `upstream/devel@d1ad0eb8784dbc55f0796e2ba8775f7363247b91`
- **단계 목적**: 생성기가 generated data와 provenance metadata만 소유하게 하고 확인 가능한 공개
  canary에서 결정론과 계보 필드를 검증한다.
- **판정**: 통과

## 1. 발견한 위험과 폐기한 경로

기존 `font-metric-gen --dir` 생성 모드는 디렉터리 파일명을 자동 정렬하고
`(family, bold, italic)` 중복을 글리프 수로 자동 선택한 뒤 `font_metrics_data.rs` 전체 type·lookup과
data를 함께 출력했다. 이 방식은 현재 600행의 물리 순서, first-match와 overlay 경계를 입력자의 의사와
무관하게 바꿀 수 있다.

W6-4는 `--dir`을 진단용 `--list`로만 남겼다. `--dir` 생성은 실패하고, 생성에는 JSON plan과 다음
두 출력이 모두 필요하다.

1. `--generated-output <data.rs>`: `LatinRange`, Hangul table과 `GENERATED_FONT_METRICS`만 포함
2. `--metadata-output <provenance.json>`: source·face·identity·name·license·압축 계보

생성 source에는 type 정의, lookup 함수, alias 또는 overlay가 없다. 따라서 생성기가 runtime 정책을
다시 소유하지 않는다.

## 2. 명시적 입력 계약

plan의 각 입력은 배열 위치와 같은 연속 `order`, checkout 상대 `path`, 명시적 `faceIndex`, 예상
family·bold·italic identity를 가진다. 다음 조건은 출력 파일을 만들기 전에 실패한다.

- order 누락·중복·비연속
- 같은 source/face 반복
- 절대 경로 또는 `..`가 포함된 입력/evidence 경로
- 실제 SFNT identity와 예상 identity 불일치
- `verified` evidence의 source 또는 evidence path 누락
- verified license의 SPDX 누락

자동 sort·dedupe·style 정규화는 없다. TTC도 모든 face를 암묵적으로 순회하지 않고 plan이 지정한
face 하나만 선택한다.

## 3. 출력 ownership

`font_metrics_data.rs`와 `font_metrics_overlays.rs`는 파일명과 canonical 경로 양쪽에서 금지한다.
따라서 다른 이름의 symlink를 통한 우회도 거부한다. 두 출력이 같은 파일로 해석되는 경우도 실패한다.
검사는 파싱·생성 전에 수행되며 ownership 실패 시 metadata를 포함한 어떤 출력도 만들지 않는다.

canonical `font_metrics_generated.rs` 역시 일반 generated 출력과 구분한다. 해당 경로는
`targetRegion: historical-generated-0-594`, `expectedEntryCount: 595`인 전체 plan에만 열린다. 2개
canary plan으로 canonical 파일을 지정한 negative test는 출력 전에 실패하고 595행 원본 hash가
유지됐다.

허용된 Rust 출력은 Stage W6-3에서 분리한 generated fragment 형태다. 그러나 기존 595행의 원본
source/face/order plan은 복원되지 않았으므로 이번 단계는 canonical
`src/renderer/font_metrics_generated.rs`를 재생성하거나 교체하지 않았다.

## 4. provenance metadata

각 생성 항목은 다음을 기계 판독 가능한 JSON으로 남긴다.

| 필드 | 의미 |
| --- | --- |
| `sourceSha256` | 입력 font bytes 전체의 SHA-256 |
| `faceIndex` | TTF는 0, TTC는 plan이 선택한 실제 face index |
| `namingRecords` | SFNT name ID 1·2·3·4·6·16·17·25 중 디코딩 가능한 record와 platform·language |
| `license`, `provenance` | plan 선언과 추적 evidence 파일 SHA-256 |
| `hangulCompression` | 표본 수, 그룹 수, 최대·평균 advance 오차 또는 `not-applicable` |
| `inputPlanSha256` | 입력 순서와 선언 전체를 고정한 plan digest |
| `generatedSourceSha256` | metadata가 가리키는 generated Rust fragment digest |
| `generatorSourceSha256` | 실행 binary에 포함된 generator source digest |

증거 없이 exact 값을 추정하는 필드는 추가하지 않았다.

## 5. 공개 canary 결과

[`font_metric_generator_canary_plan.json`](../tech/investigations/issue-4964/font_metric_generator_canary_plan.json)은
Git에 이미 추적된 두 입력만 사용한다.

| order | source/face | identity | Hangul compression |
| ---: | --- | --- | --- |
| 0 | `NotoSansKR-Regular.ttf#0` | Noto Sans KR Regular | 11,172자, 1×1×1, max 0, avg 0.0 |
| 1 | `RHWPExactFaceSmoke.ttc#1` | RHWP Exact Face One | 표본 없음, `not-applicable` |

source digest는 각각 다음과 같다.

- Noto Sans KR: `6e06a7fe5d696ca719894a23f36bb2b1be8c816a5937cd4ad0f23ca67780dd74`
- 합성 TTC: `f58698f3e60e0aa6e182ecadf4f8239f309a0bac7be252538611e6c5cad723b5`

같은 checkout에서 두 번 생성한 결과는 byte-identical이었다.

- input plan: `eac50d3c6700cdef48ca252c3754b86e5e7ab2eb4f63d7564938f6d025e685e7`
- generator source: `25854b2c8576936a03d7a4b001c0915c2021fee50863e989a7a77e74acc1002d`
- generated source: `e4c8899982ddafc3e964a3b0bab1f632d642133ec87740c81a1e223e53b22239`
- metadata file: `feda0875b6a86dcde56cbd6e5fffae7df53926f2b8ec23053f2306a0e429d4df`

Noto canary는 기존 lineage index 12의 추적 source를 재사용하지만, W6-2가 판정한 기존 항목의
`printable-ascii-only` 검증 범위를 자동으로 넓히지 않는다. 새 generator 출력이 가능하다는 사실과
기존 DB 595행의 역사적 source-exact성은 별개의 주장이다.

## 6. 검증

| 명령 | 결과 |
| --- | --- |
| `cargo check --bin font-metric-gen` | 통과 |
| `node --test scripts/tests/font_metric_gen.test.mjs` | 3/3 통과 |
| 동일 plan 연속 2회 생성 | generated source·metadata SHA-256 동일 |
| core/overlay/canary→canonical 출력 시도 | 3건 모두 종료 코드 1, 원본 hash 불변, 산출물 없음 |
| 폐기된 `--dir` 생성과 비연속 order | 종료 코드 1, 산출물 없음 |
| `cargo fmt --all` | 통과 |
| `cargo fmt --all -- --check` | 통과 |
| `git diff --check` | 통과 |

Cargo 실행이 기존 `Cargo.lock`의 workspace package 두 행을 자동 재정렬했으나 W6와 무관한 파생
diff이므로 기준 checkout bytes로 복원해 변경에 포함하지 않았다.

## 7. Stage 판정과 다음 게이트

W6-I10 결정론과 W6-I12 generator ownership을 공개 TTF·합성 TTC에서 검증했다. 현재 600개 runtime
metric, alias, fallback, generated/overlay source는 이 단계에서 바뀌지 않았다.

다음 Stage W6-5는 분리 이후의 exhaustive lookup·폭·W1/#2430 계약을 다시 묶고, 공개 대표 fixture의
native/WASM parity와 프로젝트 표준 Docker WASM build를 수행하는 최종 통합 게이트다. 메인테이너 승인
전에는 W6-5로 진입하지 않는다.
