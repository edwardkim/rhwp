# Task 1440 최종 보고서

## 이슈

- GitHub Issue: https://github.com/edwardkim/rhwp/issues/1440
- 제목: `온새미로 p35: 그림 주위 본문 감싸기 레이아웃 불일치`
- 브랜치: `local/task_m100_1440`
- 기준 브랜치: `upstream/devel`
- 작업 모드: 기여자 모드. 오늘할일 문서는 생성/갱신하지 않았다.

## 완료 내용

1. 35쪽 그림 어울림 본문 침범 보정
   - HWP/HWPX 원본의 35쪽 대상 문단은 앞 7줄에 `column_start=850`, `segment_width=20999` LineSeg wrap zone을 저장하고 있었다.
   - 기존 렌더는 후속 body 문단에 저장된 precomputed wrap zone을 적용하지 않아 본문이 우측 상단 그림 영역을 침범했다.
   - 셀 내부가 아닌 body 문단에서 같은 문단 안에 좁은 LineSeg 폭과 일반 LineSeg 폭이 섞인 경우에만 LineSeg `column_start/segment_width`를 적용하도록 보정했다.

2. 6쪽 문단 테두리 박스 회귀 보정
   - 35쪽 보정이 LineSeg `cs/sw`만으로 판정되면 6쪽 지문 박스와 #547 passage box를 그림 어울림으로 오인할 수 있음을 확인했다.
   - anchor 메타데이터가 없는 fallback 보정은 문단 내부에 실제 혼합 폭 LineSeg가 있는 precomputed picture-wrap 흐름으로 제한했다.
   - 6쪽 지문 박스의 점선 테두리가 `Rectangle` 최적화 과정에서 실선으로 바뀌지 않도록, 비실선 문단 테두리는 면별 `LineNode`로 렌더하도록 했다.

3. 문단 테두리 연결 속성 보존
   - `ParaShape.attr1 bit 28`을 `문단 테두리 연결`, bit 29를 `문단 여백 무시`로 정리했다.
   - HWPX `<hh:border connect ignoreMargin>` 파싱/수정/저장 경로와 Studio 문단 모양 다이얼로그 표시를 연결했다.
   - 관련 기술 문서를 갱신했다.

## 시각 검증 자료

- 35쪽 한컴 PDF raster: `mydocs/report/assets/task_m100_1440/hancom_pdf/page-35.png`
- 35쪽 rhwp HWP render: `mydocs/report/assets/task_m100_1440/rhwp_hwp_p35.png`
- 35쪽 rhwp HWPX render: `mydocs/report/assets/task_m100_1440/rhwp_hwpx_p35.png`
- 6쪽 한컴 PDF raster: `mydocs/report/assets/task_m100_1440/stage2/hancom_pdf/page-06.png`
- 6쪽 rhwp HWP render: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwp_p06.png`
- 6쪽 rhwp HWPX render: `mydocs/report/assets/task_m100_1440/stage2/rhwp_hwpx_p06.png`

## 회귀 테스트

- `tests/issue_1440_onsamiro_picture_wrap.rs`
  - 35쪽 대상 그림 bbox와 본문 TextLine bbox가 겹치지 않음을 검증한다.
  - 35쪽 원본 문단의 LineSeg wrap zone 보존을 검증한다.
  - 6쪽 문단 테두리 박스에 LineSeg `column_start`가 이중 적용되지 않음을 검증한다.
  - 6쪽 문단 테두리 연결 bit와 비실선 dash 렌더 보존을 검증한다.

## 검증

- `cargo build --release` 통과.
- `cargo test --release --lib` 통과. (`1842 passed; 0 failed; 6 ignored`)
- `cargo test --profile release-test --tests` 통과.
- `cargo fmt --check` 통과.
- `cargo clippy --all-targets -- -D warnings` 통과.
- `npm --prefix rhwp-studio run build` 통과.
- `cargo test --test issue_1440_onsamiro_picture_wrap` 통과. (`4 passed`)
- `cargo test --release --lib renderer::layout::integration_tests::tests::test_547_passage_text_inset_match_pdf_p4` 통과.

## 남은 판단

- 현재 범위에서는 35쪽 그림 어울림과 6쪽 문단 테두리 박스 회귀를 함께 보정했다.
- 일반 LineSeg 재해석이나 전체 문단 배치 재튜닝은 이번 이슈 범위 밖으로 남겼다.
