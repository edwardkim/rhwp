# rhwp 퍼징 인프라 (cargo-fuzz)

RFC #3141의 1단계 구현입니다. `cargo-fuzz`(libFuzzer) 기반으로 rhwp의 포맷 최상위
파서 진입점 4개를 퍼징합니다. 목적은 **비정상·적대적 입력**에 대한
크래시(패닉/abort) · 자원 고갈(OOM) · 무한루프(타임아웃) 검출입니다.
정상 입력의 왕복 정합성(#2740 영역)은 이 인프라의 대상이 아닙니다.

## 하네스 목록

| 타깃 | 진입점 | 위치 |
|---|---|---|
| `parse_hwp` | `rhwp::parser::parse_hwp(&[u8])` — HWP 5.x (CFB) | `src/parser/mod.rs` |
| `parse_hwp3` | `rhwp::parser::hwp3::parse_hwp3(&[u8])` — HWP 3.x | `src/parser/hwp3/mod.rs` |
| `parse_hwpx` | `rhwp::parser::hwpx::parse_hwpx(&[u8])` — HWPX (ZIP) | `src/parser/hwpx/mod.rs` |
| `parse_hml` | `rhwp::parser::hml::parse_hml(&[u8])` — HML (XML) | `src/parser/hml/mod.rs` |

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
```

### 권장 플래그

- `-rss_limit_mb=2048` — 무검증 할당(#2743류)을 OOM 크래시로 검출합니다.
  libFuzzer 기본값도 2048이지만, 의도를 명시하기 위해 항상 지정할 것을 권장합니다.
- `-timeout=30` — 부호확장 무한루프(#3012류)나 사실상 종료되지 않는 경로를
  타임아웃으로 검출합니다. 기본값(1200초)은 이 용도에 너무 깁니다.
- 병렬 실행이 필요하면 `-jobs=N -workers=N` 을 추가합니다.

### Windows 참고

MSVC 링크 단계에서 `dbghelp.lib` 관련 오류가 나면 rust-lld로 우회합니다:

```sh
RUSTFLAGS="-C linker=rust-lld" cargo +nightly fuzz build
```

## 시드 코퍼스

`fuzz/corpus/<타깃>/` 에 저장소의 기존 샘플 중 작은 파일들을 복사해 두었습니다.
CFB/ZIP처럼 구조 제약이 강한 컨테이너 포맷은 시드 없이는 변이가 깊이
들어가지 못하므로, 시드가 실질적인 커버리지를 좌우합니다.

| 코퍼스 | 출처 |
|---|---|
| `corpus/parse_hwp/` | `samples/basic/` (english, Textmail, shortcut) |
| `corpus/parse_hwp3/` | `samples/` (hwp3-pagedef-1915, hwp3-sample) |
| `corpus/parse_hwpx/` | `samples/task2136`, `samples/task2093`, `samples/` (tac-host-spacing) |
| `corpus/parse_hml/` | `tests/fixtures/hml/`, `samples/hml/` |

퍼징 중 커버리지를 넓힌 입력은 같은 디렉터리에 자동 축적됩니다.
유의미하게 커버리지를 늘린 최소화 입력만 선별해 커밋하는 것을 권장합니다
(`cargo +nightly fuzz cmin <타깃>` 으로 코퍼스를 최소화할 수 있습니다).

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
  WMF·EMF 등 컨테이너를 우회하는 내부 파서 직접 하네스
- CI 통합: PR당 짧은 스모크 퍼징 또는 회귀 코퍼스 재생
- OSS-Fuzz 등재 (메인테이너 판단)
