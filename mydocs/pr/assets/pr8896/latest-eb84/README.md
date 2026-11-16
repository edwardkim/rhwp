# 최신 devel 통합 검증 완료 — 2026-11-12

기준은 `eb84afc0364c51031bb07cbbb013476293367990`, 최종 Rust 검사 source는
`759212053e4b289db93034c7b2b32f5b50bdcd4e`다. 실제 Native는 `1749b66d704b…`,
fresh 최적화 WASM은 `68e8190d334f…`이며 그 뒤 commit은 검사·문서만 바꿨다.
[완료 요약과 제품 source 동일성](completion-summary.json)을 따른다.

| 범위 | 결과·증거 |
| --- | --- |
| 원 사례·#8945 대조군 | Native/WASM 각 전 14쪽 raw 100%, [Native](focus/native/)·[WASM](focus/wasm/) TSV·manifest·실행 영수증 |
| 보호 18개·각 전 1,771쪽 | 공식 두 자리 하락 0, [Native 대조](native-protected-comparison.json)·[WASM 대조](wasm-protected-comparison.json), `protected/<backend>/<key>/` 원본 TSV·manifest |
| 같은 조건의 최신 devel 비교 | Native 1,760쪽·WASM 1,766쪽 PNG 동일. 최악 변화 각각 −0.00112%p·−0.00181%p, 2%p 초과 하락 0. 실제 프로그램·환경·전체 쪽수 대조 포함 |
| 기존 base 기능 7개·전 19쪽 | Native 19쌍·WASM 18쌍 동일, 나머지 1쪽 작은 삼각형 개선. [Native](native-preservation-comparison.json)·[WASM](wasm-preservation-comparison.json) |
| #8958의 새 39개 기능 | 44개 조건·주장 88쪽씩 PNG 모두 동일, 실제 전체 문서 쪽수도 양쪽 동일. [Native](native-incoming-comparison.json)·[WASM](wasm-incoming-comparison.json) |
| 필수 편람 HWPX | Native 공개 지원 웹폰트·383쪽 PASS, [gate](handbook-gate/required-handbook-8153-gate.json)·[전쪽 TSV](handbook-gate/silhouette.tsv) |
| 전체 기본 회귀 | 4,466 + 4,398 + 4,462 = 13,326 PASS, 중복 PASS identity 없음. [실행 영수증](receipts/)·[요약](completion-summary.json) |
| Native 공식 3종 | library 4,035 + placeholder 2 + direct export 36 = 4,073 PASS. 새 WMF 관계 검사도 실제 기존 선택 경로에서 PASS |
| 추가 Native·lint | 공급 protocol 22건, Native 조건부 integration Clippy 2종 및 모든 Rust lint PASS |
| 기존 회귀 보완 | 두 공용 검사의 고정 영역 선택을 소유·폭·하단·clip 관계로 전환, [집중 9건](root-selector-focused.json)·[변경 근거](root-selector-repair-audit.json) |
| 실제 입력·환경 | [154개 Git blob·SHA 대조](input-inventory.json)·[도구 버전](tool-versions.json)·[노출된 공개 글꼴 5개](receipts/minimal-font-exposed.txt) |

원 사례의 base `2 / 3 / 3`쪽은 독립 웹한글기안기 Print와 같은 `1 / 2 / 3`쪽으로 개선됐다.
실제로 이월된 빈 host 줄은 보존됐다. 별도 수용 예외·baseline·판정 한도 변경은 없다.
PDFium 96 DPI·print·문서별 compat·색상 경계 보정 전 2px raw이며 raw 100%를 RGB 전체 동일로 뜻하지 않는다.

이전 생산자와 다른 보호 7쪽씩은 최신 Native/WASM과 독립 Print를 같은 배율로 직접 대조했다.
편람 248쪽 도형, 중첩 표 8·9쪽 내용, sample16 세 형식의 51쪽 WMF 글자·선은 보존되며
raw가 유지되거나 개선됐다. 다른 각 1,764쪽은 이전 PNG와 hash로 연결한다.
[직접 판독 패널](direct-review/)·[Native 변경 영역](native-prior-changed-regions.json)·[WASM 변경 영역](wasm-prior-changed-regions.json)을 따른다.
원 사례 14쪽과 현재 중첩 표 17쪽의 판독·최종 출력 연결도 각각 hash 영수증에 보존했다.

실제 font 프로그램의 순서·경로·SHA는 base/head에서 같다. 공개 웹폰트 변환 중 manifest에 잡힌
크기 0의 `tmp*.ttf`는 프로그램이 아님을 명시해 분리했고 원본 receipt도 유지했다.
43개 명시 환경은 [환경 JSON](font-environments/incoming/), 공개 프로그램 26개의 독립 원본·변환 SHA는
[프로그램 영수증](receipts/incoming-public-font-programs.json)에 있다. 글꼴 bytes·outline 프로그램·인증 정보는 포함하지 않는다.

공용 target에서 재사용된 초기 Native 후보는 base와 SHA가 같은 것을 확인해 제외하고 실제 재컴파일했다.
중단된 WASM 최적화도 완료로 계상하지 않고 복구했다. 입력·생산자·완료 쪽 SHA를 확인하는 정식 체크포인트 재개를 사용했다.
이전 base의 결과는 상위 원장에서 선행 검증으로 구분하며 최신 실행으로 계상하지 않는다.
