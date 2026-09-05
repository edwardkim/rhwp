# 레시피 — 프로필이 도구를 막을 때

픽스처: [`../fixtures/exceptions/profile_blocked.json`](../fixtures/exceptions/profile_blocked.json),
[`../fixtures/profiles/`](../fixtures/profiles/).

## 없는 프로필 이름

```bash
rhwp capabilities --mcp --profile 없는직무
# 오류: 알 수 없는 프로필 '없는직무'
# 사용 가능: 경영보고, 행정서식, 데이터분석, 콘텐츠제작, 아카이브검색, 품질검증, 개발통합
# exit=2
```

사용 가능 목록은 `agent_profiles::names()` 가 한 곳.

## 있는 프로필, 없는 도구

`경영보고` 는 조회·요약만 연다. `hwp_fill_fields` 는 목록에 없다.

- `tools/list` 에 없음
- `tools/call` 로도 우회 불가 (`allows_tool`)

`아카이브검색` 은 세션 조회만. `hwp_doc_save` / `hwp_doc_fill_fields` 차단
(`SESSION_READ_TOOLS`).

`개발통합` 은 `tools: &[]` + 세션 필터 없음 — 전 표면.

## 다음 수

1. 도구 이름을 발명하지 않는다 (`hwp_doc_redact` 금지).
2. 직무가 맞으면 그 프로필의 `recipe[]` 를 따른다
   (`capabilities --mcp --profile` 이 같이 준다).
3. 직무가 아니면 프로필을 바꾸거나 `개발통합`.
4. 차단을 `isError` 봉투 판정으로 오독하지 마라. 실행 전 경계다.
