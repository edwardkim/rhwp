# 첫 그림과 뒤 가사가 공유하는 저장 셀 프레임

원본 `samples/basic/issue1994_behindtext_table_20200830.hwp`와 독립 `pdf/semanticist21-20261008/basic-issue1994_behindtext_table_20200830-8e7a95cf5919-2020.pdf`의 전4쪽을 96DPI·2022 compat·PDFium·2px raw로 비교했다.

1쪽 아래 악보는 크기가 맞지만 그림만 셀 중앙에 다시 맞춰 506.40667px에 놓였다. 독립 Print의 그림 위끝은 499.73201px다. 첫 빈 호스트 vpos=0, 뒤 가사의 vpos=2553HU, 실제 그림 높이=2553HU로 닫힌 프레임을 공통 query가 확인한다. 그림과 뒤 가사는 같은 정렬 프레임을 소비하며, 음수 개체 offset을 비겹침 흐름의 앞 공간으로 재소비하지 않는다. 파일·쪽별 보정은 없다.

Native 1쪽 98.59756→99.86093%, fresh WASM 98.54350→99.83671%다. 2·3·4쪽 점수는 각각 이전과 같고, 전4쪽은 양쪽 모두 98% 이상이며 페이지 누락·하락은 없다. 다만 두번째 악보 아래의 `놀라운일----------`·`전파하리---------` 가사 간격 차이는 별도 보정 중이다. 98% 수치로 직접 판독한 이 결함까지 완료로 표시하지 않는다.

독립 그림 x/y를 ±2px로 보호하는 새 정식 검사는 같은 소스로 실제 1 FAIL→1 PASS다. 관련 기존 #5731·#6110·#6313·#6494 및 표 원점 검사까지 frozen Native library에 연결해 10 PASS를 확인했다. --prepare 후 현재 자동 suite를 실제로 실행해 macOS 최소 글꼴·Native Skia 환경에서 같은 10 PASS·0 FAIL을 확인했다. 미선택 1,843건은 실행 수에 포함하지 않았다. 전체 nextest와 최종 head 전쪽 검증은 아직 별도 남아 있다.

## 글꼴 조건

설치한 Haansoft Batang을 사용하는 [명시적 Print 비교 설정](reference-font-environment.json)이다. PDF가 산돌이야기B·XB 요청 부분도 실제 Haansoft Batang으로 출력했음을 리소스·글리프와 대조했고, 해당 두 이름을 기존 한겨레결체·나눔고딕 비교 설정에 추가했다. 원본 Sandoll 글꼴이 확보됐다는 뜻은 아니다. glyphFallback Wingdings도 기존 조건을 보존한다.

`preserveSourceMetrics: false`는 확인된 Print 대체 프로그램의 전진폭 조건을 재현한다. 기본 원본 측정 정책이나 제품의 전역 치환을 바꾸지 않는다. Haansoft Batang이 없는 환경에서는 다른 후보가 선택될 수 있어 이 결과를 재사용하지 않는다. 글꼴 바이너리와 PDF 추출 프로그램은 Git에 포함하지 않는다. 이전 글꼴 프로그램 감사 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/reference-fonts-20261106/font-program-evidence.json`), [한겨레결체 원본 CDN의 별도 공급 검증](../hankyoreh-cdn-20261107/README.md)을 구분한다.

전후 전쪽 TSV 4개·Native/fresh WASM의 1쪽 review/overlay 8개·원장 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/print-frame-20261107/validation.json`)을 보존했다. Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드이며 Docker 최적화 결과가 아니다.
