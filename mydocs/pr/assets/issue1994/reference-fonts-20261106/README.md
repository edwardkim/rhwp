# #1994 용지 기준 표 원점과 독립 Print 글꼴 환경

코드 HEAD `fa0436f4951db3bcafa2403c05e202cb425355fe`, base `45684640be5eddcf5972b063260250345b34d928`.
2026-11-06 Mac에서 현행 Visual Sweep의 PDFium·96DPI·2022 compat·색상 경계 보정 전2px raw를 사용했다.
원본은 `samples/basic/issue1994_behindtext_table_20200830.hwp`, 독립 기준은 기존 `samples/issue1994/issue_1994.pdf`다.
전체 입력·PDF·실행 파일·WASM·변환기 해시와 페이지별 수치는 validation.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/reference-fonts-20261106/validation.json`)에 보존한다.

| 경로 | base 3쪽 | 후보 3쪽 | 후보 전4쪽 최저 | 98% 미달 |
| --- | --- | --- | --- | --- |
| Native | 86.38551% | 98.22544% | 98.22544% | 0쪽 |
| fresh WASM | 86.10823% | 98.19505% | 98.14207% | 0쪽 |

## 비교 환경의 독립 근거

원본은 `한겨레결체`·`나눔고딕`을 요청하지만 기준 PDF의 실제 fill/stroke 본문은 Haansoft Batang이다.
PDF의 글리프 ID와 설치한 검증용 `HBATANG.TTF`의 정규화 윤곽을 비교해 전체486쌍 중471쌍이 일치했다.
15개 특수기호 차이는 font-program-evidence.json (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/reference-fonts-20261106/font-program-evidence.json`)에 남겼다. 전체 글꼴의 완전 일치라는 주장이 아니다.
독립 한컴2020 Print(11.0.0.9136, Print 방식0, job `792978dc-aeb5-475e-83e1-e896b7cdeff0`)도 같은 대체를 재현했다.
기존 PDF는 교체하지 않았다. 추출 글꼴 프로그램·윤곽 배열·원본 글꼴 바이너리는 로컬 검증에만 보존한다.

같은 [reference-font-environment.json](reference-font-environment.json)을 base/후보와 Native/WASM에 적용했다.
글꼴 공급 순서는 로컬 `HBATANG.TTF`, 사용자 Fonts 폴더이며 기존 파일 수·바이트 제한을 유지했다.
이는 기준 Print의 실제 환경을 재현하는 명시적 비교 설정이다. 문서 IR·기본 글꼴 선택 정책은 수정하지 않았다.
기본 사용자 글꼴 환경의 전4쪽 최저는 Native91.80504% / WASM92.02788%로서 이 결과와 구분한다.
글꼴 환경만 맞춘 base 3쪽은 여전히86%이므로 실제 원점 보정의 개선과 글꼴 환경의 효과를 분리한다.

## 직접 판독과 회귀

Native/fresh WASM의 3쪽 [review](candidate-native-p003-review.png)·[overlay](candidate-native-p003-overlay.png),
[WASM review](candidate-wasm-p003-review.png)·[overlay](candidate-wasm-p003-overlay.png)를 직접 열어 확인했다.
용지 기준 두 표의 바깥 괘선·좌우 원점·내부 글줄·아래 표·기존 중첩표 잘림을 비교했다.
괘선 원점이 복구됐으며 일부 기호·글리프 차이는 남아 있다. 98%는 RGB 완전 일치가 아니다.

`issue_1994_paper_overlay_outer_origin`은 원본 바깥여백을 한 번 소비하는 관계형 검사다.
`issue_1994_print_overlay_positions`는 독립 PDF의 두 표 x/y를96DPI ±2px로 지키는 고정형 검사다.
두 유형은 별도 파일이며 시스템 글꼴 설치가 없어도 실행한다. 새 고정형은 base FAIL → 후보 PASS를 확인했다.
전쪽 TSV 4개는 base/후보·Native/WASM별 원문을 그대로 보존했다. 그 밖의 브랜치 문서는 검증 중이다.
