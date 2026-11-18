# 다섯 원본의 최종 전쪽 증적과 보호 등록

이 디렉터리는 현재 브랜치에서 개선한 다섯 원본의 Native/fresh WASM 전쪽 TSV와 대표
raw·PDF PNG·review·overlay를 보존한다. 실제 이미지 생산 소스는
`e63992fb2f4fbedb9c58569232b176ae91b98331`이다. 리베이스 후 검증 소스
`efe9540bd4f1cbaddd89bf018771ef1de20389c8`의 렌더링 객체 동일성은
[재사용 감사](../rebase-08b38-rendering-reuse.json)를 따른다. 이전 출력을 최신 빌드의
새 캡처로 표시하지 않는다. 최종 필수 게이트와 원격 PR·CI·병합 완료 여부는 별도 검증 기록을 따른다.

## 전쪽 TSV

| 문서 | 쪽수 | Native 최저·전쪽 TSV | fresh WASM 최저·전쪽 TSV |
| --- | ---: | --- | --- |
| 화학물질 표시 기준 원본 (#7733) | 103 | [99.75%](scope/native/chemical/silhouette.tsv) | [99.71%](scope/wasm/chemical/silhouette.tsv) |
| 한글 문서 파일 형식 5.0 개정 1.3 (#7735) | 71 | [99.69%](scope/native/spec13/silhouette.tsv) | [99.67%](scope/wasm/spec13/silhouette.tsv) |
| 개요 탐색 패널 (#7736) | 3 | [100.00%](scope/native/outline-demo/silhouette.tsv) | [100.00%](scope/wasm/outline-demo/silhouette.tsv) |
| 가상융합산업 시행령 (#7737) | 74 | [99.74%](scope/native/decree/silhouette.tsv) | [99.73%](scope/wasm/decree/silhouette.tsv) |
| 표 셀 개요 번호 (#7738) | 1 | [100.00%](scope/native/outline-minimal/silhouette.tsv) | [100.00%](scope/wasm/outline-minimal/silhouette.tsv) |

## 판정과 실제 공급 조건

- Native·fresh WASM 각각 전 252쪽의 입력·독립 PDF·페이지 범위를 확인했다. 비교는
  print 프로필, 96DPI, 색상 경계 보정 전 2px raw 실루엣이며 문서별 compat는
  [조건 기록](protected-font-conditions.json)을 따른다.
- 사양 문서는 보존한 한컴 공식 원문 PDF를 쓴다. 다른 네 문서는 기록된 데스크톱
  한컴의 Hancom PDF Print 출력이며 웹한글기안기 또는 Microsoft PDF Print가 아니다.
- [Native 대표 27쪽](native-direct-reading.json)과 [WASM 대표 26쪽](wasm-direct-reading.json)의
  판독은 점수와 별도로 내용 소속·순서·누락·중복·겹침·기호와 간격을 대조했다. WASM의 빈
  사양 69쪽만 [디코딩 픽셀 동일성](scope-native-wasm-pixel-identity.json)으로 Native 판독을
  재사용했으며, 나머지 WASM 25쪽은 review·overlay를 직접 확인했다. 얇은 글리프·잉크
  차이는 남으므로 실루엣 100%를 픽셀 완전 동일로 주장하지 않는다.
- [정본 보호표](../../visual-protected-baseline/final.tsv)에 새 504행을 추가했다.
  기존 3,542행은 유지하며 각 페이지의 공식 두 자리 표시값은 상승·동일만 허용한다.
  [등록 영수증](../../visual-protected-baseline/updates/issues7733-7738-20261118.json)과
  [manifest](../../visual-protected-baseline/manifest.json)에 입력/PDF SHA와 실제 생산 출처를 남긴다.
- 이 특별 보호 검증은 통합 담당 collaborator·maintainer가 수행한다. 수용 당시의
  글꼴 공급 파일 SHA와 selectedPrograms를 재현하며 비공개 글꼴 바이너리는 포함하지 않는다.
  회귀 테스트의 최소 글꼴 격리와 원본 글꼴을 사용하는 Visual Sweep을 구별한다.

## 대표 이미지

각 링크는 해당 backend의 실제 raw·PDF PNG와 같은 쪽의 review·overlay다.

| 문서 | Native | fresh WASM |
| --- | --- | --- |
| 화학물질 표시 기준 원본 (#7733) 56쪽 | [raw](scope/native/chemical/raw-056.png) · [pdf](scope/native/chemical/pdf-056.png) · [review](scope/native/chemical/review-056.png) · [overlay](scope/native/chemical/overlay-056.png) | [raw](scope/wasm/chemical/raw-056.png) · [pdf](scope/wasm/chemical/pdf-056.png) · [review](scope/wasm/chemical/review-056.png) · [overlay](scope/wasm/chemical/overlay-056.png) |
| 한글 문서 파일 형식 5.0 개정 1.3 (#7735) 70쪽 | [raw](scope/native/spec13/raw-070.png) · [pdf](scope/native/spec13/pdf-070.png) · [review](scope/native/spec13/review-070.png) · [overlay](scope/native/spec13/overlay-070.png) | [raw](scope/wasm/spec13/raw-070.png) · [pdf](scope/wasm/spec13/pdf-070.png) · [review](scope/wasm/spec13/review-070.png) · [overlay](scope/wasm/spec13/overlay-070.png) |
| 개요 탐색 패널 (#7736) 3쪽 | [raw](scope/native/outline-demo/raw-003.png) · [pdf](scope/native/outline-demo/pdf-003.png) · [review](scope/native/outline-demo/review-003.png) · [overlay](scope/native/outline-demo/overlay-003.png) | [raw](scope/wasm/outline-demo/raw-003.png) · [pdf](scope/wasm/outline-demo/pdf-003.png) · [review](scope/wasm/outline-demo/review-003.png) · [overlay](scope/wasm/outline-demo/overlay-003.png) |
| 가상융합산업 시행령 (#7737) 70쪽 | [raw](scope/native/decree/raw-070.png) · [pdf](scope/native/decree/pdf-070.png) · [review](scope/native/decree/review-070.png) · [overlay](scope/native/decree/overlay-070.png) | [raw](scope/wasm/decree/raw-070.png) · [pdf](scope/wasm/decree/pdf-070.png) · [review](scope/wasm/decree/review-070.png) · [overlay](scope/wasm/decree/overlay-070.png) |
| 표 셀 개요 번호 (#7738) 1쪽 | [raw](scope/native/outline-minimal/raw-001.png) · [pdf](scope/native/outline-minimal/pdf-001.png) · [review](scope/native/outline-minimal/review-001.png) · [overlay](scope/native/outline-minimal/overlay-001.png) | [raw](scope/wasm/outline-minimal/raw-001.png) · [pdf](scope/wasm/outline-minimal/pdf-001.png) · [review](scope/wasm/outline-minimal/review-001.png) · [overlay](scope/wasm/outline-minimal/overlay-001.png) |

## 실제 Studio 소비 경로

[실제 45건의 판독 기록](actual-consumer-direct-reading.json)은 Canvas2D 15건과 CanvasKit 요청 30건을
구분한다. CanvasKit은 4건이 실제 해당 백엔드로 실행됐고, 나머지 26건은 기존 글꼴 준비 가드에
따라 Canvas2D로 대체됐다. 이 26건을 CanvasKit 통과로 세지 않는다. 디코딩 픽셀이 같은 중복
출력을 제외한 17개 review·overlay 쌍을 직접 대조했다. 화학 79쪽의 캡션 폭·글리프 차이와
시행령 13쪽의 글머리표 점 크기 차이는 기록에 남긴다.

[글꼴 목록 동등성](actual-catalog-equivalence.json)은 이전 경로의 목록과 현재 전쪽 WASM의
다섯 목록에서 실제 소비 필드·순서가 같음을 확인한 기록이다. 경로 이름만으로 신규 캡처나
새로운 글꼴 공급이라고 표시하지 않는다.

## 최종 빌드의 공통 공개 검사

새 Native 및 fresh WASM으로 실행한 [공통 4문서·8경로](fresh-public-validation.json)는
모두 통과했다. 전쪽 TSV·실제 생산 해시와 같은 쪽 raw·PDF PNG·review·overlay를 보존했고,
8쌍을 직접 판독했다. [frontend 검증](fresh-frontend-validation.json)의 단위 검사
2,814건과 TypeScript·production build·E2E manifest·링크·Chrome 옵션 검사도 통과했다.
이 검증과 앞선 재사용 이미지의 생산자를 구분한다.

[새 브라우저 E2E 두 건](fresh-browser-validation.json)도 통과했다. 초기 렌더와 로컬 글꼴 입력의
[실제 초기화 WASM 응답 4건](fresh-browser-wasm-consumption.json)이 새 빌드 해시와 일치한다.
[sample4 25·26쪽 새 빌드 대조](fresh-gray-tail-validation.json)는 Native/WASM 모두 100.00%이며
기존 판독본과 raw·PDF PNG 바이트가 같아 직접 판독을 재사용했다.
