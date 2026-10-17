# 가맹사업 원본의 결측 글리프 공급 보정

원본은 `samples/issue5966/1130000-202100008_franchise_review_report.hwp`이며,
독립 기준은 `pdf/pr_6088_6144/hancom2020/`의 같은 원문 한컴 2020 Print PDF다.
입력·PDF·설정·생산 바이너리 해시는 ignored v428/v429 실행 기록에 고정한다.
이 문서는 전쪽 검증 완료나 PR 수용 판정이 아니다.

## 독립 근거와 실제 원인

19쪽의 `⌜`·`⌟`는 휴먼명조 `HMKMM.TTF`의 cmap에 없다. 독립 PDF는 해당 글리프에
Haansoft Batang을 사용하며, 같은 12.48pt 크기에서 다음 글자의 전진도 12.48pt다.
실제 `HBATANG.TTF`의 해당 hmtx 전진은 1024/1024 em이다. 보정 전에는 이 결측
글리프에 반 em 전진이 적용돼 양쪽 본문 간격과 양끝 정렬이 어긋났다.

[프로그램 해시·cmap·hmtx 관측](font-program-proof.json)은 상용 글꼴 바이트를 포함하지 않는다.
기존 `src/renderer/font_metrics_technical_symbols.rs`의 검증된 프로그램 정보와도 일치한다.

## 공급 계약과 소비 경로

[FontEnvironment 설정](font-environment.json)은 원 face를 유지하고 결측 글리프 공급만 선언한다.
`trueTypeFaces`는 실제 공급한 원본 TTF를 입증하며, `glyphFallbacks`는 다른 face로
문서 전체를 바꾸지 않는다. 실행 시 HMKMM과 HBATANG이 있는 디렉터리를 함께 제공해야 한다.

기존 `resolve_styles_with_environment`가 선언된 글꼴 체인을 생산하고,
`text_measurement.rs`의 검증된 결측 글리프 경로가 그 체인의 cmap/hmtx를 소비한다.
Native SVG와 fresh WASM SVG의 글꼴 공급도 같은 설정을 사용한다. 문서 ID 분기,
괄호의 전역 폭 휴리스틱, 원문 변경 또는 절대 픽셀 좌표 보정은 추가하지 않았다.

## 현재 확인 범위

동일 v423 Native 바이너리에서 19쪽은 **86.41272% → 99.98153%**다.
새 `review_019.png`에서 괄호 뒤의 글자와 두 줄의 간격을 직접 대조했다.
숫자 통과가 전체 142쪽이나 fresh WASM의 완료를 뜻하지 않는다.

v429는 추가 셀·OLE 소스 보정이 필요해 `interrupted-for-batched-source-correction`으로
중단했다. 서비스의 MainPID 0·inactive/dead와 자식 종료를 확인했다.
Native 142쪽 SVG와 render tree, 27쪽 PNG 및 512개 글꼴 검사까지 생성됐으며,
전쪽 TSV와 fresh WASM 비교는 완료되지 않았다. 전체 142쪽 수용 근거로 쓰지 않는다.
종료 후 불필요한 SVG 133개를 삭제해 13,307,396,298바이트를 확보했고 대표 9쪽 SVG와
이미 생성된 PNG·원본·기준 PDF·글꼴 검사·로그·해시는 보존했다.
명령·생산 해시·입력 해시·중단 및 SVG 정리 기록은
`output/pr-review/semanticist21-20261008/franchise-current-whole-v429/`에 보존한다.
보정을 모아 적용한 최종 소스에서 공용 target과 루트 pkg로 Native/fresh 전쪽 비교를 다시 수행한다.
