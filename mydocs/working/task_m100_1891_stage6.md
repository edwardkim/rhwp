# Task m100 #1891 Stage 6

## 목표

#1913 이후에도 남아 있는 HWP5 -> HWPX export -> 재파스 쪽수 팽창/수축 축을
분리해 수정한다.

Stage 5의 빈 문단 렌더 높이 일괄 보정은 HWP 원본 쪽수와 페이지 경계를 망가뜨려
버렸다. 따라서 이번 스테이지는 원본 렌더 쪽수 보정이 아니라, HWP5 원본에서
`line_segs` 가 없던 문단이 HWPX로 저장될 때 HWPX 일반 `linesegarray` 누락 문단으로
오해되는 문제만 다룬다.

## 수정 원칙

- 특정 파일명, 페이지 번호, PR/issue 번호, 임의 계수로 분기하지 않는다.
- HWP5 출처(`source_format == Hwp`)와 원본 `Paragraph.line_segs.is_empty()` 라는
  문서 상태에만 근거한다.
- HWPX export 산출물에는 “원본에 lineSeg가 없었다”는 marker lineSeg를 넣되,
  HWPX 재파스 직후에는 marker를 제거하여 HWP5 원본 로드와 같은 레이아웃 경로를 탄다.
- HWP5 -> HWPX export 산출물에는 `META-INF/rhwp-hwp5-origin` marker를 기록한다.
  이 marker가 있는 HWPX는 ZIP 컨테이너이지만 `source_format == Hwpx` 전용
  lineSeg reflow/pagination 분기에서 제외한다.
- 실제 HWPX 원본에는 적용하지 않는다.

## 원인

HWP5 원본을 HWPX로 저장하면 파일 컨테이너는 HWPX가 되지만, 문단 `lineSeg`
부재와 RowBreak/빈 문단 pagination 해석은 HWP5 원본 시멘틱을 유지해야 한다.
Stage 6 초안의 marker lineSeg만으로는 HWPX parser가 일반 HWPX 문서로 인식하여
`include_empty`, HWPX `is_hwpx_source` pagination tolerance, layout engine 분기를
적용했고, `samples/issue1891/76076_regulatory_analysis.hwpx` 는 82쪽에서 81쪽으로
줄었다.

단일 실험으로 HWPX `is_hwpx_source` 분기를 끄면 같은 파일이 82쪽으로 자기정합했다.
따라서 파일명/페이지별 보정이 아니라 HWP5-origin HWPX를 구분하는 문서 marker가
필요하다고 판단했다.

## 수정

- `HWP5_ORIGIN_HWPX_MARKER_PATH = "META-INF/rhwp-hwp5-origin"` 추가.
- HWP5 원본의 `export_hwpx_native` 경로에서 marker ZIP 엔트리를 기록한다.
- HWPX parser가 marker 엔트리를 `hwpx_aux_entries`로 보존한다.
- `DocumentCore::from_bytes`는 marker가 있는 HWPX에서 HWPX 전용 lineSeg reflow와
  HWPX paragraph normalize를 건너뛴다.
- pagination/render layout의 `is_hwpx_source` 판정은 marker가 있는 HWPX를 HWPX
  전용 분기에서 제외한다.
- HWP5 원본에서 `line_segs.is_empty()` 였던 문단은 HWPX export 시 0-height marker
  lineSeg로 materialize하고, 재파스 직후 marker를 제거해 HWP5 원본과 같은
  `line_segs.is_empty()` 상태를 유지한다.

## 검증 계획

- 4개 HWP5 샘플과 `.hwpx` 확장자 HWP5 payload 샘플에서
  `원본 parse page_count == export_hwpx_native 재파스 page_count` 를 확인한다.
- PDF 기준 페이지 수 일치 문제는 별도 stage로 유지한다.

## 검증 결과

`env CARGO_INCREMENTAL=0 cargo build --bin rhwp`

통과.

수동 8개 샘플 `export-hwpx --verify-pages`:

| 샘플 | 결과 |
| --- | --- |
| `samples/76076_regulatory_analysis.hwp` | 통과: 81쪽 |
| `samples/80168_regulatory_analysis.hwp` | 통과: 150쪽 |
| `samples/80250_regulatory_analysis.hwp` | 통과: 16쪽 |
| `samples/86712_regulatory_analysis.hwp` | 통과: 64쪽 |
| `samples/issue1891/76076_regulatory_analysis.hwpx` | 통과: 82쪽 |
| `samples/issue1891/80168_regulatory_analysis.hwpx` | 통과: 149쪽 |
| `samples/issue1891/80250_regulatory_analysis.hwpx` | 통과: 16쪽 |
| `samples/issue1891/86712_regulatory_analysis.hwpx` | 통과: 65쪽 |

회귀 테스트 추가:

- `tests/issue_1891.rs::issue_1891_hwp5_origin_hwpx_export_reparse_keeps_page_count`

`env CARGO_INCREMENTAL=0 cargo test --test issue_1891 -- --nocapture`

통과: 3 passed.

남은 범위:

- PDF 기준 원본 HWP/HWPX 렌더 쪽수 일치 문제는 아직 별도 축이다. 이번 stage는
  HWP5-origin HWPX export 산출물의 재파스 자기정합만 닫는다.
