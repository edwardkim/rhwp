# 오래된 이슈 10건 중 기존 해결 확인과 남은 조판 경계

2026-11-26. 기준 `32014de6e0ade1a3baea8852c02270e40b27b345`, 브랜치
`task/7777-oldest-ten-issues`. 대상은 [작업 계획](../plans/task_m100_7777_oldest_ten_issues_plan.md)에 고정했다.

## 완료한 이슈

| 이슈 | 확인한 해결 및 종료 근거 |
| --- | --- |
| [#7782](https://github.com/edwardkim/rhwp/issues/7782) | PR #7568의 통합 SHA `731de9e1b4bb946d76f35108ed7e186ebe4ebecb`가 devel에 포함됐다. 빈 문단 외곽선은 동일 원본 전 1쪽 Native/WASM 96.86991%이며, 닫힌 표 프레임을 포함한 기존 17개 검사도 통과했다. |
| [#7780](https://github.com/edwardkim/rhwp/issues/7780) | PR #9393의 통합 SHA `0fbee2c04ecee63cf526a9a7e61b38b32608bc3c`가 devel에 포함됐다. 동일 원본 전 1쪽 Native/WASM 99.99%와 양쪽 review에서 지도·소요시간·운임안내 제목의 굵은 획과 그림자를 확인했다. 원 이슈 시점 PDF에는 시간표·운임표의 오른쪽 내용이 누락됐음을 두 독립 PDF의 실제 raster에서 확인했다. |
| [#7781](https://github.com/edwardkim/rhwp/issues/7781) | PR #9393의 반복 점검표 행 이어받기 보정과 `b504b547b8`의 회귀가 devel에 포함됐다. 그때 사용한 PDF와 이번 한컴 2024 1-up PDF는 같은 쪽의 공백을 뺀 텍스트 순서가 전 10쪽 동일하며 raw 100.00%다. 보존 Native 실행 파일로 새 PDF와 전 10쪽을 직접 대조해 표시 최저 100.00%를 확인했다. 2→3쪽의 마지막 셀과 4쪽 서명·후속 본문 순서를 review PNG로 판독했다. WASM 전쪽 근거는 PR #9393의 비교를 사용했다. |

세 이슈는 GitHub API에서 `closed`, `state_reason=completed`를 확인했다.
반복 점검표의 공식 PDF는 [이번 기준 파일](../../pdf/issue7781/issue2439-repeat-table-2024.pdf)에 보존했다.
진단용 Native 실행 파일은 SHA-256 `28e77ba66de9cdf2b9e03e272e1529577d76f6447966265856d83bde2211ed2a`로
고정했다. 이 실행 파일의 정확한 Git source SHA는 확인되지 않았으므로 이번 새 빌드 증거와 구분한다.
이번 산출물과 PDF 대조 로그는 `output/pr-review/issues7777-7787/`에 있다.

## 남은 원본의 진단

- #7777: Native 진단은 47쪽, 한컴 Print는 49쪽이다. 독립 PDF 14·27쪽에는 쪽번호만 있고 다음
  규제 개요가 15·28쪽에서 시작한다. 같은 제목은 Native 14·26쪽에 있다. 해당 제목 문단 155·276은
  원본 문단 모양에 `page_break_before=true`이며 문단 헤더 쪽 나누기 비트는 0이다. 앞의 단일 행
  RowBreak 표 154·275는 저장 개체 높이 18,755·16,355 HU보다 선언 셀 높이 20,219 HU가 크다.
  396은 둘 다 20,219 HU이고 다음 제목 앞에 별도 빈 Print 쪽이 없다. 이 대조는 원인 후보를 좁힌
  결과다. 두 쪽을 임의로 삽입하거나 변경 추적 suffix를 쪽 나누기 표식으로 해석할 근거는 없다.
- #7786: 보존 원본의 Native 진단은 전 27쪽 최저 99.99%, 목차 2쪽과 뒤 표 26쪽은 표시 100.00%다.
  새 devel Native 및 fresh WASM, 목차의 번호·점끌기·표 경계와 뒤 본문 판독을 추가한다.
- #7787: 한컴 PDF의 두 표 윗변 차이는 약 15.996 px, 진단 트리는 16.0 px다. 후속 표의 가로
  포함도 유지되지만 전체 raw는 89.66243%이며, 비교 PNG는 배경표 주위 호스트 문단의 수평선과
  본문 상자의 하단 괘선에 잔차를 보인다. 외곽선에 포함할 줄간격과 배경 개체의 흐름 점유를
  분리해 확인한다. 이 입력은 [합성 HWPX](../../samples/stored-nested-content-flow/README.md)다.
- #7785: 원본 HML은 Native 진단 7.70051%다. 한컴 2020 저장 HWPX에는 장평·상대 크기 0과
  7200 HU × 7200 HU의 떠 있는 수식이 실제 저장됐다. 원본 HML에는 `RATIO`, `RELSIZE`,
  `SHAPEOBJECT`가 생략되어 있고 현재 adapter는 수식 크기를 내재 크기로 계산하고 글자처럼
  취급한다. 이미 검증된 HML 스키마 기본값과 생략된 요소의 호환 동작을 구분해 후속 판정한다.
