# #8413 곡선·회전·마스크 검증

진행 기록과 최종 판정은 [공통 작업 기록](../../../../working/task_m100_8413_implementation.md)에 있다.
원본 PDF를 독립 PDFium으로 96dpi 출력하여 가져온 문서의 Native/fresh WASM과 비교한다.
TSV는 색상 경계 보정 전 2px `silhouette_raw_match_percent`를 전쪽 기록한다.
원본별 전체 쪽수는 32·37·57·1쪽이며 합성 대조군은 13쪽이다.

이 폴더에는 점수와 공개 합성 대조군의 PNG만 보존한다. 사용자가 제공한 PDF와 해당
화면, 원본에서 추출한 비교용 글꼴은 로컬 `output/pdf-import-errors-20261031/`에 두며
배포 자원에 포함하지 않는다. [manifest.json](manifest.json)에 source PDF·제품 소스·Native/WASM 산출물의 SHA-256, 전쪽 점수와 시간·메모리 관측값을 기록했다.

- `controls-*`: [직접 작성한 합성 PDF](../../../../../pdf/issue8413/README.md).
- `consecutive`, `compound`, `curve`: 기존 #8174 원본 5·1·1쪽. 거부하던 두 경로의 지원 검증.
- `hancom-palette-final`: 기존 한컴 2024 Print 색상 대조군 1쪽. PDF의 색 보존을 수정해도
  일반 HWP의 기존 색 처리와 출력이 바뀌지 않는지 확인한다.
- `img-clipping-color-final`, `img-bug-color-final`, `encrypted-final`, `f87pro-final`: 사용자 원본 전쪽.

원본 글꼴을 포함하지 않은 Helvetica 합성 입력은 명시적 Arial 글꼴 환경으로 비교한다.
37·57쪽 원본은 원본에서 복원한 로컬 글꼴 프로그램을 명시한다. Helvetica의 대체 글꼴은 Arial Narrow와 혼동하지 않도록 표준 Arial을 별도 지정한다.
폰트 프로그램 자체를 검사하는 회귀가 아니므로 자동화 검사는 설치 글꼴에 의존하지 않는다.
