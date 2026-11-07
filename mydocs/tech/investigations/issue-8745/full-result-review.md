---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-8745/full-result-review.md
last_verified: 2026-11-07
---

# 전체 Rust 분석과 테스트 호출 경계

## 첫 전체 결과

`f922a759aa`의 제품·테스트 소스를 포함한 전체 snapshot을 CodeQL 2.27.4,
`rust-postmerge.yml`, 기본 Rust security suite로 분석했다. 2026-11-07 22:37 KST에
종료 코드 0과 SARIF `executionSuccessful=true`를 확인했다.
[정규화 결과](candidate-full-initial-analysis.json)와 [개별 원장](full-result-inventory.json)을 보존한다.

PR 범위의 logging 107→0과 전체 범위의 84건은 대상 경로가 달라 증감으로 비교하지 않는다.
전체 결과는 logging 84건, hard-coded 126건, invalid-pointer 6건, weak-hashing 1건,
weak-algorithm 2건이다. 오류 없이 추출한 파일은 4,610개, 문제를 보고한 파일은 85개다.
추출 종료를 모든 파일·매크로가 완전히 분석됐다는 뜻으로 쓰지 않는다.

## 이번에 보정하는 48건

- 조판 진단 46곳은 `issue_7911_pdf_frontend`, `issue_8160_hwp5_password`,
  `issue_8303_hwp3_hwpx_textpos_idempotent`, `issue_8304_hwp3_index_mark`의 기존 암호
  constructor 호출에서 시작했다. 실제 표·문단·저장 검사는 유지하고, 인증 성공 뒤 조회하는
  `load_bytes`로 호출을 연결한다. 호환 constructor 자체의 검사는 #8745 회귀에서 유지한다.
- HWP3 fixture 두 곳은 실패 메시지에 복호화 본문을 출력했다. 공개 테스트 문서지만 암호
  문서 검사의 진단 관행을 개선할 필요가 있다. assertion 조건과 파서 API 검증은 그대로 두고
  본문 출력 인자만 없앤다. 어떤 조건이 실패했는지와 문단 index는 계속 보고한다.

선행 계약은 #8415→[#8420 검토](../../../pr/archives/pr_8420_review.md), #8160의
`df85dd5967fd1a7e6eefc928ad553837af0d5288`, #8303·#8304→[#8438 검토](../../../pr/archives/pr_8438_review.md)와
현재 파일의 한컴 저장값·독립 출력 근거를 대조했다. 마지막 파일 변경은 각각
`20c9678b01c5`, `df85dd5967fd`, `a942678e0f5a`, `a65b70d4265f`다.
입력·기대값·쪽수·좌표·검사 개수·제품 조판은 변경하지 않는다.

이 보정 이후 전체 회귀와 workspace all-target Clippy, 같은 전체 범위 CodeQL을 재실행한다.
제품 소스는 `f922a759aa`와 같으므로 이미 통과한 Native·WASM 빌드·브라우저 검증을 재사용한다.
현재 보정의 분석 완료 여부는 최종 결과로 따로 확인한다.

## 첫 전체 결과에서 별도로 구분한 항목

| 구분 | 수 | 근거와 처리 |
| --- | ---: | --- |
| 공개 합성 입력·테스트 벡터 | 113 | 실제 credential이 아니다. 고정 입력을 인코딩하거나 테스트를 제외해 경고를 숨기지 않는다. 이번 #8745의 공개 합성 값 4건도 포함한다. |
| 빈 암호 처리 | 5 | PDF의 빈 암호 인증과 FFI의 명시적인 빈 값 처리다. 고정 운영 암호로 판정하지 않는다. |
| 작업 배열 | 4 | 기존 HWP 키 유도 배열 2개와 vendor RNG 채움 전 배열 2개. 오류가 반환되면 미충전 배열을 성공 결과로 사용하지 않는다. |
| 형식 계약 | 5 | PDF R5/R6 UE·OE 복호화의 zero IV 4개와 기존 HWPX PBKDF2 앞의 prehash 1개. 알고리즘을 임의 변경하지 않는다. |
| legacy 알고리즘 | 2 | HWP3 DES와 PDF RC4의 실제 사용이다. 현대적 암호로 인정하거나 안전하다고 재분류하지 않는다. |
| vendor Python logging | 36 | 별도 `#[pymethods]` expansion의 `extract_pyclass_ref[_mut]_trusted`에서 시작해 PDF page index·진단으로 흐른다. 이번 RHWP 테스트 constructor source와 다르다. 기존 vendor 파일을 변경하거나 GitHub 경고를 dismiss하지 않는다. |
| vendor qcms 포인터 검사 | 6 | `cfg(all(test, feature="c_bindings"))`의 gtest raw pointer 역참조다. 암호 문서 호출 경계와 다른 검사이며 안전·오탐으로 확정하지 않는다. |

[실제 기본 feature 해석](default-feature-boundaries.json)에서 PDF backend의 `python`과
qcms의 `c_bindings`는 활성화되지 않았다. 이는 전체 분석에서 그 코드를 제외하거나 vendor
결과를 해결 완료로 처리할 이유가 아니다. 원장과 전체 SARIF에 그대로 남기며, 이 작업을
저장소 전체의 모든 보안 경고 해소로 보고하지 않는다. 기존 PR 범위 18건의 상세 근거는
[기존 암호 결과 검토](crypto-result-review.md)를 유지한다.

## 둘째 분석과 남은 호출의 보정

테스트 6개 파일을 보정한 둘째 전체 분석은 종료 코드 0으로 끝났고,
[logging 43건](candidate-full-second-analysis.json)을 보고했다. 첫 결과 84건보다 줄었지만
RHWP 진단 7곳은 여전히 남았다. source는 HWP3 fixture의 194·556행과 차트 저장 회귀의
251행이었다. 같은 sink에 도달하는 다른 기존 constructor 호출을 둘째 분석에서 확인했다.
따라서 이 결과를 완료로 판정하지 않는다.

남은 테스트 constructor 호출을 목록으로 확인하고, 렌더링·저장·재열기에 쓰는 호출은 모두
같은 `load_bytes` 경계로 연결했다. #4430의 reported 저장·#8415의 Unicode/빈 암호 재열기도
같이 연결한다. 공개 constructor가 열리는지 자체를 검사하거나 feature-off 오류를 보호하는
세 호출은 유지했다. [8개 파일의 변경 전후 해시와 유지 목록](test-boundary-completion.json).

차트 저장 검사는 `6d8232cd1f`에서 암호화한 Chart 부분의 실제 왕복을 보호하도록 등록됐다.
#4430은 `e2c59ca4b0`에서 명시적인 저장의 손실 보고·읽을 수 없는 중첩 자원의 보존을 검사한다.
이 두 계약과 오류 메시지·expected 값은 유지한다. 첫 보정의 전체 회귀 12,831건과 해당 파일
43건 통과는 [그 실행의 기록](test-boundary-followup.json)에 남기며, 새 보정 결과로 혼용하지 않는다.

## 최종 결과와 완료 판정

2026-11-07 23:48 KST에 세 번째 전체 분석의 종료 코드 0과 SARIF의 실행 성공을 확인했다.
[최종 정규화 SARIF](candidate-full-final-analysis.json), [실행·범위 대조 원장](full-codeql-validation.json),
[최종 소스 manifest](full-source-final-manifest.json)에 결과를 보존했다.

| 규칙 | 첫 전체 분석 | 최종 전체 분석 | 판정 |
| --- | ---: | ---: | --- |
| RHWP `cleartext-logging` | 48 | 0 | 테스트의 인증·조회 경계와 복호화 본문 진단 보정 확인 |
| vendor `cleartext-logging` | 36 | 36 | 파일·행 집합과 vendor 소스가 같음. 해결 완료에 포함하지 않음 |
| `hard-coded-cryptographic-value` | 126 | 121 | 공개 합성 값 5개의 호출 경계 변경. 실제 값·검사 입력은 유지 |
| `access-invalid-pointer` | 6 | 6 | 별도 qcms 테스트 결과 유지 |
| `weak-sensitive-data-hashing` | 1 | 1 | 기존 형식 계약 유지 |
| `weak-cryptographic-algorithm` | 2 | 2 | 실제 legacy 알고리즘 사용과 한계 유지 |

하드코딩 결과에서 빠진 5건은 `issue_7911_pdf_frontend.rs`의 `unused` 및 Unicode·metadata-clear·
padded-fields 대조군이다. 문자열을 변경·인코딩하거나 테스트를 제거하지 않았고, 인증 command로
호출한 뒤에는 해당 source 규칙이 결과를 내지 않았다. 운영 credential을 교체한 것으로 보고하지 않는다.

소스 manifest의 4,730개 파일 집합은 같다. 변경은 테스트 8개 파일과 정식 생성기가 분산한
28개 suite 하네스에 한정된다. 분석 snapshot의 모든 해시가 manifest와 같으며, 정상 추출
4,610개·문제 보고 85개 및 추출 warning 115개의 경로·내용·중복 수도 같다. 따라서 목표 경고가
사라진 것을 대상 제외나 추출 누락 감소의 효과로 해석하지 않는다. 기존 추출 문제와 타입·매크로
해석의 한계는 남으며, 이 결과는 모든 가능한 정보 유출의 부재를 보장하지 않는다.

최종 테스트 소스에서 전체 회귀 12,831건·기존 skip 45건, fmt·workspace all-target Clippy·
suite 검사가 통과했다. 변경한 8개 파일의 49건도 개별 PASS를 확인했다.
[최종 테스트 원장](test-boundary-completion.json). 제품과 신규 #8745 회귀 소스는 `f922a759aa`의
검증 해시와 같아 Native 4,077건·fresh WASM·실제 브라우저 결과를 재사용한다.

이로써 승인된 로컬 구현·검증 범위를 완료했다. query·path·severity·workflow·vendor 코드는
변경하지 않았고 실제 비밀 양성 대조 검출도 유지했다. GitHub 경고 재분류·원격 PR·protected
branch full scan의 완료를 뜻하지 않으며, 별도 vendor 결과를 오탐으로 일괄 판정하지 않는다.
