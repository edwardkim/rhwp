# 스킬 로컬 봉투 전사

생성 장의 실측 JSON 을 `.claude/skills/rhwp-codex/fixtures/envelopes/` 로 추출했다.
출처는 각 파일의 `sourceChapter` 필드다.

## 읽는 법

- `kind: live` — 생성 장에 bash+JSON 이 있었다.
- `envelope` — 절단된 표본 그대로. 필드를 보강하지 않았다.
- `untrustedContent` / `untrustedFields` — C3.
- `schemaVersion` 은 보통 `"1.0"`.

## 쓰지 않는 법

- 이 JSON 을 라이브 오라클처럼 재실행 결과와 바이트 비교하지 말 것 (절단본이다).
- 생성 장에 되붙여 넣지 말 것.
- 계약만 명령에 빈 봉투를 만들지 말 것.

전체 키 정의는 지식지도 §2-2.
