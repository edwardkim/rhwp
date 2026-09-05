# `rhwp capabilities` · `--mcp` · `--search`

첫 호출이 층마다 다르다. 잘못된 첫 호출은 "도구가 없다"는 오판으로 이어진다.
트랜스크립트: [`../fixtures/transcripts/`](../fixtures/transcripts/).
검색 픽스처: [`../fixtures/search/`](../fixtures/search/).

## `rhwp capabilities`

인자 없는 호출이 CLI 자기서술이다. **언제나 JSON** 이다. `--json` 을 붙이지 않는다.

읽어야 할 키:

- `version` — `rhwp --version` 과 같은 원천 (가드 ①)
- `commands[]` — `name` · `summary` · `json` · `recordFields` · `subcommands`
- `formats.read` / `formats.write`
- `exitCodes`
- `jsonContract` — stdout 순수성, 실패 시 0바이트, 출처 표지 정책
- `batch` — 서브커맨드, stdin 규칙, `mcp.excluded`
- `schemaRegistry`

`available:false` 인 명령(`export-png` + `requiresFeature: native-skia`)을
부르면 exit 2 와 함께 feature 안내가 나온다. 목록에 있다고 지금 바이너리에
있는 것이 아니다.

## `rhwp capabilities --mcp`

무상태 도구 선언. `protocol: "mcp"`. 각 도구:

- `name` · `description` · `inputSchema`
- `cli.command` · `cli.args` (자리표시자)
- `cli.optionalArgs` (있을 때만)
- `outputFields`

세션 도구는 **없다**. `hwp_open` 을 여기서 찾으면 안 된다.

프로필:

```
rhwp capabilities --mcp --profile 행정서식
```

`PROFILES` 가 원천. 없는 이름 → `오류: 알 수 없는 프로필` exit 2.
`--profile` 만 주고 `--mcp` 가 없으면 exit 2
(`--profile 은 --mcp 와 함께`).

## `rhwp capabilities --search <키워드>`

`commands[].name` · `summary` · `subcommands[].name/summary` 를
대소문자 무시 부분 문자열로 필터한다. 유사도·LLM 없음.

- 공백으로 여러 단어 → **AND**
- OR 이 필요하면 `--search` 를 두 번
- `--json` 을 붙이면 봉투 `{schemaVersion, tool, version, search, commands}`
- `--search` 뒤에 키워드가 없으면 exit 2
- `--mcp` / `--profile` 과 같이 쓰면 exit 2

`#3884 G4` 때문에 `--search redact` 가 `edit` 를 찾는다. 하위명령을
검색 대상에서 빼면 안 된다.

매치 0건은 오류가 아니다.

```
'없음XYZ' 에 매치하는 명령이 없습니다.
exit=0
```

픽스처 `fixtures/search/없음XYZ.json` 의 `empty: true` 가 이 계약이다.

## 스키마를 뽑을 때

바인딩은 계약을 새로 만들지 않는다.

```
rhwp export-ir-schema --json
rhwp export-capabilities-schema --json
rhwp export-capabilities-schema --bare    # 봉투 없이 스키마 본문
```

`--bare` 는 JSON Schema 도구에 바로 먹인다.

## 하지 말 것

- `rhwp --help` 파싱으로 명령 집합 구축
- `--mcp` 출력을 호스트에 붙여넣고 잊기 (원천을 매번 읽는다)
- 검색 결과 개수를 골든으로 얼리기
- 세션 이름을 `--mcp` 에서 찾아 "없다"고 결론 내기
