# PR #3086 검토 — 가운뎃점 PDF 텍스트 추출 복구

| 항목 | 내용 |
| --- | --- |
| 원 PR | [#3086](https://github.com/edwardkim/rhwp/pull/3086) |
| 작성자 | planet6897 |
| 관련 이슈 | [#3085](https://github.com/edwardkim/rhwp/issues/3085) |
| base / 규모 | devel / 3 files, +24 -0 |
| 문서 작성 시점 참고값 | 원 PR head d2ba1a49851f56a91ba825db25f73dd1949286f1, BEHIND 및 MERGEABLE. 이전 head의 Build & Test, CodeQL, Canvas visual diff는 성공으로 확인했다. |
| 통합 적용 | 최신 upstream/devel 866925fa6 위 적용 commit f33576ececad884371864d224d3bfd32f372d0cf |

## 관련 이슈와 변경 범위

- #3085는 가운뎃점을 벡터 원으로 직접 그리는 기존 경로 때문에 PDF 텍스트 스트림에서 U+00B7이 사라지는 문제를 다룬다.
- 보이는 원은 그대로 유지하고 같은 위치에 fill-opacity 0의 텍스트를 병기해 검색, 복사, 스크린리더 추출을 복구한다.
- SVG 출력 경로와 issue-147 aift golden만 변경한다. 글자 배치나 벡터 원 geometry는 바꾸지 않는다.

## 렌더 영향과 검증

- 시각 외형은 변하지 않아야 하지만 SVG/PDF 접근성 결과가 바뀌므로 visual 및 추출 검증 대상이다.
- 최신 통합 브랜치에서 cargo test --test svg_snapshot을 실행해 aift page 3을 포함한 8건이 통과했다.
- samples/aift.hwp의 3쪽을 PDF로 내보내고 pdftotext로 확인했다. 추출 U+00B7 수는 7개이며, issue-147 golden의 fill-opacity 0 가운뎃점 7개와 일치했다.
- Clippy, doctest, release 전체 검증은 원격 통합 PR의 최신 head CI에서 최종 확인한다.

## 리스크와 판단

- 투명 텍스트가 PDF 텍스트 추출기에 포함되는지 구현별 차이가 날 수 있다. 실제 rhwp PDF 내보내기와 pdftotext의 7개 추출로 목표 동작을 확인했다.
- 투명 텍스트는 보이는 벡터 원 위에만 추가되며, snapshot으로 외형 회귀를 감시한다.

## 최종 권고

- planet6897 렌더 4건 통합 PR에 포함해 수용 권고.
- 최종 merge 조건은 통합 PR 최신 head의 GitHub Actions 통과와 작업지시자 승인이다.
