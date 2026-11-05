# planet6897 원 PR 39개 통합 검토 증거

원 PR 39개를 누적 적용하고 메인터너 보정·검증·개별 리뷰와 PR 본문을 준비하는 로컬 작업이다.
최종 PR 생성 전에 작업지시자의 승인을 기다린다. 원격 push·PR 생성·review·comment·close·merge는 수행하지 않았다.

## 현재 제품과 진행 상태

| 항목 | 실제 상태 |
| --- | --- |
| 통합 브랜치 | `review/integration-20261103-planet6897-open` |
| 최신 검증 기준 base | `f529dc92d4d172bf32390d7749743a45d9e41bfc` |
| 리베이스 결과 제품 계열 | `dda217dba06e33eb0749a24872a60e99e9e63e8a` |
| 검증 시작 source | `ba2ebfd34a4ebf402bce8e002254b672a09ef764`; 이후 문서 커밋과 제품 소스를 구분 |
| 리베이스 | 168개 패치 전부 range-diff `=`, 충돌 0건 |
| 개별 검토 | 38개 조건부 수용, PR #8475 보정 후 통합 검증 대기; 최종 승인 0개 |
| 최신 source 필수 검증 | 최소 글꼴 Rust·Native Skia·lint·최적화 WASM 묶음 실행 중. 완료로 선언하지 않음 |
| 출력 보존 검증 | 기존 80입력 전쪽 결과는 원 출처로 재사용. 실제 변경 소비 경로를 집중 대조 |

[원 PR head와 현재 적용 SHA 39개](integration-commits.md)를 기록했다. 새 외부 PR은 추가로 체리픽하지 않는다.
최신 devel 기능은 같은 호스트·시스템 글꼴의 실제 base/candidate 출력을 대조한다.
상위 PR #8609의 30개 전체 문서를 자동으로 새 범위에 넣지 않고, 영향 소비 경로 14입력·41쪽을
선정했다. 별도로 현재 보정의 세로쓰기·개요·표 경계 8입력을 확인한다.
새 base 대비 0.1%p 초과 하락은 허용하지 않는다.

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
- 같은 c183 fresh WASM: 남은 실행·집계를 진행 중이다. 부분 실행을 17개 완료로 표시하지 않는다.
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

## 새 base의 CanvasKit 소비 계약

[제어문자·밑줄·장식선의 합성 계약 21개](rebase8609-backend-contract.md)는 모두 통과했다.
실제 모듈 메서드의 결과이며 한컴 문서의 화면 캡처나 전쪽 시각 통과를 대신하지 않는다.

## 원본·쪽·대표 그림의 대응

- [입력·PDF SHA-256 148개 대조표](input-integrity.tsv): 기존 공식 입력과 독립 Print를 재사용한다.
- [직접 판독 158건](direct-reading-current.md)과 [실제 그림 해시](direct-reading-image-sha256.md): 실제로 연 review와 별도 overlay만 집계한다.
- [재실행 명령과 80개 입력·PDF·compat·글꼴 계약](visual-reproduction.md): 로컬 임시 실행기가 없어도 canonical 명령으로 재산출할 수 있다.
- [주석 변경의 회귀 함수 보존](test-header-only-audit.md): 주석만 바뀐 검사를 실제 동작 변경과 구분한다.

원 PR별 리뷰는 그 PR이 주장한 입력·쪽에 대응하는 그림과 TSV를 연결한다. 이전 producer 그림 재사용은
최신 source의 재출력과 구분하고, 점수가 높아도 수식·글자·미세 간격의 직접 판독을 생략하지 않는다.

## 중간 산출물과 과거 기록

검증 중간 JSON 60개는 제출 자료에서 제거하고 로컬 output에 바이트 그대로 보존했다.
[실제 결과·중간 출처 해시](intermediate-evidence-summary.md)와 [이전 단계 이력](history-before-rebase8609.md)을 연결한다.
기능 입력인 `yeongwol-font-environment.json`, 원본·독립 PDF·전쪽 TSV·대표 PNG는 유지한다.
기존 완료 증거의 출처·범위를 확대하거나 최신 검증의 통과로 승격하지 않는다.
