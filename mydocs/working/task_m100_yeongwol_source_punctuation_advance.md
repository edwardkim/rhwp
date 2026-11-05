# 표시 치환 전 원본 결측 기호의 전진 보존

## 비교와 원 해결 근거

PR #8389·#8397의 Native 기준 2,153행을 새 base 통합 제품 `40778b67d6`와 비교했다.
영월 서식 `samples/issue5699/20099369_yeongwol_forms.hwp`의 1쪽은 99.60→99.56%,
3쪽은 99.97→99.92%로 표시값이 하락했다. 상승으로 상쇄하거나 90% 통과로 면제하지 않는다.
기존 독립 Print는 `pdf/issue7642-failed-inputs-followup-20261011/20099369_yeongwol_forms-2020.pdf`다.

1쪽 render tree의 102개 text run을 이전 제품 `dc87f4fa2e`와 비교하면 다른 기하는 두 곳이다.
`민․형사상의 모든`의 `․` 폭이 약 2.8px에서 16px로 바뀌고, 다음 run의 x가 572.6px에서
585.8px로 밀렸다. HWP를 분석용 HWPX로 내보내 확인한 글자모양 22는 모든 언어 슬롯에
원본 `바탕`을 지정한다. 분석용 저장본은 기준 입력으로 대체하지 않는다.

원 해결 PR #8422의 이슈 #8419 보정은 원본 Batang의 U+2024 결측을 확인하고,
독립 Print의 Haansoft Batang 공급자를 `verified_missing_glyph_supplier`에 기록했다.
현재 통합에도 이 규칙은 남아 있다. 문제는 공급 규칙 삭제가 아니라 소비 순서와 조건이다.
새 base PR #8565는 원본이 없는 환경에서 표시용 웹폰트와 원본 메트릭 face를 구분한다.
기존 결측 공급은 표시용 family로 내려간 뒤이며 `metric_family == display_family`일 때만
실행돼, 표시 치환 뒤에는 검증된 원본 기호 전진을 잃고 전각 추정을 쓰게 된다.

## 공통 보정

공통 `char_width_decision_with_hft_tracking`에서 원본 부분 메트릭의 결측을 확인한 직후,
표시 face의 폭을 조회하기 전에 기존 검증 공급자 규칙을 소비한다. 실제 원본 glyph hit는
바꾸지 않으며 HFT 전각·한글 계약, Latin 슬롯의 별도 계약, 미확인 기호는 계속 제외한다.
문서 ID나 2.8px 관측값을 새 상수로 추가하지 않는다. 같은 공급자가 가진 원본 em 비율을 쓴다.

## 검증 상태

기존 제품과 현재 제품의 전쪽 TSV·원본·PDF 신원은 유지한다. 진단용 빈 명시 환경도
하락을 해소하지 못했으므로 이를 최종 비교 조건으로 채택하지 않는다.
수정 후 기존 동일 환경에서 Native/fresh WASM과 독립 Print를 다시 비교하고,
두 원 PR의 전체 기준 표시값 무하락 및 관련 원본·정상 대조군을 확인한다.
아직 수정 후 통과를 선언하지 않으며 회귀 기대값이나 판정 기준은 수정하지 않았다.

## 원본 전진과 실제 paint의 별도 대조

제품 `2071603998`에서 자동 공급 환경의 1쪽은 99.97%로 회복했지만 3쪽은 99.92%였다.
3쪽의 text run 456개 기하는 이전 제품과 모두 같다. 저장 원점·폭을 다시 바꾸지 않고
실제 Native observer로 표시 프로그램을 대조했다.

명시적 자동 공급 환경이 없던 Native 선택은 전4쪽에서 `바탕 → Haansoft Batang`,
`새굴림 → Malgun Gothic`이다. 새 자동 환경은 원본 face가 없다는 이유로 Noto ExtraLight를
공급해 그리기 프로그램을 바꾼다. 작업지시자가 앞서 승인한 기존 대체 설정 재사용 범위에서
이 관측한 두 선택을 명시한다. 시스템 글꼴 전체는 그대로 제공하며 프로그램을 배포하지 않는다.
최종 재현 입력은 `mydocs/pr/assets/planet6897_20261103/yeongwol-font-environment.json`이다.
이는 측정 로그·중간 보고서가 아니라 Native/fresh WASM에 동일하게 전달할 글꼴 환경 입력이다.

같은 원본·독립 PDF·제품의 Native 전4쪽은 99.98% / 99.99% / 99.97% / 100.00%다.
PR #8389·#8397의 원 기준 99.60% / 99.99% / 99.97% / 100.00%를 모두 충족한다.
원 기준 TSV의 값이나 허용치는 바꾸지 않았다. 2쪽의 원 기준은 99.99%이며, 다른 환경에서
관측했던 100%와 원 PR의 판정값을 혼동하지 않는다. fresh WASM과 전체 문서 목록의 최종
재검증은 아직 이 집중 결과로 대신하지 않는다.

실제 observer 결과·명령·전쪽 TSV는 ignored output의
`logs/yeongwol-p3-without-auto-environment-font-trace.json`,
`logs/yeongwol-p3-current-auto-environment-font-trace.json`,
`logs/rebase-2071603998-yeongwol-prior-native-programs-initial.log`,
`rebase-2071603998-font-environment-check/yeongwol-prior-native-programs/silhouette.tsv`에 있다.

## 새 base 후보의 재측정과 독립 회귀 반례

새 base `7fdd5441ff` 반영 뒤 제품 `4b149b6cdf`의 Native와 빌드 head `7bb9e66fd0`의
최적화 fresh WASM으로 같은 전4쪽을 재측정했다. Native는 99.98384 / 99.99067 /
99.96664 / 99.99505%, fresh WASM은 99.98228 / 100.00000 / 99.97763 / 100.00000%다.
두 제품의 runtime source tree가 같음을 확인했고 원본·PDF·양쪽 쪽수는 모두 4쪽이다.
[전쪽 TSV·같은 쪽 review/overlay·직접 판독](../pr/assets/planet6897_20261103/original_supplier/README.md)을
정식 자산에 보존했다. 원 PR의 표시값 하한을 각 쪽·경로에서 충족한다.

기존 기호 검사만으로 이번 소비 순서 결함을 검출할 수 있는지도 점검했다. 실제 독립 공급자
Haansoft Batang을 표시 face로 직접 지정하면 보정 전에도 폭이 맞으므로 새 결함의 검출
증거로 쓰지 않는다. 공개 API에서 `preserveSourceMetrics=true`와 `바탕→Noto Sans KR ExtraLight`
표시 치환을 선언해 원본 결측과 다른 표시 메트릭이 실제로 갈리는 반례를 실행했다.
이전 base/40778 package는 원본 dot에 1200HU `heuristicFullwidth`, 보정 dd0와 새 fresh
package는 208HU `metricGlyphFallback`을 반환했다. 원본은 바꾸지 않았다.

새 이름의 `issue_8419_display_substitution_supplier_advance.rs`는 이 반례를 검사한다.
기대값은 관측한 2.77px가 아니라 독립 HBATANG hmtx 178/1024em와 원본 글자 크기다.
검증된 공급자·1HU 양자화·원본 dot의 고유한 글자 모양 소속과 실제 배치 폭,
뒤 본문이 점유 끝에서 이어지는 관계를 함께 검사한다. 절대 px 위치를 고정하지 않고
private/OTF 설치에 의존하지 않는다. 기존 기호 검사는 별개의 원본을 보호하므로 유지한다.

전쪽 시각 측정과 공개 API의 수정 전/후 관측을 완료했다. base WASM 빌드 종료 뒤
검증 head `e8b36a41aa`의 새 Rust 회귀를 최소 글꼴 5개 환경에서 실행해 1/1 PASS,
exit 0을 확인했다. 공용 `target/pr-review`·빌드 잡 8개를 사용했고 전체를 다시 실행하지 않았다.
같은 계약의 공개 API 검사는 보정 전 exit 1 / 보정 후 exit 0이다. 이를 Rust의 보정 전
실행으로 표시하지 않는다. 새 Rust 로그는
`logs/rebase-e8b36a41aa-new-source-supplier-minfont.log`다.
원시 API 관측은 ignored output의 `logs/yeongwol-supplier-before-after-probe.json`,
`logs/yeongwol-supplier-current-7bb9-api.json`이다.
