# PR #8622 반영 전 source의 검증 기록

이 기록의 제품 source는 `6337fe4d102b591b65c61b92a8da8b2df71eb8cb`, 비교 base는 `ed057276c1e0e07e743218d95641e015b46b6715`다. 중간 devel `d4a72efd7c`와 ed057의 제품 tree는 같다. 이후 추가된 #8622를 반영한 최신 후보의 통과로 사용하지 않는다.

Native CLI SHA-256 `b4fc22625e0a840d978bc492832b1bf8a5d35584e0970aa8c073bfde21cd7a92`, 최적화 fresh WASM SHA-256 `ad9df760f66670afa46c76fcce41bf92a2b9387e3091239b74dcc7471873c319`다.

## 완료한 실행

| 실행 | 결과 | 초 |
| --- | --- | ---: |
| `native-lib-build-v6` | exit 0 | 385.56 |
| `native-lib-runtime-v6` | exit 0 | 19.64 |
| `issue_2225_missing_picture_placeholder-native-v6` | exit 0 | 290.53 |
| `render_p37_direct_pdf_export-native-v6` | exit 0 | 1.36 |
| `clippy-root-v6` | exit 0 | 143.15 |
| `clippy-wasm32-v6` | exit 0 | 122.08 |
| `workspace-build-v6` | exit 0 | 185.67 |
| `manifest-prepare-v6` | exit 0 | 1.43 |
| `fmt-final-v6` | exit 0 | 24.62 |
| `clippy-all-targets-v6` | exit 101 | 142.85 |
| `manifest-prepare-final-v8` | exit 0 | 1.43 |
| `fmt-final-v8` | exit 0 | 23.93 |
| `manifest-final-v8` | exit 0 | 0.77 |
| `clippy-all-targets-v8` | exit 0 | 13.94 |
| `nextest-build-final-v8` | exit 0 | 643.84 |
| `nextest-runtime-final-v8` | exit 100 | 674.03 |

전체 nextest 첫 실행의 1 FAIL은 `samples/` 전용 환경변수에 `tests/fixtures/` 목록을 잘못 전달해 입력을 거부한 실행 오류다. 탐지기나 허용 목록은 바꾸지 않았으며, 환경변수를 제거한 보안 suite 재실행 6건과 **fixture 45개 × CLI hidden-text·injection(include-fields)·unicode 3종**이 모두 통과했다. 이번 diff에 신규/변경 `samples/`는 없다. 올바른 환경의 전체 nextest 재실행은 **12,544 PASS·0 FAIL·45 SKIP**, 실제 테스트 실행 889.668초·exit 0이다.

편람은 기존 고정 KoPub/HCR 환경의 실제 Native 383쪽이 passed·최저 99.96%이며 370쪽이 100%였다. baseline을 변경하지 않았다. 일반 샘플 글꼴 10개를 전달한 초기 실패와 이 canonical 환경의 최종 통과를 구분한다.

Studio TypeScript·2,720 tests·build, 실제 최종 WASM API 9개 묶음(한컴 계산 값 37개), 실제 Studio Canvas2D 6장면, 실제 CanvasKit 문자 획 및 테두리/각주 소비 검사, bindings 계약은 exit 0이다. 화면/효과 검사와 독립 Print 시각 비교는 별개다. FORMULA의 명시적 재계산 API와 drop-cap 속성 저장 보존 범위를 새로운 자동 UI 구현으로 확대해 설명하지 않는다.

글꼴 보정은 [문서별 공급 근거](font-qualification.md)를 따른다. 108.90 MiB 초기 공통 팩의 TSV는 최종 수용 증거에서 제외했다. 올바른 구성의 전쪽 수치와 확대 판독을 확보한 뒤 별도 표로 보존한다.


## 글꼴 보정 후 전쪽 결과

46개 입력/Print 상태, Native/fresh WASM 각각 **124쪽**이다. Native 최저 **96.49%**, fresh WASM 최저 **97.10%**, 90% 미만·누락·독립 PDF 쪽수 불일치·base 대비 2%p 초과 하락은 없다. [실제 전쪽 TSV와 입력 해시](validation/pre8622/summary.json)를 보존했다. 이 source를 최신 #8622 반영 후보의 결과로 사용하지 않는다.
