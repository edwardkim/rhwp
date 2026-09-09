# Task #3273 Stage 1 — WMF·ooxml_chart 퍼징 하네스 확장 (수행계획서)

## 배경

#3141(RFC) 2단계. 1단계(#3158)가 hwp/hwp3/hwpx/hml 파서 4개에 cargo-fuzz 하네스를
도입했으나, 이번 주 자원고갈·부호확장 결함이 가장 많이 나온 **WMF 파서**에는 하네스가
없다. 메인테이너 코드 스캔으로 같은 결함 클래스가 아직 남아 있음을 확인했다
(poly_line.rs:47 / polygon.rs:50 number_of_points i16→usize 무검증, bitmap16.rs:126
i16 곱셈 오버플로). 손으로 고치는 대신 **퍼저가 자동 발견하도록 하네스를 먼저 확장**한다.

## 진입점 확정 (코드 확인 완료)

- **WMF**: `crate::wmf::converter::{WMFConverter, SVGPlayer}` — 둘 다 `pub`.
  `convert_wmf_to_svg`(svg.rs:3308)가 `&[u8]` 를 그대로 `WMFConverter::new(data, SVGPlayer::new())`
  에 넘기고 `.run()` 하므로, 하네스도 동일 조합을 쓴다. `convert_wmf_to_svg` 자체는
  `pub(crate)` 라 fuzz 크레이트에서 접근 불가 → 조합을 하네스에 직접 편다.
- **ooxml_chart**: `crate::ooxml_chart::parse_chart_xml(&[u8]) -> Option<OoxmlChart>` — `pub`.

## 작업 항목

1. `fuzz/fuzz_targets/parse_wmf.rs` — `let _ = WMFConverter::new(data, SVGPlayer::new()).run();`
   (반환 무시, 패닉/abort/자원고갈/타임아웃만 검출 — 1단계 하네스와 동일 철학)
2. `fuzz/fuzz_targets/parse_ooxml_chart.rs` — `let _ = rhwp::ooxml_chart::parse_chart_xml(data);`
3. `fuzz/Cargo.toml` — `[[bin]]` 2개 추가 (test/doc/bench = false)
4. `fuzz/corpus/parse_wmf/`, `fuzz/corpus/parse_ooxml_chart/` — 시드 corpus
5. `fuzz/README.md` — 타깃 표 2행 추가

## 시드 corpus 전략 (결정 필요 지점)

WMF/차트 샘플이 저장소에 **독립 파일로 존재하지 않는다** — HWP/HWPX 컨테이너 안에
임베드돼 있다. 두 안:

- **안 A (권장)**: 최소 유효 시드를 코드로 합성해 커밋한다. WMF 는 META_PLACEABLE
  (magic 0x9AC6CDD7) + META_HEADER + 최소 레코드, 차트는 `parser.rs` 테스트의 인라인
  BAR_XML(이미 존재)을 파일로 저장. 퍼저는 유효 시드에서 커버리지를 넓히므로 최소 1개면
  씨앗 역할을 한다. 컨테이너 포맷(WMF)은 유효 시드가 커버리지의 전제.
- **안 B**: 임베드 WMF/차트를 추출하는 도구를 만들어 실제 샘플을 뽑는다. 정확하지만
  범위가 커지고 별도 검증이 필요하다.

→ **안 A 로 진행 제안**. 차트는 인라인 XML 재사용으로 즉시 가능, WMF 는 최소 헤더
합성으로 1 시드 확보.

## 검증 방침 (#3158 과 동일 조건)

- cargo-fuzz 빌드·실행은 nightly 도구체인 필요 → **로컬 재실증 범위 밖**.
- 대신 확인하는 것: ①`fuzz/` 가 workspace 분리라 `cargo test --tests`·본 빌드에 무영향
  (기존 1단계에서 이미 성립) ②하네스가 참조하는 진입점 시그니처가 현 devel 과 정합
  (컴파일 가능성을 진입점 타입으로 정적 확인) ③시드 파일이 유효 매직/구조.
- 실제 퍼징 실행은 CI/후속 단계에 위임 (PR 본문에 조건 명시).

## 다음 단계

승인 시 Stage 2(구현계획서 없이 바로 구현 — 작업이 파일 5개 추가로 단순)로 진행하거나,
구조가 승인되면 구현→게이트→PR. 구현은 기존 소스 0줄 수정이 원칙(하네스는 순수 추가).
