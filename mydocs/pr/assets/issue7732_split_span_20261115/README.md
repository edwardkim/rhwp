# #7732 분할 셀 정렬 후속 보정의 최신 검증 원장

이 PR 범위는 #7732 완료 뒤 확인된 각주 37쪽의 분할 병합 셀 정렬과,
동일한 독립 Print 비교에서 확인한 표시 em·SVG 가는 선·표 바깥여백의 공통 소비 보정이다.
#7445 전체 해결을 주장하지 않는다.

제품 생산 소스는 `2749e491e843ec851de12467751ec200ab551014`다. 이후 base
`67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` 위로 리베이스했으며,
[내용 신원 감사](rebase-67b4-20261116/source-read-path-reuse.json)로 Rust·Cargo·fixture·기존 출력
runtime이 같은 것을 확인했다. asset을 담은 커밋과 실제 실행 파일을 생산한 소스를 구분한다.

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
**최종 전체 시각 수용과 PR 제출은 아직 완료되지 않았다.** 각주 원본 전50쪽 Native/fresh WASM PNG·SVG,
실제 Studio SVG/Canvas2D/지원 CanvasKit, 명시적 `--handbook-gate`를 순차 검증한다.
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
