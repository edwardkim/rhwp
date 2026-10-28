# 처리결과: #3141 1단계 — cargo-fuzz 파서 퍼징 인프라 도입

- 관련 이슈: #3141 ([RFC][fuzzing] cargo-fuzz 파서 퍼징 인프라 도입 + OSS-Fuzz 등재 로드맵)
- 범위: RFC 로드맵의 **1단계**(cargo-fuzz 스캐폴드 + 1순위 하네스 4개 + 시드 코퍼스 + 운영 문서)
- 기준 커밋: upstream/devel `139971ac`

## 1. 무엇을 추가했나

### 1-1. `fuzz/` 크레이트 (cargo-fuzz 표준 구조)

- `fuzz/Cargo.toml` — `rhwp-fuzz` 독립 크레이트.
  - `[workspace] members = ["."]` 로 루트 크레이트와 분리 — 본 크레이트의
    빌드·의존성·`cargo test` 에 아무 영향이 없음.
  - `[package.metadata] cargo-fuzz = true` 표준 마커 포함.
  - 의존성은 `libfuzzer-sys 0.4` + 경로 의존 `rhwp` 뿐.
- `fuzz/.gitignore` — `target/`, `artifacts/`, `coverage/`, `Cargo.lock` 제외.

### 1-2. 1순위 하네스 4개 (`fuzz/fuzz_targets/`)

이슈 4장의 1순위 진입점 4종과 1:1 대응. 시그니처는 현재 devel 기준으로 확인함.

| 하네스 | 호출 진입점 | 확인 위치 |
|---|---|---|
| `parse_hwp.rs` | `rhwp::parser::parse_hwp(data: &[u8])` | `src/parser/mod.rs:175` |
| `parse_hwp3.rs` | `rhwp::parser::hwp3::parse_hwp3(data: &[u8])` | `src/parser/hwp3/mod.rs:2921` |
| `parse_hwpx.rs` | `rhwp::parser::hwpx::parse_hwpx(data: &[u8])` | `src/parser/hwpx/mod.rs:254` |
| `parse_hml.rs` | `rhwp::parser::hml::parse_hml(bytes: &[u8])` | `src/parser/hml/mod.rs:38` |

참고: `parse_hwp3`는 이슈 본문에 `:2856`으로 적혀 있으나 현 devel에서는
`:2921`로 이동해 있음(시그니처는 동일). 각 하네스는 RFC 6장 2항 그대로
`let _ = parse_xxx(data);` 형태 — 반환 `Err`는 정상이고 패닉/abort/자원
고갈/타임아웃만 검출 대상. `parse_hml`은 기본 `HmlLimits`가 적용되는 공개
경로를 그대로 사용해 상한 기구의 빈틈(#2743류)을 탐지 대상에 포함.

### 1-3. 시드 코퍼스 (`fuzz/corpus/<타깃>/`)

저장소에 이미 커밋돼 있는 샘플 중 작은 것들을 복사(총 10개, 최대 89KB).

| 타깃 | 시드 | 출처 |
|---|---|---|
| `parse_hwp` | english.hwp(29KB) · Textmail.hwp(34KB) · shortcut.hwp(42KB) | `samples/basic/` |
| `parse_hwp3` | hwp3-pagedef-1915.hwp(2.4KB) · hwp3-sample.hwp(87KB) | `samples/` |
| `parse_hwpx` | neartop_reset_sb2500.hwpx(3.7KB) · saved_single_line_spacing_after.hwpx(3.7KB) · tac-host-spacing.hwpx(4.1KB) | `samples/task2136/` · `samples/task2093/` · `samples/` |
| `parse_hml` | exambank_math_equations_min.hml(4KB) · formatting_table.hml(29KB) | `tests/fixtures/hml/` · `samples/hml/` |

CFB/ZIP 컨테이너 포맷은 구조 제약이 강해 시드 없이는 변이가 깊이 못
들어가므로(RFC 9장), 포맷별 유효 샘플을 시드로 두는 것이 커버리지의 전제.

### 1-4. 운영 문서 (`fuzz/README.md`, 한글)

실행법, 권장 플래그(`-rss_limit_mb=2048 -timeout=30` — 각각 #2743류 OOM,
#3012류 무한루프 검출용), Windows `dbghelp.lib` 우회법, 트리아지 절차
(재현 → `tmin` 최소화 → `fuzz/regressions/<타깃>/` 보존 → 이슈/수정 PR →
클래스 반복 시 전수 스윕 이슈), 코퍼스 운영 방침을 문서화.

## 2. 검증

- 이 환경에는 nightly 툴체인·cargo-fuzz가 설치돼 있지 않아
  (`rustup toolchain list`: stable만) `cargo +nightly fuzz build` 스모크는
  수행하지 못함.
- 대신 `fuzz/` 디렉터리에서 `cargo check` 로 하네스 4개 전부의 컴파일을 확인:
  - 명령: `RUSTFLAGS="-C linker=rust-lld" cargo check` (MSVC `dbghelp.lib`
    손상으로 인한 `CVT1107`/`LNK1123` 링크 오류를 rust-lld로 우회 — README에
    동일 우회법 기재)
  - 결과: `Finished dev profile ... in 1m 38s` — rhwp 본 크레이트 + rhwp-fuzz
    4개 bin 모두 통과, 경고 없음.
- 루트 크레이트 영향 없음: 기존 소스는 한 줄도 수정하지 않았고, `fuzz/` 는
  `[workspace]` 분리로 루트 `cargo build/test` 에서 인식되지 않음.
- 장시간 퍼징 실행은 하지 않음(로컬 환경 제약 + 트리아지 유입 속도 조절은
  메인테이너 판단 영역).

## 3. 범위에서 제외한 것 (후속 단계)

- **CI 통합** — RFC 6장 5항(PR당 스모크 퍼징/회귀 코퍼스 재생)은 기존
  워크플로 변경을 수반하므로 이번 PR에서 제외. `.github/workflows` 는 건드리지 않음.
- **2순위 하네스** — `parse_body_text_section` / `parse_doc_info` /
  `parse_control` / WMF·EMF 직접 하네스는 1순위에서 크래시 유입이 잦아들면 추가.
- **`fuzz/regressions/`** — 첫 발견물이 나올 때 트리아지 절차와 함께 도입.
- **OSS-Fuzz 등재** — RFC 7장, 메인테이너 판단 사항.

## 4. 변경 파일

```
fuzz/.gitignore
fuzz/Cargo.toml
fuzz/README.md
fuzz/fuzz_targets/parse_hwp.rs
fuzz/fuzz_targets/parse_hwp3.rs
fuzz/fuzz_targets/parse_hwpx.rs
fuzz/fuzz_targets/parse_hml.rs
fuzz/corpus/parse_hwp/{english,Textmail,shortcut}.hwp
fuzz/corpus/parse_hwp3/{hwp3-pagedef-1915,hwp3-sample}.hwp
fuzz/corpus/parse_hwpx/{neartop_reset_sb2500,saved_single_line_spacing_after,tac-host-spacing}.hwpx
fuzz/corpus/parse_hml/{exambank_math_equations_min,formatting_table}.hml
mydocs/report/task_m100_3141_report.md
```
