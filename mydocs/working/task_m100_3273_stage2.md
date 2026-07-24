# Task #3273 Stage 2 — 구현계획서

수행계획서(Stage 1) 승인 완료. 시드 전략 **안 A(최소 유효 시드 합성)** 확정.
기존 소스 0줄 수정, 순수 추가만 한다.

## 추가/수정 파일 (5개, 전부 fuzz/ 하위)

### 1. `fuzz/fuzz_targets/parse_wmf.rs` (신규)
```rust
//! WMF(Windows Metafile) 최상위 변환 진입점 퍼징 하네스.
//! 반환값은 무시 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.
//! convert_wmf_to_svg(renderer/svg.rs)가 pub(crate) 라 그 내부 조합
//! (WMFConverter::new(data, SVGPlayer::new()).run())을 직접 편다.
#![no_main]
use libfuzzer_sys::fuzz_target;
use rhwp::wmf::converter::{SVGPlayer, WMFConverter};

fuzz_target!(|data: &[u8]| {
    let _ = WMFConverter::new(data, SVGPlayer::new()).run();
});
```
근거: svg.rs:3308 `convert_wmf_to_svg(data: &[u8])` 가 동일 조합. `WMFConverter`·`SVGPlayer`
모두 `pub`. `&[u8]` 는 `embedded_io::Read`(= `wmf::Read`) 를 구현하므로 buffer 로 유효.

### 2. `fuzz/fuzz_targets/parse_ooxml_chart.rs` (신규)
```rust
//! OOXML 차트(c:chartSpace) XML 파서 진입점 퍼징 하네스.
//! 반환값은 무시 — 패닉/abort/자원 고갈/타임아웃만 검출 대상이다.
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = rhwp::ooxml_chart::parse_chart_xml(data);
});
```
근거: parser.rs:62 `pub fn parse_chart_xml(xml: &[u8]) -> Option<OoxmlChart>`.

### 3. `fuzz/Cargo.toml` (수정 — [[bin]] 2개 추가)
기존 4개 뒤에 parse_wmf / parse_ooxml_chart 를 동일 형식(test/doc/bench=false)으로 추가.

### 4. 시드 corpus (신규 2개)
- `fuzz/corpus/parse_ooxml_chart/bar_chart.xml` — parser.rs 테스트 BAR_XML 그대로 저장
  (유효 c:chartSpace, 파서가 Some 반환하는 최소 유효 입력).
- `fuzz/corpus/parse_wmf/minimal_placeable.wmf` — META_PLACEABLE(22바이트, key=0x9AC6CDD7
  + hwmf=0 + bounding_box 8B + inch=1440 + reserved=0 + checksum 2B) 다음에 META_HEADER
  최소값(type=1 memory / header_size=9 words / version / size / number_of_objects=0 /
  max_record / number_of_members=0) + META_EOF 레코드(size=3 words, function=0x0000).
  → `MetafileHeader::parse` 가 placeable 분기(0x9AC6CDD7)를 타고 header 를 읽는 경로를 씨앗으로.
  바이트열은 스크립트로 생성해 커밋한다(생성 스크립트는 커밋하지 않고 산출 바이너리만).

### 5. `fuzz/README.md` (수정 — 타깃 표 2행 추가)
| parse_wmf | `WMFConverter::new(data, SVGPlayer::new()).run()` | renderer/svg.rs:3308 |
| parse_ooxml_chart | `ooxml_chart::parse_chart_xml(data)` | ooxml_chart/parser.rs:62 |

## 게이트 (nightly 불가 → 정적 확인, #3158 동일 조건)

1. **본 크레이트 무영향**: `cargo test --tests --profile release-test --no-fail-fast` 무실패
   (fuzz/ 는 [workspace] members=["."] 분리라 본 빌드 그래프 밖 — 1단계에서 성립, 재확인).
2. **진입점 시그니처 정합**: `cargo build -p rhwp` 로 `rhwp::wmf::converter::{WMFConverter,
   SVGPlayer}` 와 `rhwp::ooxml_chart::parse_chart_xml` 가 하네스가 참조하는 형태로 pub
   노출되는지 확인 (경로가 실제로 존재·pub 임을 심볼로 검증).
3. **시드 유효성**: parse_ooxml_chart 시드는 `parse_chart_xml` 이 Some 반환하는지 lib 테스트로
   간접 확인(BAR_XML 은 이미 테스트가 검증). WMF 시드는 `WMFConverter::run()` 이 첫 레코드
   파싱을 통과하는지 임시 확인 후 바이너리만 남긴다.
4. `fmt --check` / `clippy` (fuzz 타깃은 clippy 대상 아님 — 본 크레이트만).

## PR

- 브랜치 `task/3273-fuzz-wmf-chart`, `Closes #3273`, 부모 #3141 참조.
- 본문에 조건 명시: "cargo-fuzz 빌드·실행은 nightly 필요 → CI/후속 검증. 본 변경은
  하네스·시드·문서 추가로 기존 소스 0줄 수정."
- assignee=edwardkim / milestone=v1.0.0.
