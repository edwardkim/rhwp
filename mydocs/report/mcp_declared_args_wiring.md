# MCP 선언-배선 불일치 — 스키마에만 있고 CLI 에 닿지 않는 인자 10건

`capabilities --mcp` 가 내는 `inputSchema` 는 에이전트에게 **계약**이다. 에이전트는 그 스키마를
읽고 인자를 만들어 보낸다. 그런데 `cli.args` 자리표시자에도 `cli.optionalArgs` 에도 없는 인자는
서버가 조용히 버린 뒤 `isError: false` 로 성공을 보고한다. 실패가 아니라 **거짓 성공**이라서
에이전트는 재시도조차 하지 않는다.

## 가장 위험한 형태 — dryRun

`dryRun` 은 "쓰지 마라"는 뜻이다. 이것이 무시되면 되돌릴 수 없는 쓰기가 일어난다.

```console
$ # 요청: 1번 표 (0,0) 칸을 바꿔볼 건데, 실제로 쓰지는 말고(dryRun) 결과는 never.hwp 로.
$ echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"hwp_set_cell",
  "arguments":{"path":"in.hwp","table":1,"row":0,"col":0,
  "text":"DRYRUN이면 안 써야 함","output":"never.hwp","dryRun":true}}}' | rhwp mcp-serve

# 수정 전 응답
{"dryRun":false,"output":"in_cell.hwp", ...}
```

세 가지가 동시에 어긋난다.

1. `dryRun: true` 로 보냈는데 응답은 `"dryRun": false` — 요청이 사라졌다.
2. `output: never.hwp` 를 지정했는데 응답은 `in_cell.hwp` — 경로가 사라졌다.
3. 그리고 **파일이 실제로 기록된다.** 지정한 경로가 아니라 서버 프로세스의 CWD 에.

### 문서 증적

수정 전, `dryRun: true` 요청이 만들어낸 문서다. 표 머리칸 "구 분" 이 실제로 덮어써졌다 —
쓰지 말라고 했는데 쓴 것이다.

![before](assets/task_m100_3712_args/before.png)

수정 후 같은 요청. 응답은 `"dryRun": true` 와 바뀔 예정인 `newText` 를 담고, 파일은
어디에도 생기지 않는다. 원본 문서의 "구 분" 은 그대로다.

![after](assets/task_m100_3712_args/after.png)

## 배선한 인자 10건

| 도구 | 인자 | 전달할 CLI 플래그 | 무시됐을 때의 증상 |
|---|---|---|---|
| `hwp_export_text` | `page` | `-p {page}` | 한 쪽만 요청해도 전체(6쪽)가 온다 |
| `hwp_export_structure` | `mode` | `--mode {mode}` | `outline` 을 줘도 항상 auto 결과 |
| `hwp_fill_fields` | `output` / `dryRun` | `-o {output}` / `--dry-run` | 위 dryRun 사례와 동일 |
| `hwp_replace_text` | `output` / `dryRun` | 〃 | 〃 |
| `hwp_set_cell` | `output` / `dryRun` | 〃 | 〃 |
| `hwp_batch` | `threads` | `--threads {threads}` | 병렬도 지정이 먹지 않는다 |
| `hwp_batch_search` | `threads` | 〃 | 〃 |

실측 확인:

```console
$ # page — 수정 전 pageCount 6, 수정 후 1
$ # mode — 수정 전 항상 "clause", 수정 후 "outline"
$ # dryRun:false + output 지정 — 지정 경로에 정확히 기록(CWD 오염 없음)
```

## presence 플래그의 `false`

`optionalArgs` 는 `args.get(key).is_none()` 으로만 걸렀다. `--dry-run` 처럼 값이 없는
presence 플래그에서 이는 치명적이다 — `dryRun: false` 도 "존재" 이므로 `--dry-run` 이
주입되고, **끄라고 보낸 요청이 켜는 요청이 된다.** JSON 의 `false`/`null` 은 "그 축을
쓰지 않음" 으로 읽도록 고쳤다.

## 재발 차단

`every_declared_input_property_is_wired_to_the_cli` 는 선언된 모든 입력 속성이
`cli.args` 자리표시자이거나 `optionalArgs.when` 임을 요구한다. argv 가 아닌 축으로
전달되는 것(batch 의 `paths`, `password`)만 근거와 함께 `NON_ARGV_PROPERTIES` 에 등재한다.

기존 드리프트 가드(`tools_list_matches_capabilities_manifest`)는 **도구 이름만** 대조해
이 계급을 통째로 못 봤다. 새 가드를 수정 전 매니페스트에 돌리면 정확히 10건을 지목하며
실패한다 — 가드가 공허하지 않음을 확인한 것이다.

`boolean_false_does_not_inject_a_presence_flag` 가 `false` 경로를 따로 못 박는다.
