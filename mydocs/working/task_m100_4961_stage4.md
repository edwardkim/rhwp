# Task M100 #4961 Stage 4 — 공개 HWP/HWPX E2E와 결정론 검증

- **Issue**: [#4961](https://github.com/edwardkim/rhwp/issues/4961)
- **상위 추적**: [#4960](https://github.com/edwardkim/rhwp/issues/4960)
- **계획**: [`task_m100_4961.md`](../plans/archives/task_m100_4961.md)
- **브랜치**: `local/task4961-font-decision-trace`
- **기준**: Stage 3 commit `83757074a23a73349410c394a961e83631e25831`
- **날짜**: 2026-08-17 KST
- **단계 상태**: Stage 4 기술 완료, Stage 5 메인테이너 승인 대기

## 1. 결과

Stage 4는 repository-tracked 공개 문서만 사용해 exact face, missing face와 document-declared
`substFont`를 서로 다른 profile로 고정했다. native Rust, optimized WASM과 Studio backend 보강이 같은
machine-readable fixture를 소비한다.

| profile | source → layout → metric |
| --- | --- |
| exact | `바탕` → 이름 변경 없음 → exact entry 582 → `embeddedMetric` |
| missing | `HCI Poppy` → `Palatino Linotype` → exact entry 457 → `embeddedMetric` |
| document substitute | `KoPubWorld돋움체 Light` + `HCR Batang` → document chain → `heuristicHalfwidth` |

세 profile은 모두 `source.status=complete`인 record를 사용한다. 원문 face·언어 slot·`altType`·
`substFont`, layout step, metric hit/miss, width source·advance, provenance와 세 paint backend 항목이 같은
record에서 이어지는지 검사한다.

## 2. 공개 fixture와 feature detection

`public_fixtures.json`에 이미 추적된 공개 문서 네 건을 추가했고, path·size·SHA-256·Git 추적 상태를
기존 validator가 검사한다. private 10k corpus와 외부 font bytes는 읽거나 포함하지 않았다.

`font_decision_trace_e2e.json`은 page, 4,096 상한, count와 portable `layoutHash`, profile record ID와
예상 decision을 고정한다.

### 2.1 동일 객체 상태의 HWP/HWPX

`samples/3-10월_교육_통합_2022.hwp`와 `.hwpx` page 0은 다음 결과가 동일하다.

```text
runsSeen=175, charactersSeen=440, recordsEmitted=440
layoutHash=ecc09cc1b2bfc374f5f94871a26a0a6f7e0263fa08eba4f586e4518ad2955251
HWP records == HWPX records
```

### 2.2 `substFont` 객체 상태가 다른 HWP/HWPX

`[2027] 온새미로 1 본교재` 쌍은 page·run·문자 수가 같지만 HWPX에만 document `substFont`가 있다.

| format | substFont 관측 | layoutHash |
| --- | --- | --- |
| HWP | 0 record | `4cc30e2ffa7cabe850ceb85e8d41796eac590091a3786710660760dc24bc515b` |
| HWPX | 41 record | `6dc1c3429ae84ef2c29615d42996e106acdc149cbdebcfc8ac116c587fc54004` |

이를 format/version 정책으로 같게 만들지 않는다. 현재 객체에 `substFont`가 있는지를 feature detection해
trace와 hash가 달라지는 것이 정상이다.

## 3. RED에서 발견한 target-dependent source marker

첫 native/WASM 공유 fixture 검증은 실패했다.

```text
native layoutHash=3cc425c918a5ccf3f1b303bf0e396103c086db640b9a9b7f731d1a789902201d
wasm32 layoutHash=c44ac37549a0d5340d3c7a8a71214cdb7919eb3cb9ff390eb113a3f9816e183f
```

첫 구조 차이는 header/footer record의 `source.paragraphIndex`였다.

```text
native: 18446744073709551615  # usize::MAX on 64-bit
wasm32: 4294967295            # usize::MAX on 32-bit
```

이는 문서 문단 좌표가 아니라 header/footer·note layout이 쓰는 `usize::MAX - n` 내부 marker다. 처음에는
`u32` 변환 가능 여부로 걸렀지만 wasm32의 `usize::MAX` 자체가 `u32`에 들어가므로 RED가 유지됐다.
최종 구현은 각 target의 `usize::MAX - 4096` 근방을 source 좌표에서 제외한다. 해당 record는 값을
추정하지 않고 `paragraphIndex=null`, `source.status=unavailable`을 유지한다.

최종 exact fixture의 native와 wasm32 portable hash는 모두 다음 값이다.

```text
234a429072626a8d57f988ce1be89db5cc64449bc05eaf567ab36bdb5f76384a
```

## 4. backend 차이와 결정론

- native-skia build: native backend `complete`, 실제 후보·glyph capability 포함
- WASM raw query: native `unsupported/nativeSkiaFeatureUnavailable`
- WASM raw query의 Canvas2D·CanvasKit: `unsupported/studioSnapshotRequired`
- Studio Canvas2D: `complete/notObserved`, `cssActualGlyphFaceUnobservable`
- Studio CanvasKit: 현재 SFNT plan이 있으면 `notObserved/planned`; source join이 없으면 명시적
  `backendJoinMissing`

backend 보강 전후 `layoutHash`는 같고 `normalizedHash`는 다르다. 이는 portable layout과 환경 capability
snapshot을 분리한다는 계약에 맞는다.

결정론 검사는 다음 변이를 포함한다.

- 같은 문서·page·options를 두 번 실행해 전체 JSON 동일
- 실제 WASM trace의 모든 object key 순서를 역전해 hash 동일
- 같은 OS font 집합의 insertion order와 local snapshot 배열 순서를 역전해 Studio hash 동일
- fallback candidate 배열 순서는 정책이므로 변경하지 않음

## 5. 상한과 fail-closed

- 공개 exact fixture를 `maxCharacters=1`로 실행하면 record 1건, `truncated`, 실제 누락 수와
  `characterLimitExceeded`를 반환한다.
- `maxCharacters=4097`은 clamp하지 않고 오류로 거부한다.
- 사용할 수 없는 native·Studio backend는 빈 성공 대신 record와 summary 모두 explicit
  `unsupported` reason을 가진다.
- CanvasKit plan은 CSS chain 전체를 하나의 family로 오인하지 않고 첫 실제 family를 사용한다.
- trace 보강은 기존 Stage 3 trap 검증대로 font fetch·권한 요청·repaint를 시작하지 않는다.

## 6. 검증 결과

### 6.1 Rust native

```bash
cargo test --lib --features native-skia stage4_public -- --nocapture
cargo test --lib document_core::queries::font_decision -- --nocapture
cargo clippy --no-default-features --lib -- -D warnings
cargo clippy --features native-skia --lib -- -D warnings
cargo fmt --check
cargo check --target wasm32-unknown-unknown --lib
```

- native-skia Stage 4: **2 passed**
- 전체 font decision query focused: **4 passed**
- clippy 두 구성, format과 wasm32 compile: 통과

### 6.2 optimized WASM·Studio·SDK

```bash
docker compose --env-file .env.docker run --rm wasm
node --test scripts/tests/font_decision_trace_e2e.test.mjs
cd rhwp-studio && node --test tests/font-decision-trace.test.ts tests/embed-protocol.test.ts
npm run build
```

- 표준 Docker optimized WASM build: 통과
- Stage 4 WASM E2E: **3 passed**
- Studio trace·Embed focused: **22 passed**
- Studio TypeScript/Vite production build: 통과

공개 editor embed **2건**, editor transport·diagnostics **13건**, generated WASM binding 검사도 통과했다.
Vite의 기존 대형 chunk 경고는 non-failing이며 새 오류는 없었다.

### 6.3 schema·fixture·문서

```bash
node --test scripts/tests/font_decision_trace_contract.test.mjs
node scripts/font_decision_trace_contract.mjs check
python3 scripts/check_markdown_links.py <Stage 4 문서들>
git diff --check
```

- trace contract: **12 passed**
- 공개 fixture path·size·SHA-256·private boundary: 통과
- repository contract: `font decision trace Stage 1 contracts: ok`

## 7. 종료 게이트 판정

| 항목 | 판정 |
| --- | --- |
| 공개 HWP/HWPX document → layout → metric → backend 계보 | 통과 |
| exact·missing·document substitute profile 분리 | 통과 |
| native·WASM·Canvas2D·CanvasKit 차이 reason 단위 설명 | 통과 |
| 반복·key·font enumeration 순서 결정론 | 통과 |
| 상한·unsupported·source join fail-closed | 통과 |
| target architecture 독립 portable hash | 통과 |
| private corpus·허가되지 않은 font bytes 비포함 | 통과 |

## 8. 다음 승인 지점

Stage 4 변경은 이 보고서를 포함한 별도 commit으로 고정한다. 메인테이너가 Stage 5 진행을 승인하면
FI-01~FI-14, trace disabled output 0-delta, 전체 로컬 gate, API 문서·최종 보고와 W3·W4 인계를
감사한다. remote push와 PR 생성은 여전히 별도 승인 대상이다.
