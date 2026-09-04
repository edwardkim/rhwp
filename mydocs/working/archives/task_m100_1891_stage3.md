# Task m100 #1891 Stage 3

## 기준 브랜치

- 기준: `review/pr1912-1913-cherrypick`
- 포함된 선행 PR:
  - #1912: tac 그림 문단 뒤 lazy vpos 재역산의 trailing line spacing 이중 가산 수정
  - #1913: 외부 참조 BinData Link 그림 HWPX 왕복 소실 수정 및 #1891 군집 판별

## 백업 작업과의 관계

백업 브랜치 `task/m100-1891-hwpx-roundtrip-format-preflight` 의 작업은 #1891 중
HWP5-in-.hwpx / HWP5 -> HWPX 변환 축을 다룬다. #1913 은 같은 이슈 번호를 사용하지만,
진짜 HWPX 외부 참조 BinData Link 그림 소실 축만 해결한다.

직접 충돌 지점은 `tests/issue_1891.rs` 1개다.

- #1913 의 `tests/issue_1891.rs`: 외부 Link BinData/pic 보존 테스트
- 백업 작업의 `tests/issue_1891.rs`: HWP5 payload 의 HWPX export/reparse 페이지 수 보존 테스트

따라서 후속 작업에서는 백업 테스트를 `tests/issue_1891_hwp5_disguised_pages.rs` 같은
별도 파일로 분리하고, #1913 테스트 파일은 유지한다.

## 후속 통과 기준

단순 roundtrip 페이지 수 보존만으로는 통과가 아니다. 사용자가 지정한 기준은 다음 셋이
모두 같은 페이지 수를 내는 것이다.

- 한컴 2024 기준 PDF
- rhwp 원본 HWP/HWPX 로드 결과
- rhwp HWP5 -> HWPX export 후 재파스 결과

## #1913 이후 현재 기준선

`output/poc/task1891-after1913/page-baseline/page_counts.tsv`

| sample | pdf_pages | hwp_src | hwp_rt | hwpxname_src | hwpxname_rt |
| --- | ---: | ---: | ---: | ---: | ---: |
| 76076 | 82 | 81 | 87 | 82 | 88 |
| 80168 | 157 | 150 | 172 | 149 | 173 |
| 80250 | 17 | 16 | 17 | 16 | 19 |
| 86712 | 65 | 64 | 71 | 65 | 72 |

## 판단

#1913 이후에도 HWP5-in-.hwpx 축은 남아 있다.

- 원본 rhwp 로드 자체가 PDF 기준보다 1~8쪽 부족한 샘플이 있다.
- HWP5 -> HWPX 재파스는 대체로 쪽수가 팽창한다.
- 백업 구현의 0-height placeholder 방식은 일부 roundtrip 팽창을 줄였지만, PDF 기준까지
  맞추는 최종 해법은 아니므로 그대로 PR 대상이 아니다.

## 다음 작업

- 백업 코드에서 재사용 가능한 부분은 HWP5 원본 `line_segs` 부재 의미를 보존하는 marker
  아이디어와 샘플/기준 PDF뿐이다.
- 다음 스테이지에서는 PDF 기준보다 원본 로드가 부족한 이유를 먼저 분석한다.
- 보정 근거는 샘플명/페이지번호가 아니라 HWP5 원본의 `LineSeg`, `ParaShape`, 표/셀 속성,
  section/page 속성 중 읽을 수 있는 값이어야 한다.
