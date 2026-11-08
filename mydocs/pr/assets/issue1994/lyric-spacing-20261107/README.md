# 가사와 반복 하이픈의 자연 전진 보존

Issue: #1994. `issue1994_behindtext_table_20200830.hwp` 1쪽의
`놀라운일----------`, `전파하리---------`를 독립 한컴 Print와 비교했다.
그림에 포함된 가사가 아니라 실제 텍스트다. 두 구간의 문자 모양 226은 9.5pt,
장평 90%, 자간 +40%, 굵게이며, 원본 셀과 문자 소유는 변경하지 않았다.

## 원인과 공통 보정

Task #352의 `61db5d61e7ff3e9d4fc363b3efd33ce99ef9c885`는 반복 하이픈의
추정 폭을 0.3em으로 제한했다. 원 해결의 `exam_eng.hwp` 5쪽을 확인했고,
기존 추정 경로를 유지했다. 그러나 호출자가 실제 글꼴 프로그램의 자연 전진을
선택한 경우에도 이 제한이 적용되어 셀 맞춤 전에 전체 폭을 과소평가했다.
그 결과 한글 간격은 넓고 하이픈 구간은 짧았다.

공통 문자 측정에서 자연 전진 보존이 선택되어 있고 실제 embedded metric의
문자 hit가 확인된 경우에만 그 프로그램의 반복 하이픈 전진을 그대로 사용한다.
문서명·쪽 번호·특정 글꼴명으로 분기하지 않는다. 기존 leader 추정과 미확인 글꼴의
결측 경로는 유지하며, trace도 실제 제한 적용 여부를 기록한다.

## 실제 출력과 검사

동일한 [명시적 Print 글꼴 환경](../print-frame-20261107/reference-font-environment.json),
원본·독립 PDF, canonical Visual Sweep의 PDFium·96DPI·compat2022·2px raw 기준으로
Native와 fresh WASM의 전 4쪽을 다시 산출했다. 원본 산돌 글꼴을 확보했다는 뜻은 아니며,
독립 PDF가 사용한 Haansoft Batang 프로그램에 대응하는 비교 조건이다.

| 쪽 | Native 수정 전 | Native 수정 후 | fresh WASM 수정 전 | fresh WASM 수정 후 |
| --- | ---: | ---: | ---: | ---: |
| 1 | 99.86093% | 99.96356% | 99.83671% | 99.96269% |
| 2 | 99.72650% | 99.72650% | 99.73012% | 99.73012% |
| 3 | 98.75606% | 98.75606% | 98.62409% | 98.62409% |
| 4 | 99.94357% | 99.94357% | 99.83503% | 99.83503% |

쪽수 불일치·측정 누락·raw 하락은 없다. Native와 fresh WASM 1쪽의 review·overlay를
직접 판독했으며, 가사의 시작·문자 간격과 하이픈 길이가 개선됐다. 글리프 가장자리와
농도 차이가 일부 남아 있다. Mac WASM은 locked wrapper의 `--no-opt` 로컬 대체 빌드이며
Docker 최적화 빌드 통과를 뜻하지 않는다.

- [Native 수정 전 TSV](before-native.tsv) · [수정 후 TSV](after-native.tsv)
- [fresh WASM 수정 전 TSV](before-wasm.tsv) · [수정 후 TSV](after-wasm.tsv)
- Native 수정 전 [review](before-native-p001-review.png) · [overlay](before-native-p001-overlay.png)
- Native 수정 후 [review](after-native-p001-review.png) · [overlay](after-native-p001-overlay.png)
- fresh WASM 수정 전 [review](before-wasm-p001-review.png) · [overlay](before-wasm-p001-overlay.png)
- fresh WASM 수정 후 [review](after-wasm-p001-review.png) · [overlay](after-wasm-p001-overlay.png)

독립 PDF의 첫 네 한글 문자 원점을 ±2px로 보호하는 고정형 2건은 실제 수정 전
2 FAIL에서 수정 후 2 PASS로 바뀌었다. 자연 전진·기존 leader·미확인 글꼴 경로를
구분하는 관계형 3건은 2 PASS·1 FAIL에서 3 PASS로 바뀌었다. 고정형과 관계형은
별도 검사 파일에 둔다. `--prepare` 뒤 최소 글꼴 macOS Native Skia 자동 suite에서
관련 44건을 실제 실행해 44 PASS·0 FAIL을 확인했다. 선택하지 않은 2,141건은
전체 회귀 통과로 세지 않는다. 앞서 한겨레결체 공급을 등록하면서 누락한 원장 검사의
활성 규칙 수와 추가 순서도 기록된 change set에 맞춰 보완했다. 의미·투영 순서 비교는 유지한다.

원 Task #352의 실제 5쪽 렌더 트리와 모든 문자 전진도
수정 전후 138,919바이트 및 SHA가 동일하다. 이 대조는 해당 원 동작 보존이며
그 문서의 전쪽 PDF 수용을 주장하지 않는다.

자동 suite 검사, 입력·PDF·프로필·producer 해시와 실행 로그 위치는
검증 원장 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue1994/lyric-spacing-20261107/validation.json`)에 기록한다. 최종 브랜치 전체 회귀·편람 전쪽·Studio 실제
소비 검증은 이 집중 검증과 별도로 완료해야 한다. 글꼴 바이너리·임시 스크립트·로그는
Git에서 제외된 output에만 보관한다.
