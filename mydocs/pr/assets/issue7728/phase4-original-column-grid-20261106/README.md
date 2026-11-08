# #7728 — 저장 글줄과 원본 열 너비의 독립 소유

`base-native-silhouette.tsv`와 `base-wasm-silhouette.tsv`는 보정 전 고정 devel `170b886f`의
실패 증거다. 그 파일의 5·6·7·9·10쪽 미달은 보정 후 모두 90% 이상이며, 코드 `04f03cfa`에서도
Native/fresh WASM 전 18쪽을 다시 확인했다. 최신 전쪽 값과 중요 회귀는
[6단계 증적](../phase6-important-regressions-20261106/README.md)에 연결한다. 아래 각 단계의
전체 회귀 실패·잔여 기록은 당시 이력이며 최종 상태는 해당 단계 이후의 검증과 구분한다.

원본 `70833-electrical-safety-rule-regulatory-analysis.hwp`와 독립 한컴2020 Print PDF는
이전 단계와 같은 바이트다. 원본·PDF·비교 문턱·기존 검사 기대값은 바꾸지 않았다.
코드 부모는 `0be6d69ee75df4eb5ec657d09b264aaae109a108`, 적용 diff SHA-256은
`0d72c82d83c4b858759a67e7c9de436c82a7ba9aed23fd3c19e92afd4248bd29`다.
Native producer (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/native-producer.json`)와 WASM producer (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/wasm-producer.json`)에 고정했다.

## 원인과 공통 보정

7쪽의 완결 중첩 표는 원본 폭이 각각23,254HU·23,537HU지만 full layout의 부모 내용 상자
맞춤 분기에서 둘 다296.21333px로 축소됐다. 부분 표 경로는 같은 원본 그리드를 쓰므로 마지막
조각에서만 열 경계와 줄바꿈이 달라졌다. 저장 글줄 결측은 원본 셀의 열 소속·열 너비를
무효화하지 않는다. 원본 overlay 배율도 두 표 모두1.0이었다.

`complete_declared_column_grid`는 모든 행의 셀 소속·병합·선언 폭 합과 편집 여부로 수평
그리드를 인증한다. full paint의 폭 맞춤 분기가 이 관계를 사용한다. 기존
`stored_complete_column_frame`은 이 관계에 원본 저장 글줄 조건을 다시 더하므로 세로 프레임·
마지막 조각 등 다른 소비자의 인증은 약화되지 않는다. 문서명·쪽수·개체 ID 예외는 넣지 않았다.

선행 폭 맞춤은 `20d990b04d`(#4042), 저장 열 프레임 보존은 PR #7877(`429dee416e6`),
물리 부모 셀 안의 원본 폭 보존은 PR #7882(`5f2897b120b`)에서 비교했다. 모두 고정 base의 조상이다.
원본 열 합과 최종 paint 폭의 실제 진단 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/source-grid-diagnostic.json`)은 FAIL→PASS다.

## 새 소스의 전쪽 검증

고정 devel `170b886f167b705b6e630c5b34bb638d92fae0a9`와 같은 입력·PDF·검증 글꼴·
96DPI·compat2022·PDFium·2px raw로 비교했다. 페이지별 전후 값 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/page-comparison.json`)은
Native/fresh WASM 각각18/18쪽, 누락0,90% 미달0, devel 대비0.1%p 초과 하락0이다.

| 출력 | 최저 raw | 최저 쪽 | 7쪽 | 10쪽 |
| --- | ---: | ---: | ---: | ---: |
| Native | 94.62153% | 6 | 94.79966% | 99.57724% |
| fresh WASM | 94.27517% | 7 | 94.27517% | 99.47384% |

[Native TSV](candidate-native-silhouette.tsv) · [WASM TSV](candidate-wasm-silhouette.tsv) ·
[base Native TSV](base-native-silhouette.tsv) · [base WASM TSV](base-wasm-silhouette.tsv).
WASM은 현재 Mac의 locked `--target web --out-dir pkg --no-opt` 로컬 대체 빌드이며
Docker 최적화 통과가 아니다. root pkg와 Studio public의 JS/WASM hash도 producer에서 일치한다.
브라우저 실제 초기화 응답 확인은 별도 후속 검증이며 이 빌드 확인으로 대신하지 않는다.

7쪽의 [보정 전 review](before-native-p007-review.png)·[overlay](before-native-p007-overlay.png),
[Native 보정 후 review](after-native-p007-review.png)·[overlay](after-native-p007-overlay.png),
[WASM 보정 후 review](after-wasm-p007-review.png)·[overlay](after-wasm-p007-overlay.png)를 직접 판독했다.
원본 그리드는 복구됐고 약간의 세로 위치·글리프 차이는 남아 있다. 100%라고 주장하지 않는다.
Native PNG 비교 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/native-png-change-scope.json`)에서 이전 단계와 같은 쪽과 실제 변경 쪽을 구분했다.
10쪽은 [Native review](after-native-p010-review.png)·[overlay](after-native-p010-overlay.png)를
직접 판독하고 독립 PDF 벡터 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/page10-independent-pdf-geometry.json`)의 표 경계를96DPI로 환산했다.

## 대조군과 필수 Native Skia

원래 폭 맞춤·클리핑 관련 대조군은 같은 입력의 base/후보 전쪽을 양쪽 backend로 재산출했다.
Native 22쪽 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/controls.json`)과 WASM 22쪽 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/controls-wasm.json`)에서 페이지별 raw는 전부 동일하다.
42065의17쪽과3194097의1쪽은90% 이상이다. #5587의4쪽 중3쪽은 base부터
Native88.23109%·WASM88.27702%이며 후보도 같다. 기존 미달 대조군을 새 통과로 집계하지 않는다.

Native Skia 필수3종 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/native-required.json`)은 현재 Mac 최소 글꼴 sandbox·jobs8·threads8·
Cargo 순차 실행이다. 라이브러리4,039PASS(루트3,857+나머지182)/0FAIL/12ignore,
누락 그림 PNG2PASS, 직접 PDF22PASS, 세 종료 코드 모두0이다. 기본 nextest나 이전 소스의
결과로 대신하지 않았다. 실행 로그는 ignored output에만 보존하고 명령·요약·hash를 정식 JSON에 기록했다.

## 회귀 검사와 잔여

새 검사 비교 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/new-regression-before-after.json`)는 동일한 네 검사로 이전 단계3PASS/1FAIL→
현재4PASS를 확인한 최소 글꼴 Native 실행이다. 실패한 검사는 실제로 축소된 열 너비를 검출했다.
4~7쪽은 원본/출력 폭과 내용 소속의 관계형이고,10쪽은 양쪽 raw98% 이상과 독립 Print를
확인한 ±2px 고정형이다. 새 suite 등록 경로 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/registered-native-focused.json`)에서도 같은 최소 글꼴 Native Skia 조건으로4PASS/0FAIL을 완료했다.

새 검사 포함 전체 nextest·범위별 lint/build·#7727/#7700 등 영향 경계·실제 소비 경로·
최종 제출 head의 편람 gate는 아직 남아 있다. 이 단계는 전체 PR 준비 완료나 원격 이슈 종료가 아니다.


## 전체 회귀 완료 뒤의 현재 상태

코드 `e37fde03642ed2a0fda68ff290337d00e7cada3f`의 범위 게이트 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/final-local-gates.json`)는
fmt·Native/기본/WASM/all-target Clippy·workspace build·manifest·unit-tier를 통과했다.
전체 nextest는12,661실행 중12,652PASS/9FAIL/45skip, 실행561.200초이며 종료코드100이다.
Native Skia3종 통과와 전체 회귀 실패를 구분한다. 실패는 #4138의 저장 조각·자식 프레임3건,
#7114의 편집/재열기 관계4건, #5846의 비용편익 라벨 소속1건,71352 글자 겹침 원장1건이다.
기존 정상 검사·기대값·허용치는 유지하며 PR 제출을 보류한다.

이전 영향 범위 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/prior-scope-summary.json`)는9조건416쪽씩 Native/fresh WASM으로 완료했다.
전832쪽의 [페이지별 값](prior-scope-page-comparison.tsv)에서90% 미달0·고정 devel 대비
0.1%p 초과 하락0이다. 직전 수용 코드와의 별도 비교 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/prior-accepted-comparison.json`)에서도
같은 문턱의 하락은0이다. CBTA의152·170쪽은 경계의4픽셀·최대RGB차4만 달라
픽셀 차이 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/cbta-png-pixel-comparison.json`)를 구분했고 raw는 같다. 이미지가 같은 쪽도 재사용으로
대체하지 않고 이번에 전쪽을 새로 산출했다.

실제 Studio 소비 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/actual-studio-consumption.json`)는 원본4·7·10쪽의 명시적 Canvas2D
screen/print6건과 지원table-core의 실제 CanvasKit4건을 구분했다. 원본의 CanvasKit12요청은
나눔바른고딕·한양신명조·휴먼고딕 준비 실패로 Canvas2D에 fallback하므로 통과가 아니다.
지원 대조군의4개 browser parity는 통과했지만 software surface의 selectedCorpus readiness는
0건이므로 해당 readiness gate 통과라고도 하지 않는다. 기존 guard는 우회하지 않았다.

실제24회 초기화의 WASM 응답은 locked artifact와 같다. JS 관측기는 Vite의 source-map에도
원문이 그대로 남는다고 가정해 `matched:false`를 기록했다. 이 기록을 보존한 채 실제 응답의
source-map과 원문을 비교했으며, 유일한 차이는 같은 WASM 자원 URL의 치환1건이었다.
그 치환만 원복한 원문 hash와 root pkg/Studio public/frozen JS hash가 모두 같다.
별도 진단 fetch로 초기화 응답을 대신하지 않았다.

프레임 회귀 단계 분리 (로컬 진단: `output/open-issues-oldest-20261105/pr-final-20261107/rebase-5077b216-20261108/temporary-asset-json/mydocs/pr/assets/issue7728/phase4-original-column-grid-20261106/frame-regression-localization.json`)에서 기존 #4138/#7114의 동일10검사는
보정 전과1단계에10PASS,2단계에9PASS/1FAIL,3·4단계에4PASS/6FAIL이었다.
71352 겹침도1단계까지0,2단계부터1이다. 열 그리드 보정만 원인으로 오인하지 않고
2·3단계의 원본 프레임 인증·내용 소유 전달을 보정한다. 이 분리 실행은 성공 수집과
검사 통과를 구분하며, 실패를 기존 미달이나 글꼴 문제로 재분류하지 않는다.


실제 paint의 대표 확인: [원본7쪽 Canvas2D print](actual-canvas2d-print-p007.png),
[지원 대조군 CanvasKit default print](control-canvaskit-default-print.png)를 직접 열어
글자·열 경계·표 내용이 표시되는 것을 확인했다. 이 두 그림은 원본 CanvasKit 통과나
전쪽 독립 PDF 수치 검증을 대신하지 않는다.
