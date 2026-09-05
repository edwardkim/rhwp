---
kind: investigation
status: active
canonical: mydocs/plans/task_m100_6731.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 CodeQL alert #186 데이터 흐름 기준선

## 판정 경계

CodeQL alert #186의 메인테이너 최종 분류는 `dismissed` / `used in tests`다. 지적된
`scripts/font_rule_ledger.mjs:104`의 SHA-256은 암호 저장·검증이 아니라 테스트·계측용 폰트 규칙
snapshot JSON의 무결성 digest다. 이 문서는 그 판정을 바꾸기 위한 자료가 아니라 실제 flow와 실행
경계를 재검증하기 위한 기준선이다.

## 증적 원본

| 역할 | 분석 ID | ref와 SHA | CodeQL | 시각(UTC) | 파일 SHA-256 |
| --- | ---: | --- | --- | --- | --- |
| 최초 관찰 | `1659063897` | `refs/heads/devel@5057a7fcaf055b928e76115cdee4bc20bf0936f9` | 2.26.3 | 2026-08-23 15:04:25 | `cd85891abcf457c23153e288a1afa26363f920fa4f8c2bc22f86af88ce8a1242` |
| 현재 재현 | `1725324233` | `refs/heads/devel@d1831146587b1ac2346f9ed1216a64c2943a02f9` | 2.26.4 | 2026-09-04 14:30:28 | `663e898ddbf8d6efabceea1eb9c112dc4dafbf6fee494718473b1f7cf1b75c73` |

- [최초 전체 flow](alert_186_first_dataflow.json)
- [현재 전체 flow](alert_186_current_dataflow.json)

두 JSON은 해당 SARIF의 `js/insufficient-password-hash` result에서 판정에 필요한 provenance,
source·sink와 code flow를 정규화한 기계 판독 증적이다. 각 파일은 네 개 code flow의 모든 thread-flow
location을 생략 없이 포함한다. 사용자 암호, token 또는 secret 값은 없으며 source 식별자와 저장소
상대 경로만 담는다. PR diff를 불필요하게 키우지 않도록 기계용 JSON은 한 줄로 직렬화했다.

다음 명령으로 같은 분석에서 다시 수집할 수 있다. `<analysis-id>`와 provenance 값은 위 표를 사용한다.

```bash
gh api -H 'Accept: application/sarif+json' \
  repos/edwardkim/rhwp/code-scanning/analyses/<analysis-id> \
  | jq '.runs[].results[] | select(.ruleId=="js/insufficient-password-hash")'
```

GitHub가 분석 보존 기간 뒤 원본 SARIF를 제거하더라도 committed JSON과 파일 hash로 이번 판정의 입력을
검증할 수 있다.

## flow 구조

두 분석 모두 네 flow이며 각 location 수는 `61`, `64`, `60`, `63`이다. 연속된 같은 파일을 하나로
접으면 네 flow 모두 다음 구조다.

```text
rhwp-studio/src/main.ts
  -> rhwp-studio/src/core/document-font-status.ts
  -> rhwp-studio/src/core/font-substitution.ts
  -> scripts/font_rule_runtime_snapshot.mjs
  -> scripts/font_rule_ledger.mjs
  -> scripts/font_rule_runtime_snapshot.mjs
  -> scripts/font_rule_ledger.mjs
```

line number를 제외하고 각 location의 파일 경로와 CodeQL message만 정규화한 네 flow의 SHA-256은 두
분석 모두 다음과 같다.

```text
e05bfecb52277f618bb35577d62e2a0997d0cb8f5639b368e56c81c93eb410f5
```

그러므로 CodeQL 2.26.3에서 2.26.4로 바뀐 뒤에도 flow topology는 같고, 서로 다른 hash는 중간에 추가된
코드로 line number가 이동했기 때문이다.

## 실제 원인

CodeQL JavaScript 라이브러리의 `SensitiveCall`은 호출한 함수 이름이 민감정보를 뜻하면 그 **호출
반환값**을 `SensitiveNode`로 만든다. 현재 source message도 다음 두 호출 반환값을 가리킨다.

- `loadDocumentWithPassword()`
- `loadPasswordProtectedDocument()`

실제 호출의 반환값은 `DocumentInfo`이며 필드는 version, section/page count, encrypted 상태,
HWP3 variant, fallback font, `fontsUsed`, HWPX font substitution metadata뿐이다. raw password 필드는
없다. 입력 암호는 `main.ts`의 지역 변수로 WASM open에 전달되고 `finally`에서 지역 참조를 비운다.

그 뒤의 비현실적 연결은 `font-substitution.ts`의 module-global `_resolveCache`에서 발생한다. 브라우저
Studio가 문서 폰트를 해소하는 실행과 Node 계측 스크립트가 후보 폰트를 해소하는 실행은 별도
프로세스지만, 단일 CodeQL database의 정적 분석은 같은 module cache에 대한 write/read로 연결한다.

## 개선 판단

현재 source가 password 값 자체가 아니라 password 이름을 가진 command의 metadata 반환값이므로,
query 제외나 SHA-256 교체보다 command/query 분리가 우선이다. password open command는 반환값으로
metadata를 운반하지 않고, 성공한 문서의 `DocumentInfo`는 별도 `getDocumentInfo()` query로 읽는다.

이 변경 뒤에도 CodeQL flow가 남으면 내부 WASM call이나 module-global cache를 새 source로 연결했는지
새 SARIF로 비교한다. 그 증거 없이 광범위한 sanitizer나 분석 경로 제외를 추가하지 않는다.
