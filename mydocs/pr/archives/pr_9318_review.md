# PR #9318 표 조판·WMF 렌더링 통합 검토

## 판정

제출 head `93482b2d0069cfa7648c5f08df95ddc2eee9c179`의 변경 범위, 로컬 검증과 GitHub Actions를 검토하고 병합을 승인했다.
검토 기록 커밋은 별도 필수 검사와 최신 devel 병합 가능 여부 확인을 거친다.
[PR #9318](https://github.com/edwardkim/rhwp/pull/9318)

## 원 PR과 메인터너 보정

| 원 PR | 검증한 변경 | 개별 검토 |
| --- | --- | --- |
| #9240 | 이어받은 표 조각의 셀 배경과 원 HFT가 없는 조건의 글꼴 대체 | [리뷰](pr_9240_review.md) |
| #9269 | 떠 있는 캡션 표·한 행 표의 쪽 소속, 굴림체 작은 네모와 OLE·EMF 원점 | [리뷰](pr_9269_review.md) |
| #9278 | 마지막 표 조각 뒤 빈 줄과 후속 표의 페이지 보존 | [리뷰](pr_9278_review.md) |
| #9282 | 같은 문단에서 표·도형 줄 소속과 문단 뒤 간격 | [리뷰](pr_9282_review.md) |
| #9287 | 색상표가 생략된 WMF 1비트 DIB와 다음 레코드 | [리뷰](pr_9287_review.md) |

#9316을 포함한 `8a196ca7aea2aa3cf32d394f5b7da2bdf4520479`에서 표·그림 및 글꼴 공급의 독립 Print 대조를 수행했다.
합성 글꼴 두 종과 전용 검사는 통합 이력에서 제거했다. 실제 글꼴의 기호 윤곽과 저장 원본의
sample10 474쪽을 다시 대조했다. 편람 HWPX Native 383쪽의 동등한 정본 보호 결과를 편람
검사 증적으로 사용할 수 있도록 안내서를 갱신했다. 이번 실행에서는 전체 보호 문서와 별도
편람 명령을 모두 마쳤다. [리베이스·글꼴 기록](../assets/planet6897_9240_9282_20261121/after-9316/README.md)

## 로컬 검증

- 최소 글꼴 환경에서 전체 nextest 13,951건이 통과했고 45건이 제외됐다.
  Native Skia 라이브러리 4,032건, 자리표시자 2건, 직접 PDF 37건도 통과했다. [Rust 검증](../assets/planet6897_9240_9282_20261121/after-9316/rust-validation.json)
- 원 PR 주장과 보조 문서의 Native·fresh WASM 334쪽이 98% 이상이며 이전 후보 대비 하락은 없다.
  보호 문서 46조건·4,046쪽에서 정본 대비 누락·하락은 0쪽이다. [주장 TSV·PNG](../assets/planet6897_9240_9282_20261121/after-9316/claim-validation.json) · [보호 TSV](../assets/planet6897_9240_9282_20261121/after-9316/protected-pages.tsv)
- 실제 Studio 화면·인쇄의 WASM 초기화 응답 24건에서 새 빌드를 확인했다. Canvas2D 20건과
  CanvasKit 4건이 실제로 선택됐다. [Studio 경로](../assets/planet6897_9240_9282_20261121/after-9316/studio-consumer.json) · [편람·기여자 검사](../assets/planet6897_9240_9282_20261121/after-9316/formal-gates.json)

## 제출 코드의 GitHub CI

| 실행 | 확인 결과 |
| --- | --- |
| Adapter inter-diff | [success](https://github.com/edwardkim/rhwp/actions/runs/45011510726) |
| CI | [success](https://github.com/edwardkim/rhwp/actions/runs/45011510672) |
| CodeQL | [success](https://github.com/edwardkim/rhwp/actions/runs/45011510737) |
| Proptest roundtrip | [success](https://github.com/edwardkim/rhwp/actions/runs/45011510781) |
| Render Diff | [success](https://github.com/edwardkim/rhwp/actions/runs/45011510740) |

## 후속 처리

최종 head 필수 검사 및 최신 devel과의 병합 가능 상태를 확인하고 병합한다. 원 PR 5건에는
각 문서의 실제 review·overlay를 병합 SHA로 고정해 결과를 남긴 뒤 종료한다.
