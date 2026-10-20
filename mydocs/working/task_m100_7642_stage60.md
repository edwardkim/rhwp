# 이슈 #7642 Stage 60 — 인용 기호의 공통 글리프와 자동 간격

Issue: #7642

## 독립 Print에서 남은 차이

Stage 59 code head `992a275829741001740ef0deb2c6e192d7fbc555`는 기본 인용
변수의 글꼴 전환을 보정했으나 혼합 문자열의 수학 기호·자동 간격은 빠뜨렸다.
99.13060%의 높은 점수에도 직접 확대 판독에서 x+y·x-y·x=y가 Print보다 좁다.
한컴 Print의 텍스트 원점과 실제 Native RenderTree/LayoutBox를 같이 확인했다.

- 독립 PDF의 기호는 HYhwpEQ의 수학 PUA다. +는 U+E048, −는 U+E046,
  =는 U+E047이고 괄호도 수학 PUA다. 인용은 명령·첨자 해석을 막지만
  수학 글리프·자동 여백까지 정체 ASCII 글자 폭으로 바꾸지 않는다.
- 실제 layout_literal_text는 기호를 layout_number로 보내며 -를 하이픈으로
  표시하고 기호 양쪽의 자동 간격을 생산하지 않는다. 빈칸도 숫자 face의
  일반 space 전진폭으로 게시한다.
- 실제 Native의 9pt 입력 x+y·x-y·x=y 선언 폭은 각각
  25.36/25.36/27.04이고 자연 폭은 22.336/16.996/22.336이다.
  원본 저장 폭과 독립 PDF 위치가 공통 symbol_spacing의 글리프·양쪽 여백과
  일치한다. 이는 관측값이며 절대 px 회귀 기대값으로 추가하지 않는다.
- 인용 x y 선언 폭은 14.493333이고 자연 폭은 16.984다. 문자 x/y의 공통
  전진과 한컴 BaseUnit/8 Thin 공간을 합하면 선언·Print와 대응한다.

원본 PDF SHA는 Stage 59의 인쇄 증거와 같다. PDF word 원점은
`stage59-quoted-mode-probe/print-text-bbox.html`, 실제 완성 Native library의
모델·RenderTree·LayoutBox는 `stage60-current-quoted-layouts.log`에서 읽었다.
사용한 helper 원문과 컴파일 로그도 같은 ignored output에 보존했다.

## 원인 소비 경로와 수정 준비

Quoted AST 원문 → literal_text의 문자별 glyph 종류 → 공통 layout_row/
symbol_spacing → LayoutKind·전진폭·기준선 → SVG·Canvas의 같은 children
소비 경로로 연결한다. 문자열 자체를 parser에 다시 넣지 않는다.
예약 명령 sin/alpha·밑줄/지수 문자를 함수·명령·첨자로 바꾸지 않으며,
표시할 수학 연산자·구분자와 공백만 기존 측정·자동 간격 결과를 소비한다.
절대 문단 위치·원본 이름·고정 폭 보정은 넣지 않는다.

수정안은 `stage60-literal-symbol-spacing.patch`와 source hash JSON에 준비했고
`git apply --check`를 통과했다. 아직 source에는 적용하지 않았다.
Stage 59 fresh WASM이 실제로 종료되고 해시를 확인할 때까지 source를 유지한다.

인용/비인용 +/−/=, 빈칸/명시적 Thin, 괄호, 점, 시작·끝 연산자,
연속 연산자, 관계 기호와 나눗셈 22개를 같은 정상 section·font·BaseUnit으로
추가 생성했다. `stage60-quoted-spacing-probe/input-plan.json`에 생성·원문
해시를 남겼고 Save job `adfe55c8-605c-4dee-89d7-1e6d6c09ae19`를 모니터링한다.
이 원격 요청은 진행 중이며 성공·Print 일치 증거로 미리 사용하지 않는다.

Native/fresh WASM·직접 확대 판독과 전체 55개·853쪽의 원래 검증 범위,
관련 회귀의 관계형 전환·전체 Rust 및 필수 lint 범위는 계속 유지한다.
렌더 회귀·baseline 기대값은 아직 변경하지 않는다.


## 추가 Print의 실제 완료와 경계 판독

같은 Save job을 끝까지 모니터링했고 Print job
`211633cc-ce03-4f30-aef7-fd1261718b86`도 완료해 다운로드했다.
Hancom 13.0.0.3901, PrintMethod 0, frame_print_to_pdf_ex_one_up이며
PDF SHA-256은 `640c04b8a35154a6e3ab2c43de7c16e4413936e43891d5a384563a3a0d432c59`다.
MCP 실제 상태·출력 해시는 `stage60-quoted-spacing-probe-mcp-print.json`,
출력 이미지와 같은 배율/좌표는 `print-quoted-spacing-contract-detail.png`·
`print-detail-provenance.json`에 보존했다. 새 확대 이미지를 직접 표시했다.

- 인용/비인용 x+y·x-y·x=y는 같은 기호와 간격이다.
- 인용 x y와 명시적 Thin x`y는 같은 간격이다.
- 인용/비인용 (x), 시작 +x와 끝 x+, 관계 기호 x<y의 표시가 대응한다.
- 인용 x+-y는 두 기호를 보존하지만 비인용은 ±로 해석한다.
- 인용/비인용 x/y는 같은 한 줄 slash 표시다. 확대 이미지의 인상만으로
  비인용을 분수로 해석했던 기록은 아래 실제 메트릭 대조로 바로잡았다.

따라서 인용 원문을 tokenizer/parser에 다시 넣는 해결은 인용 x+-y의 기호 보존 경계를
훼손한다. 준비한 문자별 EqNode의 공통 layout_row 연결은 명령·분수·첨자
재해석을 하지 않고 glyph 종류와 자동 간격만 소비한다. 이를 다음 Native/
fresh WASM의 같은 입력과 확대 판독으로 확인한다.

검증 성능 대안으로 CLI subset 옵션을 확인했으나 canonical Sweep가 Unicode
cmap 훼손 때문에 명시적으로 거부했다. 해당 시도는 argument exit 2이고
원본 출력의 렌더 실패가 아니다. 이 경로의 결과를 검증에 사용하지 않으며
실행 중인 검증은 원본 글꼴 full 임베딩을 유지한다.
`stage60-subset-font-diagnostic-status.json`에 구분했다. 실제 공급 glyph의
outline/advance·cmap을 보존한 외부 subset을 full로 주는 별도 계약과 혼동하지 않는다.


## 실제 소비 결과를 확인한 뒤 적용

실제 Native의 대응 LayoutBox를 더 확인했다. 비인용 x+y는 공통
symbol_spacing을 사용하며 같은 Print 선언 틀과 글리프 원점에 대응한다.
일반 괄호 (x)의 실제 paint 결과도 이미 공통 Row로 변환되어 폭 17.84다.
layout_paren의 private helper만 보고 최종 출력이 다르다고 판정하지 않는다.
증거는 `stage60-current-quoted-paired-layouts.log`다.

Stage 59 fresh WASM은 exit 0, 1212초로 종료했고 고정 패키지와 Studio
public의 JS/WASM SHA-256 일치를 확인했다. 빌드 중 소스는 유지했다.
종료 확인 뒤 Stage 60 수정안을 적용했다. AST의 인용 원문은 보존하고,
문자별 표시 노드만 공통 줄 계산에 전달한다. +/−/=, Thin 공간, 괄호의
측정과 paint가 같은 결과를 소비하며 여백을 나중에 clamp하지 않는다.

새 Native를 빌드해 독립 Print 진단 입력 3개, 원본의 영향 페이지, 전체
55개·853쪽 Native/fresh WASM TSV를 다시 확인한다. 앞 source의 47쪽은
Native 최저 90% 이상이지만 새 source의 최종 증거로 사용하지 않는다.
관련 회귀의 관계형 전환과 전체 Rust/lint는 아직 완료되지 않았다.


## 새 소스 검증과 이전 전체 큐의 정리

보정 code head는 `f6f51dd5d`다. 새 Native 빌드 → 집중 40쪽/전체 55개·853쪽
Native → 순차 Native lint/fresh WASM → 집중/전체 fresh WASM pipeline을
시작했다. `stage60-validation-pipeline.json`·`stage60-wasm-follow.json`과
각 Native/WASM provenance에서 실제 소스 해시·명령·owner·종료를 확인한다.
현재 Native 빌드는 실행 중이며 예약한 검증은 통과 증거가 아니다.

앞 code head 992a27582의 전체 Native에서는 원본 두 개의 47쪽이 완료됐다.
미주 사이 20 HWP는 24/24쪽, 최저 92.13937%; 구분선 아래 20·위 20 HWP는
23/23쪽, 최저 90.77312%이고 누락 쪽은 없다. `stage60-native-whole-checkpoint.json`
에 원본·PDF·쪽수와 수치를 고정했다. 이 앞 소스 결과를 새 소스의 완료로
대신하지 않는다.

소스가 바뀐 뒤 중복 실행되는 앞 소스의 전체 Native/WASM 큐만 종료했다.
해당 owner·하위 PID·명령 소유를 확인했으며 현재 보정과 무관한 프로세스,
집중 검증과 공유 Cargo 작업은 보존했다. `stage60-superseded-whole-queues.json`
에 실제 종료 대상과 새 검증의 전체 범위 유지 사유를 기록했다.

종료된 두 전체 큐의 중간 SVG 115개, 9,274,982,034바이트를 해시 기록 뒤
정리했다. PNG·TSV·tree·로그·원본·PDF·고정 Native/WASM·공유 target은 보존했다.
파일별 기록은 `stage60-superseded-svg-cleanup.jsonl`, 최종 집계는
`stage60-superseded-svg-cleanup-summary.json`이다. 현재 실행 중인 산출물은
삭제하지 않았다.

22개 인용 기호 대조의 수정 전 Native는 97.16845%, 엄격 내용 픽셀은
10.26565%다. 높은 점수에도 위 간격 차이가 있어 완료로 인정하지 않았다.
새 소스의 실제 비교와 확대 판독 후 회귀 전환 선행 조건을 다시 확인한다.


## 새 Native 비교와 남은 의미 차이

f6f51dd5d의 Native 빌드는 exit 0으로 끝났고 집중/전체 Native가 실제 실행 중이다.
인용 기호 22개 입력 1쪽은 97.16845%→98.63868%, 혼합 인용 24개 입력
1쪽은 99.13060%→99.79610%, 그리스 명령 입력 1쪽은 100.00000%다.
새 `stage60-current-quoted-spacing-before-after-print-detail.png`를 실제로 표시했고
provenance에 같은 좌표·배율, source·Native·PDF·PNG 해시를 남겼다.

인용 기호의 +/−/=, 공간과 괄호는 같은 비인용의 공통 배치로 개선됐다.
그러나 비인용 x+-y는 Native에서 두 기호로 남으며 Print는 ± 하나다.
이 기존 해석 결함을 인용 간격 보정의 완료로 대신하지 않는다.
신규 회귀·baseline 갱신 없이 다음 보정 묶음의 독립 계약으로 분석한다.

x/y를 비인용 분수라고 기록했던 해석은 잘못이었다. 실제 저장 선언은 인용/
비인용 모두 폭 19.0, 높이 12.0이고, PDF의 양쪽 word는 같은 PUA
U+E0FC·U+E054·U+E0FD이며 같은 줄의 크기로 출력된다.
`stage60-quoted-spacing-probe/print-text-bbox.html`와 대응 Native LayoutBox로
확인했다. 비인용 Native는 slash의 기본 여백 때문에 자연 폭이 20.2로
증가하며 인용 Native는 선언과 같은 19.0이다. 원인을 분수 재조판으로
잘못 분류하지 않고 기호 전진·여백 소비 경로로 조사한다.
