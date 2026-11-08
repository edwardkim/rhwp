---
kind: investigation
status: active
canonical: mydocs/tech/investigations/issue-8745/README.md
last_verified: 2026-11-07
---

# 암호 문서 처리의 반복 보안 경고 조사

## 결론과 증거의 범위

[#8745](https://github.com/edwardkim/rhwp/issues/8745) 작업 브랜치에서 열기 command와 문서 조회를
분리하고 `OpenOptions`의 Debug 암호 값을 가렸다. 동일 로컬 CodeQL 2.27.4의 PR 범위에서
logging 결과는 [수정 전 107건](base-logging-analysis.json) → [수정 후 0건](candidate-pr-analysis.json)이다.
[추출 범위·경고 대조](analysis-coverage.json)도 같으며 기존 암호 벡터·형식 관련 18건은 유지한다.
테스트·vendor를 포함한 전체 Rust 분석에서도 RHWP 경로의 logging 48→0을 확인했다.
별도 vendor Python 경고 36건은 유지된다. 전후 소스 파일 집합·query 설정과 추출 warning
115건이 같으며, 제품 회귀·Native·fresh WASM·브라우저 검증을 완료했다.
[전체 결과 검토](full-result-review.md)와 [최종 분석 원장](full-codeql-validation.json)에서
두 분석 범위와 남은 결과를 구분한다.

반복 경고를 모두 암호 해제 테스트의 비밀 유출로 볼 수 없다. 최근 `rust/cleartext-logging`은
암호를 받는 함수의 **반환된 문서 전체**를 민감값으로 추적한 경로가 대부분이다. 개별 sink를
삭제하거나 경고를 분류한 뒤에도 이 source가 남아 다른 조판 진단에서 다시 보고됐다.

별도로 실제 보완 대상도 발견했다. 수정 전 `service::OpenOptions`의 자동 `Debug`는 `password` 필드를
가리지 않는다. 이는 문자열 표현으로 암호가 노출될 수 있는 API 결함이며, 운영 로그에서 실제
유출이 발생했다는 증거와는 구분한다. 기존 `PdfImportOptions`와 `NativeRuntimeCommand`는
명시적 `Debug`로 해당 값을 가린다.

- 조사 기준: `devel@e4ffabea38062e716f6c333cc950fa31b42b4dd7`, 작업 시작 시 clean.
- 분석 증거: PR #8739의 analysis `2117586342`, CodeQL `2.27.4`.
- 분석된 merge SHA: `8e8a09daf88cc6cd91d683c409757c08ddbc1fde`.
- 원 PR head: `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`.
- **현재 devel을 새로 분석한 결과가 아니다.** 기본 branch 또는 ref별 dismissed 수를 최신 소스의
  결함 부재나 수정 완료로 해석하지 않는다.

## 실제 분석 결과

| 규칙 | 결과 수 | 조사에서 확인한 성격 |
| --- | ---: | --- |
| `rust/cleartext-logging` | 107 | 문서 반환값 106건·424개 flow, 암호화 저장 반환값 1건·1개 flow |
| `rust/hard-coded-cryptographic-value` | 16 | 테스트·공개 벡터 14건, 키 생성 작업 버퍼의 초기값 2건 |
| `rust/weak-sensitive-data-hashing` | 1 | 기존 형식의 암호 처리 경로. 이번에 새로 안전하다고 판정하지 않음 |
| `rust/weak-cryptographic-algorithm` | 1 | 기존 형식의 암호 처리 경로. 호환성과 보안 정책을 구분해 유지 |

전체 125개 결과의 source·sink·모든 flow location은
[정규화 SARIF](rust-analysis-2117586342.json)에 보존했다. 원문 hash·도구 버전·query pack도 포함한다.
[집계](inventory.json)는 이 파일에서 산출했다. PR ref의 alert 목록 135건과 분석 결과 125개는
서로 다른 API의 집계이며 동일한 수여야 하는 지표가 아니다. 목록은 기존 dismissed 기록도 포함한다.

반복 경로는 `NativeDocumentSession::open_bytes` → `DocumentCore::from_bytes_with_password`
반환값 → `Self::from_core` 및 렌더링 → 조판 진단이다. 경고의 source는 암호 인자 자체가 아니라
함수 호출 결과다. `from_bytes_inner`는 암호를 파서에 전달하고 파싱된 문서로 Core를 만든다.

[CodeQL 2.27.4의 SensitiveData.qll](https://github.com/github/codeql/blob/codeql-cli/v2.27.4/rust/ql/lib/codeql/rust/security/SensitiveData.qll#L25-L38)은
함수·enum variant의 이름으로 민감 데이터 source를 추정한다. 실제 SARIF source와 일치한다.
이는 해당 106건을 개별 출력 인자 확인 없이 모두 오탐으로 재분류하라는 의미가 아니다.

나머지 로그 1건(#190)은 `conversion.rs`의 `export_hwpx_native_with_password` 결과에서
`bytes.len() / 1024`로 이어지는 저장 완료 메시지다. 암호화 바이트 자체를 출력하지 않는다.

하드코딩 16건은 테스트 14건과 제품 함수의 작업 버퍼 2건으로 구분된다. 후자는
`derive_hwp5_key`의 digest 복사 전 배열과 `derive_hwp3_legacy_des_key`의 bit 조립 전 배열이다.
두 초기값을 고정 운영 키로 단정하거나 공개 NIST 벡터를 무작위 값으로 바꾸지 않는다.
각 결과의 최종 판정과 보호 계약은 구현 시 별도 원장으로 확인한다.

## 선행 해결과 이번 재발

| 기존 작업 | 실제 조치·검증 | 이번에 남은 점 |
| --- | --- | --- |
| #6731 → [#6748](https://github.com/edwardkim/rhwp/pull/6748) | Studio 암호 open command를 `void`로 하고 metadata query 분리. JS 분석 87 rules·0 results | Rust 반환 경계의 효과를 대신 입증하지 못함 |
| [#8517](https://github.com/edwardkim/rhwp/pull/8517), `57ce496ef7` → `c97709b597` | 수치 진단 4곳 제거. 실제 patch 확인 | source 자체는 유지 |
| [#8548](https://github.com/edwardkim/rhwp/pull/8548), `3af2224e4e` | 진단 3곳 제거·실제 암호 fixture의 이전 binary FAIL/새 binary PASS | 알려진 출력 marker만 보호 |
| #8548, `5e374d01f6` | 추가 진단 제거. CodeQL 129→126 결과 | 해당 fixture가 새 세 sink를 실행하지 않아 runtime before-FAIL과 구분 |
| [#8739](https://github.com/edwardkim/rhwp/pull/8739) | #478·#483·#484의 24개 최초/현재 flow와 숫자 인자 확인 후 승인된 오탐 분류 | check는 성공했지만 분석 경로는 남음 |

[기존 patch](prior-security-fixes.json)를 GitHub commit API에서 확인했다. 상세 검증은
[6731 보고서](../../../report/task_m100_6731_report.md),
[8548 첫 보정](../../../pr/assets/semanticist21-20261103/ci-security-followup.md),
[8548 추가 보정](../../../pr/assets/semanticist21-20261103/ci-head223-followup.md),
[8739 분류 근거](../../../pr/assets/seo-rii-20261107/remote/security-alerts-review.md)를 연결한다.

#478·#483·#484의 source·sink 도입 SHA, 최초 공존, 최초 분석 `2068096080`과 현재 분석은
8739 기록에 이미 분리돼 있다. 이번 107건의 도입 시점을 모두 그 세 건과 같다고 추정하지 않는다.

## Debug 노출의 제한 재현

현재 `src/service/open.rs`에서 `OpenOptions` 선언과 impl을 그대로 추출해 rustc로 실행했다.
합성 문자열만 사용했고 출력은 포함 여부 두 개의 bool로 제한했다.

```text
compact_contains_synthetic_password=true
pretty_contains_synthetic_password=true
```

[재현 소스](open-options-debug-probe.rs.txt)와 [결과·원문 hash](debug-probe-result.json)를 보존한다.
이는 실제 타입의 표현을 재현한 독립 probe다. 전체 제품 binary·실제 사용자 암호·운영 로그 유출을
검사한 것으로 표시하지 않는다. 구현 시 공개 `service::OpenOptions`를 직접 쓰는 회귀로 보강한다.

## 중복 조사와 다음 단계

GitHub 전체 상태의 CodeQL 제목 검색, 열린 `cleartext`, `암호+보안`, `from_bytes_with_password`
검색에서 동일 Rust 반복 경계를 해결하는 열린 이슈·PR은 발견하지 못했다. #6731은 닫혔고
Studio 경계가 범위다. 열린 #7754는 암호 HWPX의 쪽수·피델리티 이슈이며 이번 보안 경계와 다르다.
#7445·#8232의 범용 검증 정비도 이번 작업의 구현 이슈로 대신 사용하지 않는다.

[수행 계획](../../../plans/task_m100_8745.md)에서 새 이슈·구현 범위를 제안한다.
조사·재현·계획 뒤 작업지시자가 이슈 등록과 구현을 승인했다. [#8745](https://github.com/edwardkim/rhwp/issues/8745)를
등록·할당했으며 승인된 로컬 구현·검증을 완료했다. 위의 원인 조사는 수정 전 기준이며,
구현 결과는 첫 단락과 [보고서](../../../report/task_m100_8745_report.md)에서 구분한다.
GitHub 경고 분류·분석 설정은 변경하지 않았다. PR·push·병합과 원격 분석은 후속 승인 범위다.
