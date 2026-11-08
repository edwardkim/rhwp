# PR #8739 CodeQL 경고 #478·#483·#484 검토

## 판정과 제안

세 건을 `false positive`로 분류할 것을 권고한다. 모두 문서를 여는 `DocumentCore::from_bytes_with_password`의 **반환된 DocumentCore 전체**에서 시작한 taint가 조판 숫자까지 전파된 경고다. 실제 sink는 환경변수로 선택한 진단 로그의 행·열·줄 번호, 개수, 높이·가용 공간·오차와 bool 값이다. 암호 바이트, 복호화 본문 문자열 또는 DocumentCore 전체를 로그 인자로 넘기지 않는다. 기본 비활성이라는 이유만으로 면제한 판정이 아니다.

이번 PR에서 세 sink 파일은 기준 devel `b8dd67c5001dfb0410e9e192528676396cdd6a4e`와 blob이 동일하다. 원 경고는 2026-10-31에 이미 생성됐으며, 이번 대형 PR의 종합 보안 check가 다시 차단했다. 과거에는 분류를 유지했으며 이번 턴에도 아직 변경하지 않았다. 파일 불변성만으로 실제 보안 결함을 면제하지 않고 아래 실제 인자와 flow를 확인했다.

| 경고 | 실제 출력과 guard | 동일 base/head blob |
| --- | --- | --- |
| #478 | `scan/runner.rs:120`: 행 번호 `usize`, 최소 높이와 가용 공간 `f64`; `RHWP_DIAG_SCAN` opt-in | `11d4373d5a5510efb92e29b49b0670f15dad3181` |
| #483 | `capacity_tolerance.rs:37`: 행·열, 문단 번호, controls의 길이, `Option<usize>` nested row, bool, 높이·가용 공간·오차 `f64`; `RHWP_DIAG_REFLOW_CAPACITY` opt-in | `527d5d576c769035037bc5a9b40c29493bbd2fed` |
| #484 | `unit_diagnostics.rs:103`: 행·열·cursor, `Option<f64>` accepted와 높이 tuple; `RHWP_DIAG_CELL_UNITS`가 명시한 셀만 선택 | `b7460a8d049f4efb60be4996412b530259b93e8a` |

`open_bytes`는 password를 파서에 전달하고 결과 `DocumentCore`를 session에 보관한다. `from_bytes_inner`도 password를 parse 인자로 사용하고 parsed document를 반환한다. 경고에 표시된 source는 password 변수 자체가 아니라 이름에 password가 포함된 함수의 반환 객체다. 위 세 sink의 인자 표현식·원 타입·제어 조건을 직접 읽었다.

## 도입과 분석 시점

- Native source 도입: `19d2fb9b9bd5b00984022cdf627a699d84fa978d`, 2026-10-15 18:55:08 +09:00.
- #478 sink 도입: `6e1a463b612d95f7b39f0cd450d02576a94739b0`, 2026-10-09 03:11:52 +09:00. source와 처음 함께 존재하는 현재 이력의 commit은 `19d2fb9b9b`다.
- #483 sink 및 source와의 최초 공존: `ba964f94938dea6a9751dcd5794310e859c9420f`, 2026-10-18 03:38:54 +09:00.
- #484 sink 및 source와의 최초 공존: `a858322af71dbec16ed09c7f5607c2e0e2e74eb4`, 2026-10-18 22:07:37 +09:00.
- 최초 등록: 2026-10-31 11:15:27 UTC, analysis `2068096080`, CodeQL 2.27.3, `refs/pull/8410/merge`, commit `35e97c547e0f40d0939b8509e65c78e6e192b546`.
- 현재 분석: 2026-11-07 08:22:25 UTC, analysis `2117586342`, CodeQL 2.27.4, `refs/pull/8739/merge`, commit `8e8a09daf88cc6cd91d683c409757c08ddbc1fde`. PR source head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`.
- 최초 공존 commit에 과거 분석을 새로 실행한 것은 아니다. 코드 도입·연결 가능 시점과 실제 CodeQL 검출 시점을 구분한다.

최초와 현재 SARIF에서 세 경고 각각 4개, **총 12개 flow씩**을 추출했다. #478·#484는 line을 제외한 path/message topology가 같고, #483은 source/sink는 같지만 중간 경로가 90/100/115/102 → 86/95/102/94개 위치로 달라졌다. 이를 동일 topology라고 주장하지 않는다. 현재 #483의 네 경로는 모두 `candidate_height` → `f64` 인자 → 진단 매크로에서 끝난다. #478의 끝은 가용 높이, #484의 끝은 높이 tuple projection이다.

- [최초/현재의 24개 정규화 flow](security-alerts-478-483-484-normalized.json)
- [source·sink 도입과 공존](security-code-introduction.json)
- [현재 실패 check](https://github.com/edwardkim/rhwp/runs/128348258545)

## 승인 뒤 실행 범위

1. 경고 **#478·#483·#484만** `false positive`로 분류한다. 이미 열려 있던 이 세 경고의 repository metadata 변경이며 새 경고를 임의로 일괄 해제하지 않는다.
2. 동일 head의 종합 CodeQL check가 자동 갱신되는지 확인한다.
3. 자동 갱신되지 않을 때에만 **같은 head의 CodeQL Rust 분석 job 및 해당 종속 집계 1회 재실행**을 수행한다. 성공한 제품 CI와 다른 CodeQL 언어는 유지한다.
4. source·workflow·query·severity·검증 기준을 바꾸지 않는다. check 성공 뒤 기존에 승인한 self-review·merge·후속 처리를 이어 간다.

[GitHub 운영 규칙 9.4](../../../../manual/github_operations.md#94-codeql-alert-귀속과-used-in-tests-근거-보존)은 “분류 변경은 O1 보안 metadata mutation이므로 명시 승인 없이 `false positive`, `won't fix` 등으로 재분류하지 않는다”고 정한다. 이 추가 승인이 없으면 현재 분류를 유지하고 merge를 보류한다.

## 승인된 분류와 최종 check

2026-11-07 작업지시자가 세 경고의 오탐 분류와 필요한 경우에만 Rust 분석 1회 재실행을 승인했다. #478·#483·#484를 `false positive`로 분류한 뒤 CodeQL check가 자동 성공으로 갱신됐다. 재실행은 하지 않았다. 위 승인 대기 표현은 조사 시점 기록이다. [실행 결과](security-mutation.json).
