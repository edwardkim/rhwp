# planet6897 원 PR 39개 통합 검토 증거

원 PR 39개를 누적 적용하고 메인터너 보정·검증·개별 리뷰와 PR 본문을 준비하는 로컬 작업이다.
작업지시자는 준비 완료 후 PR 생성·CI 모니터링을 승인했다. 제출 전 필수 검증을 완료했으며 최종 감사를 고정하면 진행한다. 이 기록 고정 시점에는 원격 push·PR 생성·review·comment·close·merge를 수행하지 않았다.

## 현재 제품과 진행 상태

| 항목 | 실제 상태 |
| --- | --- |
| 통합 브랜치 | `review/integration-20261103-planet6897-open` |
| 현재 PR base | `d4a72efd7c91d1e25092ed30b9d44e43cbebb9f0` |
| 실제 기능 대조 base | `f529dc92d4d172bf32390d7749743a45d9e41bfc`; 현재 base와 제품·테스트·fixture·도구 내용이 같음 |
| 리베이스 결과 제품 계열 | `dda217dba06e33eb0749a24872a60e99e9e63e8a` |
| 검증 시작 source | `ba2ebfd34a4ebf402bce8e002254b672a09ef764`; 이후 문서 커밋과 제품 소스를 구분 |
| 리베이스 | 168개 패치 전부 range-diff `=`, 충돌 0건 |
| 개별 검토 | 39개 메인터너 보정 후 수용 가능. 전쪽 대조·필수 검증 완료, 최종 제출 감사 고정 중 |
| 최신 source 필수 검증 | 필수 41개 모두 exit 0. 전체 nextest 12,583 PASS·Skia lib 4,040 PASS·CI Native 21개·doctest 완료. 새 base Native/fresh WASM 보존 대조 완료 |
| 출력 보존 검증 | 기존 80입력 전쪽 결과는 원 출처로 재사용. 실제 변경 소비 경로를 집중 대조 |

[원 PR head와 현재 적용 SHA 39개](integration-commits.md)를 기록했다. 새 외부 PR은 추가로 체리픽하지 않는다.
최신 devel 기능은 같은 호스트·시스템 글꼴의 실제 base/candidate 출력을 대조한다.
상위 PR #8609의 30개 전체 문서를 자동으로 새 범위에 넣지 않고, 영향 소비 경로 14입력·41쪽을
선정했다. 별도로 현재 보정의 세로쓰기·개요·표 경계와 기존 차이 영역 10입력을 확인한다.
새 base 대비 0.1%p 초과 하락은 허용하지 않는다.

## 최신 전체 회귀·Skia 라이브러리 완료

[실제 최종 결과와 명령](rebase8609/rust-regression-validation.md)을 보존했다. 최소 글꼴 환경의 전체 nextest는 12,583 PASS·45 SKIP, Native Skia 라이브러리는 4,040 PASS·12 IGNORE이며 각각 exit 0이다. CI Native 21개와 doctest까지 필수 실행 41개를 모두 마쳤다. 새 base 실제 제품의 Native/fresh WASM 보존 대조도 완료했다.

## 최신 Native 영향 경계 완료

[10입력·43쪽의 Native 대조](rebase8609/native-own-validation.md)는 전쪽 최저 raw 94.92448%,
90% 미만·누락·쪽수·해시 오류 0건이며 직전 후보 대비 0.1%p 초과 하락은 0쪽이다.
[fit 5쪽의 실제 review·별도 overlay](rebase8609/native-vertical-fit/README.md)와
[초과 높이·기존 외곽선·각주 번호 차이의 직접 판독](rebase8609/native-details.md)을 보존했다.
미세 차이를 점수 통과로 해결됐다고 표시하지 않는다. fresh WASM·새 base 자체 비교·전체 공통 검증도 별도 기록에서 완료했다.

## 최신 fresh WASM 영향 경계 완료

[최신 fresh WASM 10입력·43쪽](rebase8609/wasm-own-validation.md)은 최저 raw 96.96022%,
90% 미만·누락·쪽수·해시 오류 0건이며 직전 후보 대비 정밀 raw 0.1%p 초과 하락은 0쪽이다.
[PR #8475의 두 5쪽에서 양쪽 review·별도 overlay를 직접 대조](rebase8609/vertical-cell-claims.md)해
글자 보존을 확인했다. [기존 외곽선·각주 차이의 WASM 판독](rebase8609/wasm-details.md)도 기록했다. 새 base 자체 대조·전체 Rust 등 통합 공통 검증도 완료했다.

## 실제 Studio의 세로쓰기 소비 확인

[같은 두 원본 5쪽의 Canvas2D·CanvasKit 화면](rebase8609/studio-consumers.md)을 독립 PDF와 대조했다. screen/print 12개 캡처에서 실제 backend 불일치·fallback 0건, exit 0이다. 직접 확인한 print 4개 영역의 raw는 99.99761~100%이며 글자 보존을 확인했다. 엄격 RGB parity의 진단 차이는 그대로 기록했다.

## 최신 Native 필수 편람 완료

[최종 제품의 필수 편람 Native 383쪽](rebase8609/handbook-native-validation.md)은 passed·실제 exit 0이다.
최저 표시 raw 99.96%, 기준 대비 표시값 하락·누락·웹폰트 fallback 0건이다. 실행 TSV는 공식 기준과
바이트가 같으며 원시 로그·제품·입력/PDF SHA와 실행 시간을 기록했다. 전체 Rust·fresh WASM·새 base 자체 대조는 아래 별도 완료 기록으로 확인한다.

## 직전 완료 제품의 전쪽·회귀 증거

다음은 `c183a1ac61c2aa4188989fd93032cae68d3538c1`의 실제 결과다.
리베이스 후 제품의 신규 재출력으로 표시하지 않는다.

| 범위 | 완료한 결과와 정식 증거 |
| --- | --- |
| 전체 최소 글꼴 Rust | 12,563 PASS·45 SKIP·exit 0, [필수 40개 실행](rust-validation-c183a1ac61.md) |
| Native Skia | 라이브러리 4,040 PASS·12 IGNORE, CI 통합 21개 모듈 모두 exit 0; 위 실행 기록 참조 |
| 전쪽 Native | [80입력·406쪽](current-c183a1ac61/native-validation.md), 최저 raw 92.45232%, 90% 미만·누락·쪽수·해시 오류 0건 |
| 전쪽 fresh WASM | [80입력·406쪽](current-c183a1ac61/wasm-validation.md), 최저 raw 92.87288%, 90% 미만·누락·쪽수·해시 오류 0건 |
| 당시 base 대조 | 수치 비교 가능한 396쪽의 Native/WASM 0.1%p 초과 하락 0쪽. base 파싱 거부 6쪽·누락 4쪽 복원은 수치 대조와 구분 |
| 실제 실행 | [브라우저·공개 API 4개 실행](runtime-validation-c183a1ac61.md), 이전 결과와 실제 importer/edit 산출물 바이트 대조 |
| 필수 편람 Native | 전383쪽 최저 99.96%, 기준 표시값 하락 0쪽·fallback 0건; Native 검증 문서 참조 |
| 이전 리베이스 영향 Native | [3입력·7쪽](current-c183a1ac61/rebase-scope-native-validation.md), 0.1%p 초과 하락 0쪽 |

시각 측정은 시스템 글꼴 전체와 승인된 기존 대체 설정을 공급한다. Rust 회귀는 상용·개인 글꼴을
숨긴 공개 글꼴 5개 환경에서 별도로 실행한다. 글꼴 프로그램은 제출·재배포하지 않는다.

## PR #8389·#8397 보존

- 직전 c183 Native: [17문서·1,770쪽·원 TSV 2,153행](current-c183a1ac61/pr8389_8397/native-validation.md), 표시값 하락·누락 0행.
- 같은 c183 fresh WASM: [17문서·1,770쪽·원 TSV 2,153행](current-c183a1ac61/pr8389_8397/wasm-validation.md), 표시값 하락·누락 0행으로 완료했다. 최신 ba2 source의 전쪽 재출력과 구분한다.
- [이전 완료 제품의 Native/fresh WASM 전체 보존 기록](pr8389_8397/README.md)과 당시 원본 TSV는 그대로 유지했다.
- 소수 셋째 자리에서 반올림한 두 자리 표시값은 낮아지면 안 된다. 상승은 허용한다.

## PR #8475 메인터너 보정

보정 source `37e5bba3007614d9304c3dbf863503a8c7340405`는 최신 브랜치의
`4a0af7e867`에 같은 공통 로직으로 적용됐다. 표 셀의 글줄 축과 합성 줄 생성 결과를 공유하며,
정상 저장 LineSeg 경로는 보존했다. 문서 ID·특정 좌표로 분기하지 않는다.

[같은 검출 함수의 수정 전 2 FAIL·수정 후 2 PASS와 정상 대조 5 PASS](pr_8475/vertical-axis-focused-37e5bba300.md)를 확인했다.
[Native/fresh WASM 각 6입력·29쪽](pr_8475/vertical-axis-37e5bba300/README.md)은 완료했으며,
문제 입력 두 개의 각각 전5쪽은 양쪽 100.00%다. 나머지 27쪽 PNG는 직전 후보와 같다.
대표 review·별도 overlay 10쌍을 직접 읽고 마지막 글자 보존·셀 내부 포함·다음 내용 관계를 확인했다.
실물 대조군에 남아 있던 저장 세로쓰기 간격·정상 도형의 화살표 표현 차이는 이 보정의 신규 결함이나
완전 일치로 바꾸어 적지 않았다. 보정 source의 [필수 편람 383쪽](pr_8475/vertical-axis-37e5bba300/handbook-validation.md)도 통과했다.
최신 통합 source의 필수 검증과 영향 대조를 끝내기 전에는 최종 수용으로 바꾸지 않는다.

## 최신 fresh WASM 실제 실행 완료

[최적화 제품의 실제 API·브라우저 검사](rebase8609/runtime-validation.md)는 세 실행 모두 exit 0이다.
브라우저 271 assertion·17 Worker·12쪽 렌더/저장, 공개 API 계약 2개, 가져오기·편집 산출물 4종의
기존 Native 생성 입력과 바이트 일치를 확인했다. 이번에 Native importer까지 새로 실행했다고
표시하지 않는다. 최신 전쪽 시각·새 base 대조·전체 Rust의 완료와는 별개의 증거다.

## 새 base의 CanvasKit 소비 계약

[제어문자·밑줄·장식선의 합성 계약 21개](rebase8609-backend-contract.md)는 모두 통과했다.
실제 모듈 메서드의 결과이며 한컴 문서의 화면 캡처나 전쪽 시각 통과를 대신하지 않는다.

## 원본·쪽·대표 그림의 대응

- [입력·PDF SHA-256 148개 대조표](input-integrity.tsv): 기존 공식 입력과 독립 Print를 재사용한다.
- [직접 판독 170건](direct-reading-current.md)과 [실제 그림 해시](direct-reading-image-sha256.md): 실제로 연 review와 별도 overlay만 집계한다.
- [재실행 명령과 80개 입력·PDF·compat·글꼴 계약](visual-reproduction.md): 로컬 임시 실행기가 없어도 canonical 명령으로 재산출할 수 있다.
- [주석 변경의 회귀 함수 보존](test-header-only-audit.md): 주석만 바뀐 검사를 실제 동작 변경과 구분한다.

원 PR별 리뷰는 그 PR이 주장한 입력·쪽에 대응하는 그림과 TSV를 연결한다. 이전 producer 그림 재사용은
최신 source의 재출력과 구분하고, 점수가 높아도 수식·글자·미세 간격의 직접 판독을 생략하지 않는다.

## 글꼴 공급 계약과 주석 보완

[공급 adapter의 합성 계약 11개](rebase8609/font-registration-validation.md)는 최소 글꼴 환경에서 모두 통과했다.
수정한 Node 테스트 파일의 첫 줄부터 관계형·실행 근거·직전 commit 시각을 기록하고, 기존 실행 코드가
바이트 단위로 같음을 확인했다. 주석 보완 후 해당 11개만 재실행했으며 전체 Rust를 재시작하지 않았다.

## 중간 산출물과 과거 기록

검증 중간 JSON 60개는 제출 자료에서 제거하고 로컬 output에 바이트 그대로 보존했다.
[실제 결과·중간 출처 해시](intermediate-evidence-summary.md)와 [이전 단계 이력](history-before-rebase8609.md)을 연결한다.
기능 입력인 `yeongwol-font-environment.json`, 원본·독립 PDF·전쪽 TSV·대표 PNG는 유지한다.
기존 완료 증거의 출처·범위를 확대하거나 최신 검증의 통과로 승격하지 않는다.

## 새 base 기능의 실제 보존 대조 완료

[Native](rebase8609/native-base-preservation.md)와 [fresh WASM](rebase8609/wasm-base-preservation.md)의 14입력·41쪽은 양쪽 모두 raw 하락 0쪽·미달/누락/쪽수 차이 0건이다.
[문서-only base 갱신의 내용 동일성](latest-rebase.md)을 확인했고 추가 전체 회귀·전쪽 재측정을 반복하지 않았다. 각 경로의 39쪽 PNG가 base와 바이트 단위로 같고, 나머지 2쪽도 raw가 같으며 같은 차이 영역을 직접 확인했다.

[39쪽의 바이트 동일성과 나머지 2쪽의 직접 확대 판독](rebase8609/base-preservation-direct-reading.md)을 함께 보존했다.
