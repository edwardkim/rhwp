# 개인정보 제목의 결측 기호 공급자 전진 보정

## 독립 근거와 원인

원본은 `samples/task2097/75544_pii_bunseok.hwpx`66쪽이며 SHA-256은
`f429768ec02e5a849d08c06678cbee029fdc8618ca9e2bdcdbba485087fe71f1`이다.
독립 한컴2020 Print `pdf/task2097/75544_pii_bunseok-hwpx-2020.pdf`의 SHA-256은
`0fef38f44b3825f1c97869c2720112536504eaf30cec2a2d2af6d4399ffe8770`이다.
compat2022·192DPI·print·공통 공급 폴더와 실제 Library/Fonts를 사용했다.

50쪽 `1. 목적‧수단 간 비례적 타당성`의U+2027은 원본 휴먼명조 cmap에 없고,
선언한 다음 공급자인 Haansoft Batang에 있다. 실제 glyph를 그리는 공급자의 전진178/1024em과
달리 기존 측정은 합성0.3em 적중을 실제 cmap 적중으로 보았다. PDF에서 확인한 다음‘수’의
원점은144CSSpx인데 기존 공개 paint 원점은146.6267CSSpx였다. 글꼴 자체를 교체하거나
원본 자간을 바꾸지 않고, 검증된 cmap 결측을 기준으로 선언 공급자 전진을 선택하도록 보정했다.

검증된 실제 cmap 계약에U+2027을 추가하고 `metricCharacterMiss`라는 합성 width 출처만으로
결측을 판단하던 제한을 제거했다. HFT·공백 및U+2024의 불명확한 공급 환경 보호는 유지한다.
font program·폭 배열은 새로 커밋하지 않았다.

## 전쪽 raw 판정과 회귀

판정은 사용자 지정 `737d2cea2eeb6fa7eb5afda393eb0f96c1df12fb`의
`silhouette_raw_match_percent`다. 색상 경계 보정값으로98%·90%를 대신하지 않는다.
Native와 fresh WASM 각각 명시 기준 환경66쪽 및 실제 자동 환경66쪽을 비교했다.
두 환경의 각 backend 최저raw는93.77937%, 미달0쪽이며 독립 PDF와66쪽이 일치한다.
빈 명시 환경은 실제 자동 환경의 대용으로 쓰지 않았다.

| 대상 | Native raw | fresh WASM raw |
| --- | ---: | ---: |
| 수정 전50쪽 |99.13401%| 이번 before 범위 아님 |
| 수정 후50쪽 |99.99825%|99.99869%|
| 수정 후 전66쪽 최저(두 환경 각각) |93.77937%|93.77937%|

동일 핵심 제목7·26·50쪽 중 양쪽 raw 최솟값이 가장 높은50쪽을 골랐다.
정식 `issue_7445_missing_glyph_advance`는 독립 Print의 기호140.48CSSpx·다음‘수’144CSSpx·
기준선146.08CSSpx를±2CSSpx로 보호한다. 원문66쪽·제목 완전성·단일 소유·내용 순서·
같은 줄/기호 전진 관계와 원본 글꼴 체인도 함께 검증한다. 최종 공개 paint의 cluster 원점을
검사하며 helper 계산을 기대값으로 재인용하지 않았다.

원래 source6295에서 정식1FAIL(.179초) → 보정 source1PASS(.162초)다.
정상 대조군 #6389 5PASS, #4962 7PASS다. 작업 중 candidate preflight1PASS를
before 실패 대신 제시하지 않는다. before는 실제 부모 커밋 source와 frozen CLI를 사용했다.

## 생산 출처·직접 판독·한계

생산 부모는 `6295fd0dcf4a7dab0d4b5b643a2551209d632243`, 기준 devel은
`552514f423ec91f1b7171f6e6b0882a4efe034f2`다. 두 Rust 파일 후보 diff SHA-256은
`647fc87dce50d6243a87d80707f3852097413694c8812dcd67844250e7cd98d8`이다.
전체 source hash가 보존된 frozen Native와 fresh web/no-opt WASM을 사용했다.
WASM은 Mac 로컬 대체 빌드이며 Docker 최적화 수용을 주장하지 않는다.

- `src/renderer/font_metrics_technical_symbols.rs` SHA-256: `5fc03234c93f9188dab023c764caa1f2fde6f0e09522d00c5325ec38a56bccca`
- `src/renderer/layout/text_measurement.rs` SHA-256: `a09d040012bf4e48e05c1b499eafb26657e87799acdd486e9619eded73f3a6d8`

50쪽 review·overlay를 확대해 제목과 기호 뒤 글자·본문·표·줄끝이 보존됐음을 확인한다.
이 증거는 개인정보 문서의 해당 수정이며48조건 전체·새 전체 회귀·모든 실제 Studio 경로의
완료를 뜻하지 않는다. 다른 문서의 raw 미달은 별도 보정 중이며 PR은 보류 상태다.

| 출력 | review | overlay |
| --- | --- | --- |
| 수정 전Native |[review](before-native-review.png)|[overlay](before-native-overlay.png)|
| 수정 후Native |[review](after-native-review.png)|[overlay](after-native-overlay.png)|
| 수정 후fresh WASM |[review](after-wasm-review.png)|[overlay](after-wasm-overlay.png)|
