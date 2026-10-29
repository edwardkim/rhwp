# #8174 / PR #8360 — 보안 봇 검출 검토

2026-10-29. code head `a89456207de3bd4dacae2ba3ca5531e5e290675a`, 실제 PR merge ref `refs/pull/8360/merge`의 commit `8b3029fe0047342fdf1f42e7739782215969c8a0`을 검토했다. 분석 workflow 성공과 GHAS 보안 판정은 별개다. 검토 시작 시 GHAS `CodeQL`은 **173 high로 실패**했다. 이후 명시 승인으로 분류를 적용했으며 현재 결과는 마지막 절에 기록한다.

| 구분 | 최초 검토 시 관측 | 최초 로컬 판정과 제안 |
| --- | --- | --- |
| PR CodeQL open | `rust/cleartext-logging` 173건, 모두 같은 `font_families_metric_trusted`에서 시작 | 173건을 `false positive`로 분류하는 안을 제안한다. 승인 전 원격 상태 유지. |
| Secret scanning #1 | AI provider E2E의 합성 Google 형식 key, `E2E`·`PROVIDERKEY` 표식, 외부 provider 요청 전부 mock 응답 | `used_in_tests` 분류를 제안한다. 실제 키 값·토큰을 문서에 넣거나 실제 provider로 유효성을 시험하지 않았다. |
| 기존 CodeQL | 현재 PR 분석에서 기존 dismissed 29건도 재현 | 기존 메인테이너 분류 유지. 아래 실제 제품/테스트 용도를 구분한다. |
| Dependabot | alerts API가 저장소 비활성화를 명시한 403을 반환. security updates도 disabled | 경고 부재를 확인한 것이 아니다. 설정·권한을 바꾸지 않았다. |

## CodeQL 173건의 공통 원인

[정규화한 전체 증거](../pr/assets/issue_8174/t09/security-review.json)에 173개 alert와 **688개 flow의 모든 location/message**를 보존했다. 중복 node 4,623개를 공통 표로 두고 각 flow가 순서대로 참조한다. 큰 원시 SARIF·실행 로그·사용자 입력·비밀 값은 커밋하지 않는다.

모든 시작점은 `src/renderer/style_resolver/resolved.rs:605`의 `clone_fields!(ResolvedCharStyle, ...)`에 있는 같은 식별자다. 선언은 같은 파일 19행의 `Vec<bool>`이며 언어 슬롯별 글꼴 폭 메트릭의 유효성을 뜻한다. 해당 필드는 암호·API key·세션 credential·문서 본문을 저장하지 않는다. getter는 슬롯의 true/false를 조회한다. 예시 [#400](https://github.com/edwardkim/rhwp/security/code-scanning/400)의 sink는 marker frame의 스타일 번호·행 수·시작 index를 출력하는 진단이다.

[CodeQL의 민감정보 식별 구현](https://github.com/github/codeql/blob/main/rust/ql/lib/codeql/rust/security/SensitiveData.qll)은 식별자 이름을 heuristic으로 판정한다. [공통 heuristic](https://github.com/github/codeql/blob/main/shared/concepts/codeql/concepts/internal/SensitiveDataHeuristics.qll)의 `maybeSecret`은 이름의 `trusted`도 포함한다. 현재 SARIF의 실제 시작점·변수 타입·의미가 이 이름 기반 오탐과 일치한다. `Vec<bool>`은 Boolean scalar barrier와도 다르다. 이 검토는 CodeQL query/path 제외, 광범위 sanitizer 또는 이름 변경으로 탐지를 숨기는 처리를 하지 않는다.

| 귀속 시점 | 확인한 근거 |
| --- | --- |
| 필드 도입 | `11cc64a443`, 2026-10-09: 글꼴 프로그램 폭의 유효성 메타데이터 |
| 현재 flow의 clone macro 연결 | 원 작업 `df408614f6`, 2026-10-28; 최신 devel 통합 `4d56be303c`, 2026-10-29 |
| 최초 PR 검출 | Rust analysis `2051097628`, 2026-10-29 04:34:30 UTC, CodeQL 2.27.3 |
| 실제 source/ref | 위 code head / merge commit / merge ref. default branch의 open 목록만 조회하면 이 PR 경고가 빠지므로 ref를 지정해 전 페이지를 수집했다. |

일부 기존 opt-in 진단은 `RHWP_DIAG_MIXFRAG`·`RHWP_ENDNOTE_LINE_DEBUG`로 본문 일부를 출력한다. 이를 일반적인 무본문 로그 보장으로 확대하지 않는다. 이 경계는 173건의 실제 source가 글꼴 메트릭 boolean이라는 판정과 별도로 기록하며, 비신뢰 문서 서비스의 호스트는 내용 진단을 명시적으로 관리해야 한다.

## Secret scanning #1

[경고](https://github.com/edwardkim/rhwp/security/secret-scanning/1)는 `rhwp-studio/e2e/ai-providers.test.mjs:24`에 있다. 원 provider 테스트 `b30bd0edf1`이 #8348 통합 `c76d87cfda742a4624e0a588aa09eb9d89b87d49`로 들어왔으며 현재 PR이 실제 credential을 새로 추가한 것이 아니다. 상수는 길이 39의 provider 형식이며 합성 표식 두 개를 포함한다. 58–80행에서 OpenAI·Gemini 요청을 가로채 미리 준비한 모델/SSE/error 응답을 반환한다. 키 값은 표시하지 않았고 실제 Google 호출·회전·권한 변경은 하지 않았다. 현재 GitHub validity는 unknown이며, 로컬 판정의 근거는 테스트 생성 의도와 mock 경계다.

## 기존 dismissed 29건

- Rust 고정 암호값 16건: 테스트 fixture 14건과 제품 key 유도 함수의 초기 zero buffer 2건이다. 제품의 초기 buffer는 뒤에서 password로 유도한 값으로 채워지므로 실제 고정 credential로 판정하지 않는다. #176·#177의 기존 `used in tests` 설명은 제품 함수라는 실제 역할과 다르다.
- #178 DES: 기존 `used in tests`와 달리 실제 HWP3 읽기·쓰기 경로에서 호출된다. 약한 알고리즘인 것은 사실이며 기존 문서 호환 기능의 제한이다. 이번 타스크가 새 암호 체계를 DES로 설계한 것이 아니다. 이 검토로 기존 분류를 변경하거나 위험을 해소했다고 기록하지 않는다.
- #205 SHA256: 실제 HWPX key 유도에서 SHA256 뒤에 문서 salt·iteration을 쓰는 PBKDF2-HMAC-SHA1/SHA256이 이어진다. 비밀번호를 단순 SHA256으로만 저장하는 코드와 구분한다.
- 기존 Rust cleartext 1건은 암호화한 HWPX 결과의 byte 수/저장 완료 정보를 출력하는 경로다.
- 기존 JavaScript 7건과 Python 3건은 기존 vendor/clipboard/extension/보고 경로의 메인테이너 판정을 유지했다. 이를 이번 PR의 새 취약점 또는 이번 검토에서 모두 해결한 항목으로 표시하지 않는다.

## 원격 처리 경계

[GitHub 운영 규칙 9.4](../manual/github_operations.md#94-codeql-alert-귀속과-used-in-tests-근거-보존)는 “분류 변경은 O1 보안 metadata mutation이므로 명시 승인 없이 `false positive`, `won't fix` 등으로 재분류하지 않는다”고 정한다. 제안은 **새 173건의 false positive 분류와 합성 fixture 경고 #1의 used_in_tests 처리**다. 기존 29건, 탐지 규칙·설정·권한은 유지한다. 승인 뒤 대상·전후 상태와 exact head의 GHAS를 재조회한다.

## 사용자 승인과 실제 적용 결과

2026-10-29 사용자가 두 분류 변경을 명시 승인했다. 승인한 동일 173개 ID가 새 head `f1437377645256367fd53bc1eae724a3c4cb5e19`의 PR merge ref에도 일치하는지 먼저 확인하고 각 상태를 재조회해 false positive로 분류했다. 합성 key #1도 used_in_tests로 resolved 처리했다. 기존 dismissed 29건의 상태/사유는 전후 동일하며 보안 설정은 변경하지 않았다.

적용 후 PR merge ref의 open CodeQL은 0건, 같은 head의 GHAS CodeQL·Build & Test·모든 check run이 성공했다. 새 Rust analysis `2051308996` / merge commit `bf15ecba43da6139d33e913005678707aabf3126`의 192개 결과와 모든 source/sink/flow 위치는 이전 analysis와 정규화 후 동일하다. artifact 표 번호를 제외하고 대체 flow 나열 순서만 정규화했으며 각 flow 내부 순서는 보존했다. 원시 query 결과를 제거한 source 수정으로 기록하지 않는다.
