# Task #3273 — WMF·ooxml_chart 퍼징 하네스 확장 (최종 보고서)

`Closes #3273` · 부모 RFC #3141 2단계.

## 배경

이번 주 kevin9327 기여를 보안 코딩 관점에서 분류한 결과, 파서 신뢰 경계(무검증
할당·부호확장·산술 오버플로) 결함이 WMF 파서에 집중돼 있었다(#3008 #3002 #3163 #3176).
그런데 1단계(#3158)의 fuzz 하네스는 hwp/hwp3/hwpx/hml 4개뿐이라 정작 WMF·ooxml_chart
파서는 자동 검출 범위 밖이었다. 메인테이너 코드 스캔으로 같은 결함 클래스가 아직 남아
있음을 확인했다:

- `poly_line.rs:47` / `polygon.rs:50` — `number_of_points`(i16)를
  `Vec::with_capacity(_ as usize)` 에 무검증 사용(음수 부호확장 → 과대할당 패닉). #3008
  Region scan_count 수정과 동일 클래스인데 2곳 누락.
- `bitmap16.rs:126` `calc_length()` — i16 곱셈 오버플로 미검증.

이 결함들을 손으로 고치는 대신, #3141 철학대로 **퍼저가 자동 발견하도록 하네스를 먼저
확장**했다.

## 변경 (fuzz/ 하위 5개, 기존 소스 0줄 수정)

| 파일 | 내용 |
|---|---|
| `fuzz/fuzz_targets/parse_wmf.rs` (신규) | `WMFConverter::new(data, SVGPlayer::new()).run()` |
| `fuzz/fuzz_targets/parse_ooxml_chart.rs` (신규) | `ooxml_chart::parser::parse_chart_xml(data)` |
| `fuzz/Cargo.toml` (수정) | `[[bin]]` 2개 추가 |
| `fuzz/corpus/parse_wmf/minimal_placeable.wmf` (신규) | META_PLACEABLE(magic 0x9AC6CDD7) + 최소 헤더 + META_EOF, 46바이트 합성 |
| `fuzz/corpus/parse_ooxml_chart/bar_chart.xml` (신규) | 최소 유효 `c:chartSpace` 막대 차트 |
| `fuzz/README.md` (수정) | 하네스 표·run 명령·코퍼스 표·로드맵 갱신 |

## 진입점 정합 확인

- WMF: `convert_wmf_to_svg`(svg.rs:3308)가 `pub(crate)` 라 fuzz 크레이트에서 접근 불가 →
  그 내부 조합(`WMFConverter::new(&[u8], SVGPlayer::new()).run()`)을 하네스에 직접 폈다.
  두 타입 모두 `pub`, `&[u8]` 는 `embedded_io::Read`(= `wmf::Read`) 구현.
- ooxml_chart: `parse_chart_xml` 은 `ooxml_chart::mod` 에서 재노출되지 않아
  `rhwp::ooxml_chart::parser::parse_chart_xml` 경로로 참조(계획 단계에서 정정).

## 검증 (nightly 불가 → 정적 확인, #3158 동일 조건)

- **진입점 시그니처 정합**: `cargo build -p rhwp` 오류 0 — 하네스가 참조하는 pub 경로가
  실제 존재.
- **시드 유효성 실증**: 임시 스모크 테스트로 WMF 시드가 `WMFConverter::run()` 을 패닉 없이
  통과(=하네스 진입점 조합이 컴파일·실행 가능함을 동시 입증)하고, 차트 시드가
  `parse_chart_xml` 에서 Some 을 반환함을 확인한 뒤 테스트를 제거했다.
- **본 크레이트 무영향**: 전체 `cargo test --tests --profile release-test --no-fail-fast`
  무실패 — `fuzz/` 는 `[workspace] members=["."]` 분리라 본 빌드 그래프 밖.
- 변경 파일이 `fuzz/` + working 문서로 한정 — 기존 소스 0줄 수정.

**cargo-fuzz 빌드·실제 퍼징은 nightly 도구체인이 필요해 로컬 재실증 범위 밖이다. CI/후속
단계에서 확인한다**(#3158과 동일 조건).

## 후속

- 퍼징 실행 시 위 남은 결함(poly_line/polygon number_of_points, bitmap16 calc_length)이
  자동 재현될 것으로 기대 — 재현 입력이 나오면 트리아지 절차(README §트리아지)대로 이슈
  등록 후 수정.
- #3141 로드맵 잔여: EMF 등 내부 파서 직접 하네스, CI 통합, OSS-Fuzz 등재.
