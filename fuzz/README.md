# rhwp 퍼징 인프라 (cargo-fuzz)

RFC #3141의 1~2단계 구현입니다(1단계 #3158: 포맷 파서 4개 / 2단계 #3273: 임베드
WMF·OOXML 차트 2개). `cargo-fuzz`(libFuzzer) 기반으로 rhwp의 포맷 최상위
파서 진입점 6개(포맷 4 + 임베드 WMF·OOXML 차트)를 퍼징합니다. 목적은 **비정상·적대적 입력**에 대한
크래시(패닉/abort) · 자원 고갈(OOM) · 무한루프(타임아웃) 검출입니다.
정상 입력의 왕복 정합성(#2740 영역)은 이 인프라의 대상이 아닙니다.

## 하네스 목록

| 타깃 | 진입점 | 위치 |
|---|---|---|
| `parse_hwp` | `rhwp::parser::parse_hwp(&[u8])` — HWP 5.x (CFB) | `src/parser/mod.rs` |
| `parse_hwp3` | `rhwp::parser::hwp3::parse_hwp3(&[u8])` — HWP 3.x | `src/parser/hwp3/mod.rs` |
| `parse_hwpx` | `rhwp::parser::hwpx::parse_hwpx(&[u8])` — HWPX (ZIP) | `src/parser/hwpx/mod.rs` |
| `parse_hml` | `rhwp::parser::hml::parse_hml(&[u8])` — HML (XML) | `src/parser/hml/mod.rs` |
| `parse_wmf` | `WMFConverter::new(data, SVGPlayer::new()).run()` — WMF (임베드 이미지) | `src/renderer/svg.rs:3308` |
| `parse_ooxml_chart` | `rhwp::ooxml_chart::parser::parse_chart_xml(&[u8])` — OOXML 차트 | `src/ooxml_chart/parser.rs` |

각 하네스는 `let _ = parse_xxx(data);` 형태로 반환값을 무시합니다 —
파서가 `Err`를 돌려주는 것은 정상 동작이며, 퍼저가 잡는 것은
패닉/abort/자원 고갈/타임아웃뿐입니다.

## 사전 준비

cargo-fuzz는 nightly 툴체인이 필요합니다.

```sh
rustup toolchain install nightly
cargo install cargo-fuzz
```

## 실행

저장소 루트에서:

```sh
# 빌드만 (전 타깃)
cargo +nightly fuzz build

# 개별 타깃 실행 — 권장 플래그 포함
cargo +nightly fuzz run parse_hwp  -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hwp3 -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hwpx -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_hml  -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_wmf  -- -rss_limit_mb=2048 -timeout=30
cargo +nightly fuzz run parse_ooxml_chart -- -rss_limit_mb=2048 -timeout=30
```

### 권장 플래그

- `-rss_limit_mb=2048` — 무검증 할당(#2743류)을 OOM 크래시로 검출합니다.
  libFuzzer 기본값도 2048이지만, 의도를 명시하기 위해 항상 지정할 것을 권장합니다.
- `-timeout=30` — 부호확장 무한루프(#3012류)나 사실상 종료되지 않는 경로를
  타임아웃으로 검출합니다. 기본값(1200초)은 이 용도에 너무 깁니다.
- 병렬 실행이 필요하면 `-jobs=N -workers=N` 을 추가합니다.

## Nightly 스모크 CI

`.github/workflows/fuzz-smoke.yml` 이 **nightly(`02:41 UTC`) + `workflow_dispatch`** 로
위 기존 타깃 6개를 각 60초 돌립니다. 플래그는 위와 같습니다
(`-rss_limit_mb=2048 -timeout=30` + `-max_total_time=60`).
크래시·타임아웃·OOM 이 나면 job 이 실패하고 `fuzz/artifacts/<타깃>/` 을
아티팩트로 올립니다.

이 잡은 PR required check 가 아닙니다 (`pull_request` 트리거 없음).
왕복 정합성(#2740, M04)과 OSS-Fuzz 등재(M10)는 이 잡의 범위가 아닙니다.
발견된 DoS 를 이 단계에서 고치지 않습니다(M03-3 이후).

### Windows 참고

MSVC 링크 단계에서 `dbghelp.lib` 관련 오류가 나면 rust-lld로 우회합니다:

```sh
RUSTFLAGS="-C linker=rust-lld" cargo +nightly fuzz build
```

## 시드 코퍼스

`fuzz/corpus/<타깃>/` 가 nightly 스모크(`fuzz-smoke.yml`)와 `cargo +nightly fuzz run`
의 기본 코퍼스 경로다. CFB/ZIP처럼 구조 제약이 강한 컨테이너는 시드 없이는
변이가 깊이 들어가지 못하므로, `samples/` 에서 작은 파일을 추출하고
임베드 WMF·차트 XML 을 풀어 넣었다. 개별 시드는 크기 상한(HWP/HWPX 64KiB,
HWP3 128KiB, 추출 WMF 16KiB, 차트 XML 32KiB)을 넘기지 않는다.

| 코퍼스 | 시드 수 | 출처 |
|---|---:|---|
| `corpus/parse_hwp/` | 83 | `samples/` HWP 5.x(CFB) 소형 파일, 디렉터리 라운드로빈 |
| `corpus/parse_hwp3/` | 58 | 매직 `HWP Document File V3.00` 실파일 ≤128KiB + 대형본 2/16/32/64KiB prefix |
| `corpus/parse_hwpx/` | 80 | `samples/` HWPX(ZIP) 소형 파일, 디렉터리 라운드로빈 |
| `corpus/parse_hml/` | 53 | `samples/hml/`·`tests/fixtures/hml/` 원본 3 + 파서 경로용 최소 HWPML |
| `corpus/parse_wmf/` | 67 | HWP/HWPX 임베드 추출 + placeable/표준 레코드 최소 시드 |
| `corpus/parse_ooxml_chart/` | 79 | HWPX `Chart/*.xml`·`c:chartSpace` 추출 + 차트 타입 최소 XML |

HWP3 실파일은 저장소에 23개뿐이라 50+ 를 prefix 로 채웠다. 대형 원본(수백
KiB~수 MiB)은 커밋하지 않는다. HML 원본도 3개뿐이라 파서가 실제로 읽는
요소(표·수식·RECTANGLE·인코딩 BOM 등)만 최소 문서로 보강했다.

퍼징 중 커버리지를 넓힌 입력은 같은 디렉터리에 자동 축적됩니다.
유의미하게 커버리지를 늘린 최소화 입력만 선별해 커밋하는 것을 권장합니다
(`cargo +nightly fuzz cmin <타깃>` 으로 코퍼스를 최소화할 수 있습니다).
Windows 호스트에서 `cmin --sanitizer none` 은 rust-lld 가 `__sanitizer_cov_*`
심볼을 못 풀어 링크가 실패한다. 코퍼스는 SHA-256 중복 제거와 크기 상한으로
먼저 줄였고, nightly 우분투 잡에서 cmin 을 돌릴 수 있다.

## 트리아지 절차

크래시/타임아웃/OOM이 나오면:

1. **재현** — 산출물은 `fuzz/artifacts/<타깃>/` 에 저장됩니다.
   `cargo +nightly fuzz run <타깃> fuzz/artifacts/<타깃>/<파일>` 로 단건 재현합니다.
2. **최소화** — `cargo +nightly fuzz tmin <타깃> fuzz/artifacts/<타깃>/<파일>` 로
   재현 입력을 최소화합니다.
3. **회귀 입력 보존** — 최소화한 입력은 `fuzz/corpus/<타깃>/` 이 아니라
   `fuzz/regressions/<타깃>/` 에 커밋합니다(코퍼스와 회귀 케이스를 분리).
4. **이슈 → 수정 PR** — 기존 관행대로 이슈를 먼저 등록하고, 수정 PR에
   해당 입력을 단위 테스트로 동봉합니다(#2743의 재현 파일 방식과 동일).
5. **클래스 반복 시** — 같은 결함 클래스(예: 부호 있는 정수 → `usize` 무검증
   캐스팅)가 반복되면 해당 클래스 전수 스윕 이슈를 별도로 엽니다
   (#3004 → #3012 흐름과 동일).

## 범위와 후속 단계

이 디렉터리는 본 크레이트의 빌드·의존성에 영향을 주지 않는 독립
크레이트입니다(`fuzz/Cargo.toml`의 `[workspace]`로 루트에서 분리).

후속 단계(#3141 로드맵의 나머지):

- 2순위 하네스: `parse_body_text_section` / `parse_doc_info` / `parse_control` /
  EMF 등 나머지 임베드 포맷·컨테이너를 우회하는 내부 파서 직접 하네스
- nightly 스모크는 위 CI 절. PR 게이트·왕복 정합성(M04)은 여기 넣지 않습니다
- OSS-Fuzz 등재 (M10, 메인테이너 판단)
- CI 통합: PR당 짧은 스모크 퍼징 또는 회귀 코퍼스 재생
- CI 통합: PR당 짧은 스모크 퍼징 또는 회귀 코퍼스 재생 (왕복 정합성은 아래 M04)
- OSS-Fuzz 등재 (메인테이너 판단)

## 왕복 정합성은 여기가 아니다 (M04)

위 서두가 비워 둔 **정상 입력의 왕복 정합성(#2740, IrDiff-0)** 은 cargo-fuzz
범위 밖이다. 그 공백을 메우는 계층이 M04 다 — 퍼저 타깃을 늘리지 않는다.
- **M04-1** (`tests/cases/prop_edit_plan.rs`, #5363): `proptest` 의존 +
  기존 `rhwp run` step(`fill_fields` · `replace_text` · `set_cell` ·
  `set_checkbox`)만 조합하는 편집 시퀀스 생성기. 생성 계획은 JSON
  직렬화/역직렬화와 `export-plan-schema` 정적 검증을 통과하고, 처음부터
  잘못된 계획은 거부된다. DocumentCore 변이를 직접 실행하지 않는다.
- **M04-2/3**: 실제 HWPX/HWP5 IrDiff-0 왕복 property. 이 생성기를 쓰되
  본 단계는 여기 구현하지 않는다.
- **M04-4**: 그 property 의 CI 배선.
- **M04-2** (`tests/cases/prop_hwpx_roundtrip.rs`, #5376): 작은 HWPX 픽스처에
  기존 `rhwp run` step 만 적용한 뒤 parse→serialize→reparse 가
  [`diff_documents`](../src/serializer/hwpx/roundtrip.rs) IrDiff 0.
  CI 기본은 8 cases / 0..3 steps. 전체 화력은 `PROPTEST_CASES`
  (예: `PROPTEST_CASES=256 cargo test --test regression_suite_* prop_hwpx_roundtrip::`).
  픽스처가 표현하지 못하는 step(누름틀/표/□ 없음)은 skip. DocumentCore
  편집 API 발명 금지. HWP5 는 M04-3.
- **M04-3** (`tests/cases/prop_hwp5_roundtrip.rs`, #5382): 작은 HWP5 픽스처에
  기존 `rhwp run` step 만 적용한 뒤 parse→serialize→reparse 가
  [`diff_documents`](../src/serializer/hwpx/roundtrip.rs) IrDiff 0.
  CI 기본은 8 cases / 0..3 steps. 전체 화력은 `PROPTEST_CASES`
  (예: `PROPTEST_CASES=256 cargo test --test regression_suite_* prop_hwp5_roundtrip::`).
  픽스처가 표현하지 못하는 step(누름틀/표/□ 없음)은 skip. DocumentCore
  편집 API 발명 금지. HWPX 는 M04-2.
왕복은 property 계층이지 퍼지가 아니다.

- **CI** (`.github/workflows/proptest-roundtrip.yml`,
  `scripts/run-prop-roundtrip.mjs`): debug 프로필, 기본 8 cases. 10분 퍼지가
  아니다. `tests/cases/prop_hwpx_roundtrip.rs`(M04-2, #5381) 와
  `tests/cases/prop_hwp5_roundtrip.rs`(M04-3, #5387) 가 있으면 돌리고, 없으면
  skip 한다. 배선 확인용 `tests/cases/prop_roundtrip_ci.rs` 는 항상 돈다.
  nextest archive 정규 shard 도 같은 `tests/cases/` 원본을 자동 실행한다.
- **본체**: 작은 픽스처에 기존 `rhwp run` step 만 적용한 뒤
  parse→serialize→reparse 가
  [`diff_documents`](../src/serializer/hwpx/roundtrip.rs) IrDiff 0.
  전체 화력은 `PROPTEST_CASES` (예:
  `PROPTEST_CASES=256 cargo test --test regression_suite_* prop_hwpx_roundtrip::`).
