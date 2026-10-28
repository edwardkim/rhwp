# Task m100 #1939 Stage 1

## 목표

`samples/issue1891/76076_regulatory_analysis.hwpx` 는 확장자만 HWPX인 HWP5(OLE/CFB)
샘플이다. #1936 이후 페이지 수는 82쪽으로 안정화되었지만,
`render-diff --via hwpx` strict 비교에서 page 38/39 구조 불일치가 남는다.

## 재현

```bash
target/debug/rhwp render-diff samples/issue1891/76076_regulatory_analysis.hwpx --via hwpx
```

현재 결과:

- 페이지 수: A=82 B=82
- 최대 변위: 642.53px (page 39)
- 구조 불일치 페이지: 2
- status: `STRUCT_MISMATCH`

## 진행 원칙

- 특정 샘플명, 페이지 번호, 이슈 번호, 임의 계수로 결과를 맞추지 않는다.
- 보정은 입력 문서에서 읽을 수 있는 `LineSeg`, `ParaShape`, `CharShape`, 표/셀 속성,
  control 속성, section/page 속성 또는 공개 스펙 필드에 근거한다.
- 우선 원본 HWP5-in-.hwpx와 export HWPX 재파스 결과의 page 38/39 render tree 차이를 비교해
  어떤 문서 속성이 왕복 중 바뀌는지 확인한다.

## 분석

- page 38/39 경계에서 원본은 `pi=370..374`와 `pi=375`의 첫 RowBreak 조각이 page 38에
  남지만, roundtrip HWPX 재파스 결과는 `pi=370` 이후가 page 39로 밀렸다.
- 해당 구간의 원본 HWP5 문단은 TAC 표 host 문단이면서 `line_segs`가 비어 있다.
- HWP5-origin HWPX export는 원본 `LineSeg` 부재 의미를 보존하기 위해
  `LineSeg::missing_lineseg_placeholder()`를 출력한다.
- HWPX 로드 중 TAC 표 높이 보정이 이 placeholder의 `line_height`를 표 높이로 바꾸면서
  `clear_missing_lineseg_placeholders()` 제거 조건을 통과하지 못했다.
- 그 결과 roundtrip 문서만 권위 있는 `LineSeg`를 가진 것으로 처리되어 표 배치 경계가 바뀌었다.

## 수정

- `DocumentCore::reflow_zero_height_paragraphs()`의 TAC 표 높이 보정에서
  `missing_lineseg_placeholder` 단일 문단은 보정 대상에서 제외했다.
- 이 marker는 reflow gate 전용이며, 이후 제거되어 HWP5 원본과 같은
  `line_segs.is_empty()` 경로를 타야 한다.

## 검증

```bash
env CARGO_INCREMENTAL=0 cargo build --bin rhwp
target/debug/rhwp render-diff samples/issue1891/76076_regulatory_analysis.hwpx --via hwpx
target/debug/rhwp export-hwpx samples/issue1891/76076_regulatory_analysis.hwpx tmp/issue1939/after/76076-roundtrip.hwpx --verify-pages
```

결과:

- `render-diff --via hwpx`: `페이지 수 A=82 B=82`, `구조 불일치 페이지: 0`, `status: PASS`
- `export-hwpx --verify-pages`: 82쪽 검증 통과

회귀 테스트:

- `tests/issue_1939.rs` 추가
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1939`: 통과
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1891`: 통과
- `env CARGO_INCREMENTAL=0 cargo clippy --test issue_1939 -- -D warnings`: 통과
- `git diff --check`: 통과
