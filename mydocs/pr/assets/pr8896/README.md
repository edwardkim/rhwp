# PR #8896 메인테이너 검증 증거

현재 기준은 최신 devel `eb84afc0364c51031bb07cbbb013476293367990`이다.
[최신 통합 검증 원장](latest-eb84/README.md)에서 진행·완료를 구분한다.
아래는 이전 고정 base에서 실제 완료한 선행 검증이며 최신 source의 실행으로 계상하지 않는다.

## 이전 고정 base의 선행 검증

원 기여 head `51a960e392a3d79a4d28f78fa90a2aa88ad8c0d7`, 기준 devel
`4c876654d86d071d342fc78706c51cfff6ddbde4`를 보존한다. 최종 실제 Native·fresh 최적화 WASM의
제품 source는 `ae5c72c710ac116132d2b9dcf62078bbc884beff`다.
그 뒤 테스트·도구·문서 commit과 production tree가 같음은 [생산자 영수증](receipts/production-source-equivalence.json)에 있다.
기본 작업공간의 배포용 JS/WASM은 갱신하지 않았다.

| 검증 | 증거 |
| --- | --- |
| 원 #8693 3개·전 6쪽과 #8945 5개·전 8쪽, 양쪽 모두 raw 100% | [최종 Native](final-focus/native/), [최종 WASM](final-focus/wasm/). 초기 source의 전체 비교·overlay·PNG는 `focus/`에 구분해 보존. |
| 보호 18개·1,771쪽씩, 공식 값 하락 0 | [최종 비교 JSON](protection/comparison.json), [공식 두 자리 TSV](protection/comparison.tsv), `protection/<backend>/<key>/` 원본 TSV·full-precision manifest·실행 영수증. |
| 최신 devel과 같은 최종 공급·전 3,542쪽 비교 | [devel 대조](protection/devel-comparison.json), 2%p 초과 하락 0, 최대 하락 Native 0.00117%p·WASM 0.00190%p. |
| 과거 보호 미재현 911행, 예외 미적용 | `protection/initial-comparison.*`는 초기 진단으로만 보존. 최종 수용 값과 구분. |
| 최신 base 기능 7개·19쪽씩 | [최종 비교](final-preservation/comparison.json), Native 19쪽 PNG 동일, WASM 18쪽 동일·1쪽 원 PUA 복원으로 개선. |
| 필수 HWPX Native 편람 383쪽 | [최종 실행 영수증](receipts/final-handbook-receipt.json)·[실제 gate](handbook-gate/required-handbook-8153-gate.json)·[383쪽 TSV](handbook-gate/silhouette.tsv). 공개 지원 웹폰트 공급, 최종 생산자에서 gate 통과. |
| 공급 원인·독립 프로그램 근거·실제 font hash | [글꼴 공급 기록](font-supply/README.md), [확대 판독](font-supply/direct-review/). font 프로그램 자체는 포함하지 않음. |
| 기존 회귀 점검 | [점검 결과](regression-audit.md), [정확한 경로·source hash](regression-audit-inventory.json). 기존 px 핀 교체 0, 유효한 관계·원본 단위·완료 쪽 좌표 유지. |
| 입력 커밋 확인 | [66개 파일의 Git blob·SHA 확인](input-inventory.json). 기존 정식 파일을 재사용하며 중복 사본을 추가하지 않음. |

2026-11-11~12, PDFium 96 DPI·print·문서별 compat 2022/2024·색상 경계 보정 전 2px
`silhouette_raw_match_percent`다. Native는 Skia 직접 PNG, WASM은 locked wrapper와 wasm-opt를
거친 fresh 빌드의 Chromium revision1660786 직접 Canvas PNG다. 원 #8693의 기준은 공개 합성
입력의 **웹한글기안기 Print**이며 데스크톱 한컴 출력과 구분한다. 비공개 실문서 보고를
메인테이너의 독립 실행으로 계상하지 않는다. raw 100%를 RGB 전체 픽셀 동일로 해석하지 않는다.

전체 Rust 회귀·Native 공식 검사·lint의 최종 결과는 [검토 문서](../../pr_8896_review.md)에 갱신한다.
필수 검증이 끝나기 전 이 원장만으로 승인·제출하지 않는다.
