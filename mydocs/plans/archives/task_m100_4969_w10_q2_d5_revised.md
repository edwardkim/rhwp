# 수정 수행계획 — Task M100 #4969 W10-Q2-D5 resource-qualified NO_LS activation

- **선행 결과**: Q2-D4 `qualified-bounded`, merge `1a43a507c`
- **resource 감사 checkpoint**: `ae36a8786`
- **resource 감사**:
  [`task_m100_4969_w10_q2_d5_resource_audit.md`](../../working/archives/task_m100_4969_w10_q2_d5_resource_audit.md)
- **기계 판독 계획**:
  [`w10_q2_d5_revised_execution_plan.json`](../../tech/investigations/issue-4969/w10_q2_d5_revised_execution_plan.json)
- **상태**: R0 `qualified-red`, R1·R2·R3·N0·N1 `qualified`, N2 `qualified-bounded`; checkpoint
  `422a8f7bc`·최신 devel 병합 재자격화·증적 checkpoint `167e3b3d2` 완료, Q2 최종 `bounded-subset` 승인
- **제품 변경**: R0·R1은 0, R2는 opt-in transport에서만 있음, N0은 0, N1부터 승인된 NO_LS lane에 있음

## 1. 수정 이유

D4는 단일 줄·단일 run common GlyphRun replay를 `qualified-bounded`로 끝냈다. 그러나 같은 exact font를 사용하는
run마다 font 전체 digest·face 준비를 반복하고, page별 layer JSON은 456,688B portable font payload를 다시 inline
전달한다. 이 상태로 기존 D5의 NO_LS boundary-changing lane을 먼저 열면 지원 범위와 함께 CPU·전송량도
run·page 수에 비례해 늘어날 수 있다.

따라서 D5를 resource 선행 절편과 boundary activation 절편으로 나눈다. 먼저 기존 page-local dedup과
`Arc<[u8]>` 공유를 보존하면서 unique exact source 준비와 document 전달 단위 재사용을 증명한다. 이 결과가
qualified일 때만 line selection·bbox·next origin·GlyphRun을 같은 shaping `Arc`로 원자 게시하는 NO_LS lane을 연다.

## 2. 보호할 현재 사실

1. `ExactFontSourceRegistry`는 handle별 font bytes를 `Arc<[u8]>` 한 개로 보존한다.
2. replay certificate는 그 `Arc`를 공유하며 host font 이름이나 경로를 재조회하지 않는다.
3. 한 page의 `ResourceArena`는 동일 font blob과 face를 각각 한 번만 등록한다.
4. reject는 resource mutation 전에 끝나고 TextRun fallback은 항상 남는다.
5. CanvasKit은 digest·length·resource key 검증 뒤 blob·typeface·font를 document renderer 수명 동안 재사용한다.
6. 현행 layer JSON은 font bytes inline이 기본 계약이다. 새 전달 경로도 기본값을 바꾸지 않는다.
7. D4의 단일 줄·단일 run 지원 lane과 non-target 출력은 resource 절편에서 넓히거나 바꾸지 않는다.

## 3. 수정된 owner 구조

```text
ExactFontSourceRegistry generation + exact source handle
  -> bounded PreparedPortableFontSource
       Arc<[u8]> + digest + resource key + face index + immutable face metadata
       (unique source당 한 번 준비)
       -> page ResourceArena에는 blob/face 한 번 등록
       -> N개의 run은 같은 face key를 참조

PageLayerTree JSON
  -> default: 현행 inline fontBlobs
  -> opt-in: metadata/dataRef는 유지, font bytes만 omit
       -> WASM exact getSourceFontBytes(resource key)
            -> Studio document-generation verified cache
                 -> first miss에서 한 번 fetch·검증
                 -> 이후 page는 같은 verified bytes/typeface 재사용
```

prepared source의 신원은 registry generation, exact source handle, digest, byte length, face index로 고정한다. raw font
bytes·host path·문서 원문은 trace나 JSON metadata에 기록하지 않는다. cache는 bounded하며 generation이 바뀌면 이전
entry를 재사용하지 않는다.

## 4. 계측과 판정 기준

private corpus와 Hyper-V·한컴 Oracle은 사용하지 않는다. 저장소의 공개 Source Han subset fixture로
`1/2/8 run × 1/2/8 page`만 계측한다. D4 단일 fixture 전체 계측은 이미 비용 기준선이 있으므로 반복하지 않는다.

| 축 | 구조적 종료 게이트 |
| --- | --- |
| 같은 page의 1/2/8 run | font blob 1, face 1, payload bytes 1회분, GlyphRun만 N |
| prepared source | full-font digest·face metadata 준비가 run 수가 아니라 unique source 수에 귀속 |
| 1/2/8 page transport | opt-in 최초 fetch 뒤 font byte 전달 합계가 page 수에 비례하지 않음 |
| inline 호환 | opt-in을 주지 않은 기존 JSON과 renderer 선택 결과 변화 0 |
| generation | 동일 generation 재사용, 새 generation은 이전 verified entry 재사용 0 |
| fail-closed | missing·wrong·stale·oversized key에서 GlyphRun replay 0, TextRun 유지, partial cache 0 |
| 성능 | warm/cold 절대값과 1→2→8 증가분을 기록; 임의 wall-time 합격선은 두지 않음 |

성능 판정은 “8 run이 몇 µs 이하”가 아니다. font 전체 digest·parse·전송 횟수가 unique exact source 수를 넘지
않는지를 counter와 payload byte 회계로 증명한다. run별 glyph geometry lowering 비용은 별도로 남긴다.

## 5. 구현 절편

### Q2-D5-R0 — public matrix·counter red 계약

1. 새 Rust 회귀 원본을 `tests/cases/issue_4969_*.rs`에만 추가한다.
2. 1/2/8 run에서 blob·face 개수, full-font digest·face 준비 횟수와 payload bytes를 관측할 bounded test seam을
   공개 계약 또는 기존 직렬화 결과로 고정한다.
3. 1/2/8 page에서 inline byte 합계와 opt-in by-key fetch 횟수를 기록할 Studio test fixture를 만든다.
4. 현행 구현이 run별 준비 또는 page별 전송 조건을 위반하는 red를 증명하되 제품 동작은 바꾸지 않는다.

**종료 게이트**: 기존 page-local dedup은 green, 새 unique-source preparation·cross-page transport 계약은 수정 전
red 원인이 명확하다. 제품 소스 `#[cfg(test)]` 증가와 public schema 확장은 0이다.

### Q2-D5-R1 — unique source prepared identity cache

1. exact source registry generation과 handle을 key로 bounded prepared source를 만든다.
2. BLAKE3 digest, resource key, byte length, face index와 immutable face metadata를 unique source당 한 번 계산한다.
3. 기존 certificate의 `Arc<[u8]>`를 공유하고 page lowerer는 prepared identity로 blob·face를 한 번만 등록한다.
4. stale generation·digest/length/face mismatch·상한 초과는 mutation 전에 typed reject한다.
5. cache exhaustion은 eviction 또는 준비 거부로 닫되 stale entry나 부분 resource를 남기지 않는다.

**종료 게이트**: 1/2/8 run에서 full-font 준비 횟수 1, blob 1, face 1, font byte copy 0. D4 output과 non-target
render/layer JSON hash 변화 0.

### Q2-D5-R2 — opt-in font by-key transport

1. layer JSON option에 font byte omission을 별도 opt-in으로 추가하고 default inline bytes를 보존한다.
2. `fontBlobs` metadata의 resource key·digest·length·face identity를 유지하고 payload만 생략한다.
3. WASM에 exact resource key로만 동작하는 bounded `getSourceFontBytes` 또는 동등한 resolver를 추가한다.
4. Studio는 document generation을 포함한 verified cache key를 사용하고 최초 miss에서만 bytes를 fetch한다.
5. fetch 뒤 digest·length·resource key·상한을 다시 검증한 후에만 typeface cache로 넘긴다.
6. missing·wrong·stale·oversized 응답에서는 strict GlyphRun을 선택하지 않고 TextRun으로 닫는다.

**종료 게이트**: default JSON bytes·기존 API 동작 변화 0. opt-in 1/2/8 page에서 inline font payload 0,
font fetch 1, TextRun fallback을 포함한 음성 matrix 전건 green.

### Q2-D5-R3 — resource qualification

1. Rust native/WASM, Studio selector/draw, CanvasKit 실제 replay를 함께 검증한다.
2. D4 공개 오라클과 1/2/8 matrix에서 digest·face preparation·fetch·payload byte counter를 확정한다.
3. warm/cold layer build, JSON 크기, retained bytes와 cache invalidation 결과를 D4 기준선과 분리 기록한다.
4. inline·by-key·cache miss가 같은 glyph ID·position·advance·cluster와 CanvasKit bbox를 내는지 대사한다.

**종료 게이트**: 구조적 resource gate와 cross-backend parity가 모두 qualified. 실패하면 D4 bounded lane은
유지하고 N0/N1을 시작하지 않는다.

### Q2-D5-N0 — NO_LS dormant owner·rollback 계약

1. 모델 `LineSeg`가 없는 ordinary single-interval 문단만 feature-detect한다.
2. Q2-C line range와 final target measurement `Arc`가 같은 `ComposedParagraph` transaction에 속함을 대사한다.
3. line selection·bbox·next origin·page sidecar/GlyphRun 네 소비자의 동일 `Arc` 계약을 red→green shadow로 고정한다.
4. multi-interval frame, edit reflow, stored prefix, split cell, inline control, 다중 target은 typed reject한다.
5. 일부 준비 실패 시 pristine W9/K0 composition으로 네 소비자를 함께 rollback한다.

**종료 게이트**: 제품 caller 0, serialization/output 변화 0, 실패 지점별 partial boundary·sidecar·resource 0.

### Q2-D5-N1 — NO_LS atomic activation

1. R3-qualified resource path와 N0-qualified owner transaction을 동시에 만족하는 문단만 activation한다.
2. 모든 target의 sidecar·prepared font resource를 예약한 뒤 line boundary와 geometry를 게시한다.
3. line width = TextRun bbox width = next origin delta = same shaping measurement total advance를 유지한다.
4. LayerBuilder는 같은 sidecar `Arc`에서 common GlyphRun을 만들고 nominal duplicate는 만들지 않는다.
5. 어떤 target이나 backend 준비가 실패해도 전체 문단을 기존 composition과 TextRun으로 rollback한다.

**종료 게이트**: boundary-changing fixture에서 네 소비자가 `Arc::ptr_eq`, TextRun 1 + common GlyphRun 1,
nominal duplicate 0. 음성 matrix의 line/layer/render JSON은 기준선과 동일하다.

### Q2-D5-N2 — 최종 cross-backend·성능 판정

1. native·Node WASM parity, CanvasKit draw, Canvas2D·legacy SVG·Native Skia fallback을 대사한다.
2. K0·W9·D0~D4와 non-target 회귀를 실행하고 boundary·resource counter를 함께 기록한다.
3. 최초 1/2/8 matrix와 N1 활성 결과를 분리해 성능·payload·retained cache 보고서를 작성한다.
4. 지원 범위를 `qualified`, `qualified-bounded`, `blocked` 중 하나로 판정한다.

**종료 게이트**: 전체 회귀 0, fail-closed 위반 0, resource 비용이 unique source에 귀속. Q2 최종 support
classification은 별도 다음 단계에서 확정한다.

## 6. 실패 원자성

| 실패 지점 | 필수 결과 |
| --- | --- |
| source handle/generation/digest/face 불일치 | prepared entry·page resource 0, TextRun 유지 |
| cache 상한·eviction 충돌 | stale entry 재사용 0, partial resource 0 |
| by-key missing/wrong/oversized | strict GlyphRun replay 0, verified cache mutation 0, TextRun 유지 |
| NO_LS range/owner 불일치 | line boundary·bbox·next origin·sidecar 모두 기존 owner |
| target batch 중 한 건 실패 | 문단 전체 rollback, 일부 shaped line/run 게시 0 |
| CanvasKit typeface/draw 실패 | 빈 출력 금지, TextRun fallback 선택 |

## 7. 제출·검증 규칙

- 새 Rust 회귀는 `tests/cases/` 원본만 source PR에 포함한다.
- 제품 소스 `src/**`의 `#[cfg(test)]`에 새 회귀·test support를 추가하지 않는다.
- `tests/generated/**`, `tests/suites/manifest.json`, unit-tier inventory와 Cargo generated target은 stage하지 않는다.
- task worktree에서는 `rust-test-suite-manifest --prepare/--check`를 실행하지 않는다.
- 원본 commit 뒤 별도 review worktree에서만 `--prepare`·manifest `--check`와 focused/전체 integration을 실행한다.
- module harness 비호환이 실제 실행으로 확인될 때만 별도 근거·승인을 거쳐 maintainer registry 정정을 검토한다.
- 모든 push 전 준비된 exact candidate에서 `cargo fmt --all`과 `cargo fmt --all -- --check`를 반드시 통과한다.
- Rust focused/nextest, native·WASM clippy/build, Studio unit/build/E2E, Markdown link·`git diff --check`를 변경 범위대로
  실행한다.
- generated suite가 없는 task worktree의 `cargo fmt --all` 진입 실패는 source 실패로 오인하지 않되, review
  worktree의 필수 format gate를 대신하지 않는다.

## 8. 제외 범위

- multi-interval frame, 일반 edit reflow, stored prefix, split cell, inline control
- mixed target run 자동 분할, center/right/justify/distribute 확대
- RTL, vertical, variation, nonzero GPOS y positioning
- Native Skia blob typeface replay 신규 구현
- public rejected-attempt annotation 또는 무관한 schema 확장
- private 10k corpus·Hyper-V·한컴 Oracle 재실행
- D4 단일 fixture 전수 계측의 무근거 반복

## 9. 승인 게이트

이 문서의 승인은 전체 D5 구현을 한 번에 여는 승인이 아니다. 첫 구현 승인으로 제품 출력을 바꾸지 않는
`Q2-D5-R0` red 계약을 qualified-red로 고정했고 `Q2-D5-R1` unique-source prepared identity cache,
`Q2-D5-R2` opt-in font-by-key transport, `Q2-D5-R3` resource qualification, `Q2-D5-N0` dormant
owner·rollback 계약과 `Q2-D5-N1` atomic activation을 qualified로 마쳤고, `Q2-D5-N2` 최종 cross-backend·성능
판정을 `qualified-bounded`로 마쳤다. N2 결과 승인과 checkpoint `422a8f7bc`, 최신 devel 병합
`c0998c280`, 재자격화와 증적 checkpoint `167e3b3d2`도 완료했다. Q2 최종 support classification은
`bounded-subset`으로 승인됐고 checkpoint commit도 승인됐다. Q3 수정 수행계획, remote push·PR 생성·GitHub
comment·merge는 기존 별도 승인 경계를 유지한다.
