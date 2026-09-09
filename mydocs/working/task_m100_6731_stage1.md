---
kind: snapshot
status: active
canonical: mydocs/plans/task_m100_6731.md
issue: 6731
last_verified: 2026-09-05
---

# #6731 Stage 1 — SARIF 기준선 보존 결과

## 결과

CodeQL alert #186의 최초·현재 JavaScript/TypeScript SARIF result를
[`mydocs/tech/investigations/issue-6731/`](../tech/investigations/issue-6731/)에 보존했다.

| 항목 | 최초 분석 | 현재 분석 |
| --- | --- | --- |
| 분석 ID | `1659063897` | `1725324233` |
| CodeQL | 2.26.3 | 2.26.4 |
| 대상 | `devel@5057a7fcaf0` | `devel@d1831146587` |
| flow 수 | 4 | 4 |
| location 수 | 61, 64, 60, 63 | 61, 64, 60, 63 |
| path/message topology hash | `e05bfecb52277f618bb35577d62e2a0997d0cb8f5639b368e56c81c93eb410f5` | 동일 |

line number만 현재 파일 위치로 이동했고 source, cache 교차점, Node snapshot과 SHA-256 sink의 구조는
동일하다. 따라서 alert는 특정 CodeQL 버전의 일회성 변화가 아니라 현재 source·공유 module 구조에서
반복 가능한 결과다.

## 원인 판정

1. CodeQL의 함수 이름 휴리스틱이 `loadDocumentWithPassword()`와
   `loadPasswordProtectedDocument()` 호출의 반환값을 password source로 만든다.
2. 반환값은 raw password가 아니라 `DocumentInfo`다.
3. `DocumentInfo.fontsUsed`가 문서 폰트 상태를 거쳐 module-global `_resolveCache`에 기록된다.
4. 별도 Node 계측 실행이 같은 cache를 읽는 것으로 정적 연결되어 runtime snapshot의
   `resolvedFace`가 오염된 값으로 취급된다.
5. snapshot canonical JSON의 SHA-256 무결성 digest가 password hash sink로 귀속된다.

메인테이너가 확정한 `used in tests`를 유지하며 alert 분류를 변경하지 않았다. 이슈 제목과 본문은
현행 판정, 분석 ID, 보호 불변식과 완료 조건으로 갱신했고 한글·BOM·`??` 치환 이상이 없음을 API로
재확인했다.

## Stage 2 입력

- password open command가 `DocumentInfo`를 반환하지 않는 RED 계약
- `DocumentInfo` 허용 필드와 raw password 부재 계약
- HWP3·HWP5·HWPX open 뒤 DOM·storage·metadata에 입력 암호가 없는 브라우저 계약
- Node snapshot이 승인된 폰트 projection만 digest 입력으로 쓰는 계약

Stage 2는 위 계약을 먼저 실패시키고, 아직 제품 구현은 바꾸지 않는다.
