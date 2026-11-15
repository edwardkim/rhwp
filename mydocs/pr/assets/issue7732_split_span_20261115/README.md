# #7732 분할 셀 정렬 후속 보정의 최신 검증 원장

이 PR 범위는 #7732 완료 뒤 확인된 각주 37쪽의 분할 병합 셀 정렬과,
동일한 독립 Print 비교에서 확인한 표시 em·SVG 가는 선·표 바깥여백의 공통 소비 보정이다.
#7445 전체 해결을 주장하지 않는다.

제품 생산 소스는 `2749e491e843ec851de12467751ec200ab551014`다. 이후 base
`67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` 위로 리베이스했으며,
[내용 신원 감사](rebase-67b4-20261116/source-read-path-reuse.json)로 Rust·Cargo·fixture·기존 출력
runtime이 같은 것을 확인했다. asset을 담은 커밋과 실제 실행 파일을 생산한 소스를 구분한다.

## Native SVG CLI의 후속 준비 보정

`8472674645`는 Native Skia 지원 CLI의 layer SVG에 PNG와 같은 원본 프로그램 준비를
연결했다. [후속 읽기 경로 감사](native-svg-source-program/followup-read-path-reuse.json)에서
공통 조판·PNG·WASM·입력·실행기 코드가273 생산 소스와 같은 것을 확인했다. 위의273 생산
표기는 이 PNG/WASM 증거의 출처이며 새 Native SVG CLI의 바이너리 출처는847이다.
리베이스로 유입한 About UI·dialog CSS 변경은 별도로 기록하고 실제 UI 검증을 재사용한다.
Studio 전체가273과 동일하다고 주장하지 않는다.

Native SVG 7·28쪽의 전진 차이를 공개 합성 TTF로 검출해1FAIL→1PASS를 확인했다.
fresh WASM SVG 전50쪽은 최저99.16%, 새 Native SVG 전50쪽은 최저99.08%로 통과했다.
Native SVG의 7쪽은94.37%→100.00%, 28쪽은97.19%→99.23%다.
[전쪽 TSV와 대표 이미지](native-svg-source-program/after-native-svg/validation.json)를 따른다.
추가 변경 뒤 최소 글꼴 전체 nextest도 새로 실행해13,617PASS·0FAIL·45SKIP을 확인했다
(실제 검사561.270초). Native Skia all-target Clippy 및 필수 Native/WASM lint도 통과했다.
[현재 회귀](native-svg-source-program/current-validation/results.json)와
[현재 lint](native-svg-source-program/current-validation/push-lints.json)를 이전 실행과 구분한다.

실제 Studio 8쪽은 screen/print 48캡처와 별도 SVG Print 8캡처를 완료했다.
독립 PDF Print 32쌍의 수치는 모두98% 이상이며 최저는 각주37쪽 Canvas2D99.08%다.
**CanvasKit 직접 출력은 각주3쪽의 두 모드뿐이다.** 중첩5쪽은 기존 전쪽 preflight 미지원
연산과 지정 자원 미준비로 Canvas2D에 전환됐고, 전환 계약20건을 별도로 확인했다.
이를 CanvasKit 통과로 세지 않으며 지원 판정·작업 한도·fallback guard를 우회하지 않았다.
엄격한 backend 간 픽셀 비교 reportOnly12건의 FAIL도 별도 기록하며, 독립 PDF의 canonical
2px 판정과 섞지 않는다. [실행·신원·지원 범위](actual-current-2749/validation.json)와
[직접 판독](actual-current-2749/direct-reading.json)을 따른다.

## 현재 실제 결과

| 범위 | 확인한 결과 | 증거 |
| --- | --- | --- |
| 최소 글꼴 Rust 검증 | 13개 순차 게이트 모두 PASS. 전체 nextest 13,617 PASS·0 FAIL·45 SKIP, 실행 844.891초 | [게이트](validation-2749/rust-gates.json), [전체 회귀](validation-2749/full-nextest.json) |
| 중첩 표의 기존 기능 보존 | Native/fresh WASM 각각 전17쪽 보호값 하락0, 최저99.86% | [원인·독립 Print·TSV·대표 이미지](outer-margin-stage/README.md) |
| 리베이스 유입 기능 | suite 정책·CI 분류·Python109건·About 실제 브라우저·문서 링크 PASS | [새 base 검사](rebase-67b4-20261116/incoming-validation.json) |
| 공개 4문서 공통 검사 | Native/fresh WASM 8건 모두 공식 두 자리100.00%. 반올림 전 crop99.99756% | [전쪽 TSV·각 review/overlay](rebase-67b4-20261116/public-four-validation.json) |
| sample10 기호 공급 정합 | 474쪽 양쪽100.00%, Native/fresh WASM 각각 전763쪽 보호 하락0·최저99.57% | [공급 분석](sample10-symbol-supply/cause.md), [Native 전쪽](sample10-symbol-supply/native-full-results.json) |

[정본 전체 보호 결과](final-2749-67b4/protected-validation.json)는 36데이터셋·3,542행 모두
보호 하락0이며 14행이 상승했다. 각 데이터셋의 전체 TSV를 같은 실행 단위로 보존했다.
각주 원본 전50쪽 Native/fresh WASM PNG·SVG와 실제 Studio 지원 경로 검증을 완료했다.
명시적 `--handbook-gate` Native383쪽도 새847 실행 파일에서 통과했다
([전쪽 TSV·gate](handbook-current-847/gate.json)). 상승이 있는 중첩 표의 Native/WASM 각 전17쪽만
정본에 반영해14행을 상향했으며 다른34데이터셋 출처는 유지했다
([갱신 영수증](../visual-protected-baseline/updates/issue7732-20261116.json)).
로컬 준비와 PR #9106의 code head `5de0973ab1` 원격 CI를 완료했다.
[원격 CI 원장](code-ci-9106.json)에 CI·CodeQL·Render Diff·Proptest·Adapter의 success를 기록했다.
[최종 리뷰](../../archives/pr_9106_review.md)·오늘할일을 문서 trailing으로 추가하며, 해당 head 검사와 병합은 별도 확인한다.
실행 전·진행 중·실패한 기록을 완료 결과로 세지 않는다. 완료한 게이트는 소스와 입력의 신원을
확인해 재사용하며 문서 커밋만으로 같은 Rust 빌드를 반복하지 않는다.

## 독립 기준과 실제 공급

각주·중첩 표의 기준은 원본에 대응한 **한컴오피스 2020 Print** PDF다. 웹한글기안기 PDF를
정답으로 사용하지 않는다. 바깥여백 진단의 original/top0/right0 PDF는 같은 엔진
11.0.0.9136의 독립 인쇄이며 기존 기준을 대체하지 않는다.

보호 문서 Visual Sweep은 실제 원본 글꼴을 공급한 일반 Mac 환경에서 수행하고, Rust 회귀는
최소 글꼴 sandbox에서 수행했다. Native Gulim5.03과 실제 WASM Gulim2.21은 전체 파일의
동일성을 주장하지 않으며 독립 PDF 사용366글리프의 윤곽·전진이 모두 일치함을 각각 확인했다.
[글꼴 신원과 사용 범위](outer-margin-stage/gulim-supply-qualification.json)를 따른다.

Mac fresh WASM은 locked wrapper `--target web --out-dir pkg --no-opt`의 로컬 대체 빌드다.
Docker 최적화 빌드 통과로 보고하지 않는다. 최종 실제 브라우저 검증에서 root pkg·Studio public·
초기화 응답의 JS/WASM 신원을 확인한다. CanvasKit200,000 작업 한도·지원 판정·fallback 검사는 유지한다.

이 디렉터리의 `final-source-2c90`, 이전 rebase·before·부분 검증 기록은 단계별 이력이다.
현재 소스의 최종 수용은 이 원장이 연결한 생산 소스·명령·입력·전쪽 TSV·동일 쪽 review/overlay로
판단한다. 이전 완료 결과나 높은 대표 쪽으로 미달·누락·쪽수 불일치를 상쇄하지 않는다.
비공개 글꼴 바이너리와 원 윤곽 좌표는 Git에 포함하지 않는다.

## 최신 devel 전쪽 대조

[고정 base67b4의 실제 Native/fresh WASM](current-base-67b4/validation.json)으로 각주50쪽과
중첩 표17쪽을 각각 전쪽 대조했습니다. 같은 입력·독립 PDF·글꼴·조건을 유지했으며
2%p 초과 하락0쪽입니다. 이전 base 기록을 최신 base 결과로 표시하지 않습니다.
