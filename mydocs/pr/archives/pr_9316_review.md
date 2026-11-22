# PR #9316 Windows 직접 PDF 내보내기 성능 검토

## 최종 판정

**승인.** 2026년 11월 22일 메인테이너 `edwardkim`이 셀프 리뷰했다. 조달청 공고문의 Windows 출력 시간이 25.27초에서 3.94초로 줄었고, 3개 원본의 PDF 19쪽에서 수정 전후의 픽셀과 추출 텍스트가 같았다. 기준 `devel`은 `bd3cd17133`, 제품 수정은 `9e1d1cd948`, GitHub에서 검증한 제출 커밋은 `131b75a8d7258ef75638bcc0c156dacb47c46f13`이다. [PR #9316](https://github.com/edwardkim/rhwp/pull/9316) · [이슈 #9276](https://github.com/edwardkim/rhwp/issues/9276)

## 글꼴 선택과 PDF 기록 경로

Windows에서 공고문을 내보낼 때 같은 설치 글꼴을 4,386회 다시 확인하며 20.72초를 썼다. 수정판은 글꼴 카탈로그 안에서 계열·굵기·너비·기울기가 같은 조회 결과와 조회 실패를 재사용한다. 글꼴 선택 순서와 TTC의 개별 얼굴을 보존하는지 확인했다. [Windows 계측과 구현](../../working/task_m100_9276_implementation.md) · [글꼴 조회](../../../src/renderer/skia/font_lookup.rs)

일부 한양·굴림 글꼴의 유효하지 않은 GSUB 주 버전 때문에 HarfBuzz의 부분 추출이 실패해 PDF에 글꼴 전체가 들어갔다. 수정판은 이미 선택한 글리프를 PDF에 기록할 때만 해당 테이블을 제외한 정적 TrueType 프로그램을 사용한다. 글꼴별 32 MiB와 문서별 캐시 256개·64 MiB 한도를 확인했다. 공개 글꼴로 복합 문자와 한글의 윤곽 경계·전진폭·텍스트 추출을 검사했고, 부분 임베딩이 금지된 경우에는 원본 글꼴 프로그램의 전체 바이트가 보존됐다. [PDF 기록 경로](../../../src/renderer/skia/pdf_fonts.rs) · [회귀 검사](../../../tests/render_p37_direct_pdf_export.rs)

## 성능·출력 검증

- Windows 11 26H2에서 두 빌드를 번갈아 실행한 중앙값은 공고문 11쪽 25.27초 → 3.94초, 모집 안내문 7쪽 37.84초 → 3.79초였다. 공고문 PDF는 공식 v0.9.1에서 14,436,357바이트, 기준 `devel`에서 10,933,271바이트, 수정판에서 449,074바이트다. 신고 환경은 Windows 11 24H2다. [실측 조건과 세 문서의 결과](../../report/task_m100_9276_report.md)
- 1쪽 추천 서식을 다른 빌드와 분리해 버전별 네 번씩 측정했다. 중앙값은 v0.8.7 0.10초, v0.9.1 3.15초, 기준 `devel` 3.09초, 수정판 2.76초로 초기 글꼴 준비 비용이 남는다. [단독 실행 16회](../../tech/investigations/issue9276/short-form-times.tsv)
- 기본 설치 글꼴과 원본 지정 글꼴을 공급한 환경에서 각각 Windows PDF 19쪽을 대조했다. 두 환경 모두 수정 전후에 바뀐 픽셀이 0개였고 쪽별 추출 텍스트도 같았다. 원본 지정 글꼴로 독립 한컴 Print와 비교한 전체 최저 일치율은 Windows PDF 99.66%, Native PNG 99.69%, 새로 빌드한 WASM Canvas 99.60%다. 공고문 1쪽 표의 열과 수치, 모집 안내문 7쪽 제목, 추천 서식 1쪽의 항목명을 확대해 글꼴 공급에 따른 차이를 확인했다. [기본 환경](../../tech/investigations/issue9276/pages.tsv) · [원본 지정 글꼴](../../tech/investigations/issue9276/print-font-pages.tsv)
- 그라데이션 6쪽의 벡터 셰이딩과 추가 출력 프로필 33쪽의 픽셀·텍스트를 확인했다. 전체 회귀 13,948건, Native 공식 검사 4,072건, 필수 lint·빌드, 편람 383쪽 검사가 통과했다. 보호 문서 23종의 Native와 fresh WASM 각각 2,023쪽을 측정했고 공식값 4,046행의 하락·누락은 0건이었다. [검증 원장](../../tech/investigations/issue9276/validation.json) · [보호 문서 전쪽 결과](../../tech/investigations/issue9276/protected-pages.tsv)

## GitHub 검사와 병합 조건

제출 커밋에서 전체 회귀 A–E, Native Skia, lint, 프런트엔드, Render Diff, CodeQL, Adapter, Proptest와 최종 CI 정책 감사가 성공했다. GraphQL 집계는 성공 34건, 건너뜀 5건이었다. 제출 전 같은 계정의 Draft #9301과 병합을 시뮬레이션했고, 코드 검증 종료 때 최신 `devel`과의 병합 트리 `c3569681af`가 제출 트리와 같았다. [Build & Test](https://github.com/edwardkim/rhwp/actions/runs/44995009383/job/137130993167) · [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44995009361) · [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44995009340) · [Adapter](https://github.com/edwardkim/rhwp/actions/runs/44995009314) · [Proptest](https://github.com/edwardkim/rhwp/actions/runs/44995009451) · [최종 CI 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/44997319711)

작업지시자가 셀프 리뷰·병합과 이슈 종료를 승인했다. 이 기록과 오늘할일을 담은 최종 커밋의 필수 검사와 최신 `devel` 병합 가능 여부를 확인한 뒤 병합한다.

## 병합 후 결과 안내

신고자에게 병합 커밋, Windows 실측과 1쪽 초기화 비용, PDF 19쪽의 픽셀·텍스트 대조 결과를 알린다. PR에는 실제 병합 커밋의 세 문서 비교 이미지와 이슈 종료 기록을 연결한다. [공고문 1쪽](../../tech/investigations/issue9276/announce-font-detail.png) · [모집 안내문 7쪽](../../tech/investigations/issue9276/guide-print-font-control.png) · [추천 서식 1쪽](../../tech/investigations/issue9276/form-print-font-control.png)
