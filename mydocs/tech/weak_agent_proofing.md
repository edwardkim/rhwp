---
kind: decision
status: active
canonical: mydocs/tech/weak_agent_proofing.md
last_verified: 2026-08-01
---

# 경량 에이전트 내성(weak-agent-proofing) — CLI·MCP 계약 확장 4건

> "도구 호출만 가능한 경량 에이전트"가 rhwp 표면을 안전하게 쓰도록, 실증된 에이전트 실패
> 유형을 **CLI `--json` 봉투·MCP 도구 계약**으로 흡수하는 설계 결정을 보존한다.
> 설계 이슈는 [#3630](https://github.com/edwardkim/rhwp/issues/3630),
> 로드맵 [#3608](https://github.com/edwardkim/rhwp/issues/3608)의 M2/M3/M16 확장 근거다.
> 원칙 서술만 사용하며 특정 제품·회사는 언급하지 않는다. 근거는 각주 URL 로만 남긴다.

## 1. 배경 — 에이전트 실패 유형 분류

공개 연구·실무 보고에서 반복 확인되는 코딩 에이전트 실패 유형은 다음 7가지로 수렴한다.
어느 것도 특정 구현의 결함이 아니라, 자율 도구 호출 루프라는 구조가 만드는 공통 약점이다.

| # | 실패 유형 | 실증 근거 (각주) |
| --- | --- | --- |
| F1 | **존재하지 않는 이름 환각** — 없는 명령·플래그·패키지·파일 경로를 사실처럼 생성. 코드 생성 표본에서 존재하지 않는 패키지 이름 20만 종 이상이 관측됐고, 환각 이름을 선점하는 공급망 공격까지 성립했다 | [^1] [^2] |
| F2 | **검증 없는 완료 선언** — 커밋·파일 수정·명령 실행이라는 "활동"을 "달성"으로 오인하고 완료를 선언. 코딩 과제 실패의 절반 이상이 검증 부족에서 나온다는 벤치마크 보고가 있다 | [^3] [^4] |
| F3 | **시각 결과 확인 불가** — 화면·렌더 결과를 볼 수 없는 에이전트는 시각 산출물을 눈으로 검증하지 못한 채 코드만 보고 통과 판정한다 | [^5] [^6] |
| F4 | **에러 무시·맹목 재시도 루프** — 실패 응답에 대한 처리 지침이 없으면 같은 호출을 반복하는 루프에 빠진다. 도구 오류가 "정보 없는 실패"일 때 루프가 가장 심하다 | [^7] [^8] |
| F5 | **장기 계획 붕괴** — 긴 작업에서 초기 계획이 컨텍스트에서 밀려나고 최근 도구 출력에 끌려가며 목표를 잃는다. 컨텍스트가 길어질수록 모든 모델의 출력 품질이 저하된다는 측정이 있다 | [^9] [^10] |
| F6 | **미완성 산출물** — 토큰 한도·중단으로 부분 diff·부분 파일이 남고, 에이전트나 후속 파이프라인이 이를 완성본으로 오인한다 | [^11] [^3] |
| F7 | **조용한 의미 손상** — 표면상 성공(파싱 통과·테스트 통과)이지만 의미가 달라진 결과를 확신에 차서 제출한다 | [^4] |

## 2. 설계 원칙 — 표면이 검증을 대신한다

에이전트는 결국 **CLI `--json` 봉투와 MCP 도구 결과**로만 rhwp 를 쓴다. 따라서 내성은
프롬프트·문서가 아니라 그 두 계약 표면에 심는다.

1. **최소 에이전트 가정**: 시각 능력·장기 기억·자기 검증 습관을 전제하지 않는다.
2. **검증은 봉투에 동봉**: 검증을 따로 호출해야 하는 구조는 F2 앞에서 무력하다.
   작업 응답 봉투 안에 검증 결과를 실으면, 검증을 잊는 에이전트도 검증된 결과만 받는다.
3. **실패는 다음 호출 안내**: `isError` 결과는 막다른 벽이 아니라 기계가 따라 부를 수 있는
   교정 경로를 실어야 한다. 정보 없는 실패는 F4 루프의 원료다.
4. **볼 곳을 표면이 지정**: 바뀐 페이지만 가리키면 눈검증 비용이 상수로 떨어진다.
5. **오제안 금지**: 확신 없는 안내는 생략(null)이 낫다. 환각을 막으려는 기능이 새 환각
   (잘못된 제안·거짓 정밀도·부분 목록)을 만들면 안 된다.

## 3. 실패 유형 → 표면 계약 매핑

| 실패 유형 | 기존 표면 자산 (재사용 지점) | 신규 보완 |
| --- | --- | --- |
| F1 이름 환각 | `capabilities`/`capabilities --mcp` 자기서술(#3263) — 명령·도구 목록의 단일 출처. 드리프트 가드(`capabilities_covers_every_help_command` 등)가 목록 신선도를 보증. 알 수 없는 명령은 exit 2(#2707) | **P1 did-you-mean** |
| F2 검증 없는 완료 선언 | `export-hwpx`/`convert` 의 `--verify` — 봉투 `verify:{identical,diffCount}` + exit 3/4(#3596), `ir-diff`(#3274) | **P2 편집 `--verify` 내장** |
| F7 조용한 의미 손상 | 위와 동일 — IR 왕복 차이 검출이 "표면상 성공" 뒤의 의미 변형을 잡는다 | **P2** (동일 보완) |
| F3 시각 확인 불가 | `export-svg`·`thumbnail`·M2 `hwp_doc_render_page`(#3609) — 렌더 수단은 있으나 "어디를 볼지"는 에이전트 몫 | **P3 changedPages** |
| F4 에러 무시·재시도 루프 | MCP `isError` 규약(실행 실패 = 프로토콜 오류가 아닌 도구 결과)과 [에이전트 실패 사전](../manual/agent_troubleshooting_guide.md)의 증상별 처방 — 단 처방은 사람용 산문 | **P4 nextCall 교정 제안** |
| F5 장기 계획 붕괴 | M16 prompts 템플릿(작업 절차의 표면 내장)과 문서 지도 3단 진입(`CLAUDE.md` → `mydocs/README.md` → manual·tech 지도) — 계획을 컨텍스트가 아니라 표면·문서에 보존 | 연결만 (신규 없음) |
| F6 미완성 산출물 | 실패 경로 stdout 순수성·부분 매니페스트 금지 계약 — 단건 실패 시 stdout 0바이트(`capabilities.jsonContract.failure` 로 자기서술됨) | 연결만 (신규 없음) |

F5·F6 은 기존 계약이 이미 담당하므로 이 문서는 연결만 기록한다. 신규 구현은 P1~P4 네 건이다.

## 4. 신규 계약 4건

### P1. did-you-mean — 알 수 없는 이름의 즉시 교정 (F1)

**막는 실패**: 없는 명령·도구 이름이 "알 수 없는 명령/도구"라는 정보 없는 실패로 끝나면,
경량 에이전트는 같은 환각 이름을 변주하며 루프한다(F1→F4 연쇄).

**대상 표면**

- CLI: 알 수 없는 명령·알 수 없는 옵션의 **exit 2 경로**(#2707, `EXIT_USAGE`).
- MCP: `mcp-serve` 의 알 수 없는 도구 `isError` 경로(`handle_tool_call` 의 tool_defs 미일치 분기).

**계약 명세**

- CLI stderr 마지막 줄에 힌트 1줄을 추가한다. exit 2·stdout 0바이트 불변.

  ```text
  # 전
  오류: 알 수 없는 명령입니다 - exprot-svg
  rhwp v…
  사용법: rhwp <명령> [옵션]
  # 후 — 마지막 줄 추가
  오류: 알 수 없는 명령입니다 - exprot-svg
  rhwp v…
  사용법: rhwp <명령> [옵션]
  힌트: 가장 가까운 명령은 'export-svg' 입니다
  ```

- MCP `isError` 텍스트에 구조화 필드를 동봉한다(P4 의 오류 봉투 JSON 승격 위에 얹힘).

  ```json
  // 전: content[0].text = "알 수 없는 도구: hwp_export_sgv"
  // 후:
  { "error": "알 수 없는 도구: hwp_export_sgv", "didYouMean": ["hwp_export_svg"] }
  ```

- 후보 출처는 `capabilities` 선언과 **같은 단일 출처**(CLI 는 명령 선언 목록, MCP 는
  `tool_defs`)다. 별도 하드코딩 목록 금지 — 드리프트 원천 차단.
- 거리 계산은 편집 거리(레벤슈타인) 소형 자체 구현(신규 의존성 없음). 임계값
  (거리 ≤ 2 또는 이름 길이의 1/3) 초과면 힌트 줄·`didYouMean` 필드를 **생략**한다(원칙 5).

**계약 테스트 초안**

- `unknown_command_stderr_last_line_suggests_nearest` — `exprot-svg` → 힌트 줄 + exit 2.
- `unknown_command_far_name_has_no_hint` — `zzzz` → 힌트 줄 없음(오제안 0).
- `mcp_unknown_tool_error_includes_did_you_mean` — `hwp_export_sgv` → `didYouMean` 배열.
- `did_you_mean_candidates_match_capabilities` — 후보가 자기서술 목록의 부분집합.

**무회귀 가드**: `tests/cli_exit_codes.rs`(exit 2·stdout 0바이트 기존 계약),
`tests/cli_json_contract.rs::capabilities_covers_every_help_command`(목록 단일 출처),
`tests/mcp_server_contract.rs`(알 수 없는 도구 `isError: true` 기존 계약).

### P2. 편집 `--verify` 내장 — 저장과 검증의 원자화 (F2·F7)

**막는 실패**: 편집 후 검증 호출을 잊은 에이전트가 저장 왕복에서 손상된 산출물을 완성본으로 선언.

**대상 표면**

- CLI: `edit fill-fields` · `edit replace-text` · `edit set-cell` 에 `--verify` 옵션
  (각 `--json` 병용 가능). 무상태 MCP 도구 `hwp_fill_fields`/`hwp_replace_text`/`hwp_set_cell`
  은 CLI 인자 템플릿에 `--verify` 배선으로 승계.
- MCP 세션: `hwp_doc_save { "verify": true }` — M3 `hwp_doc_verify` 와 같은 코어 경로를
  save 에 내장(별도 호출을 기억할 필요 제거, 원칙 2).

**계약 명세**

- 검증 의미론: **편집 반영 후 메모리 IR ↔ 저장 산출물 재파싱 IR** 의 내부 ir-diff.
  의도된 편집 diff 는 비교 양쪽에 이미 반영돼 있으므로 `identical: true` 가 정상이다 —
  `export-hwpx --verify` 와 동일 의미론·동일 어휘를 그대로 승계한다(#3596).
- 봉투 필드: `verify: { "identical": bool, "diffCount": number }`.
  `--verify` 미지정 시 `verify: null`(#3596 export-hwpx 봉투의 Null 규약 동일).

  ```json
  // 전 (edit replace-text --json)
  { "schemaVersion": "1.0", "source": "a.hwpx", "find": "갑", "replace": "을",
    "caseSensitive": false, "dryRun": false, "replacedCount": 3,
    "output": "a.out.hwpx", "outputFormat": "hwpx" }
  // 후 — 필드 추가(스키마 정책: 필드 추가는 schemaVersion 범프 없음)
  { "schemaVersion": "1.0", "source": "a.hwpx", "find": "갑", "replace": "을",
    "caseSensitive": false, "dryRun": false, "replacedCount": 3,
    "output": "a.out.hwpx", "outputFormat": "hwpx",
    "verify": { "identical": true, "diffCount": 0 } }
  ```

- **exit 코드: 3 채택** (0 유지안 기각). 결정 근거:
  1. 비교 기준이 "편집 후 IR ↔ 저장 재파싱 IR"이므로 의도된 편집과 무관하게
     `identical: true` 가 정상 — `export-hwpx --verify` 의 exit 3 과 의미론이 같아
     기존 exit 사전(`capabilities.exitCodes` 의 `"3"`) 을 그대로 재사용한다.
  2. 경량 에이전트·셸 파이프라인이 확실히 소비하는 신호는 exit 코드다. 판정을 봉투
     데이터로만 주면 봉투를 읽지 않는 소비자에서 F2 가 재발한다.
  3. MCP 경유 시에는 기존 `run_cli_tool` 규약(exit 3 이라도 stdout JSON 이 있으면 도구
     결과로 전달)이 그대로 적용돼, 에이전트는 `verify.identical` 로 판정한다 — 이중 계약 불필요.
  - 재파싱 실패는 판정 불가로 stdout 0바이트 + exit 3 계열(#3596 규약 승계).
- `hwp_doc_save { verify: true }` 결과 봉투는 무상태판과 동형의 `verify` 필드를 담는다.

**계약 테스트 초안**

- `edit_replace_text_verify_roundtrip_identical` — 정상 편집 → `identical: true`·exit 0.
- `edit_verify_diff_reports_and_exits_3` — 의도적 결함 주입 → `verify.identical: false`·exit 3.
- `edit_verify_null_without_flag` — 미지정 시 `verify: null`·동작 무변경.
- `mcp_doc_save_verify_matches_stateless_envelope` — 세션판·무상태판 `verify` 동형.

**무회귀 가드**: `tests/edit_replace_text_contract.rs`·`edit_set_cell_contract.rs`·
`edit_fill_fields_contract.rs`(기존 봉투 필드), `edit_format_preserve_contract.rs`(형식 보존),
`convert_verify_corpus_ratchet.rs`(verify 어휘·exit 3/4 규약), `mcp_session_edit_contract.rs`.

### P3. changedPages — 눈검증 대상의 표면 지정 (F3)

**막는 실패**: 시각 검증 능력이 없는(또는 전 페이지 렌더가 비싼) 에이전트가 편집 결과를
렌더 확인 없이 통과시키거나, 전수 렌더로 예산을 소진.

**대상 표면**

- CLI: 편집 계열 `--json` 봉투 3종(`edit fill-fields`/`replace-text`/`set-cell`).
- MCP 세션: `hwp_doc_fill_fields`·`hwp_doc_replace_text`·(M3) `hwp_doc_set_cell` 결과 봉투.

**계약 명세**

- 봉투에 `changedPages: [n, …] | null` 필드를 추가한다. 페이지 번호는 **0 기준**
  (기존 렌더 도구 `page` 파라미터의 "0부터 시작" 어휘와 동일).
- **산출 근거**: 편집 명령은 변경 위치의 문단 인덱스를 이미 안다(search/fields 주소
  어휘의 `paragraph`, 필드 위치의 `para_index`). 저장 전 레이아웃 재계산이 주는
  페이지네이션 결과(= `dump-pages` 가 소비하는 코어 경로)의 페이지별 문단·줄 커버리지와
  변경 문단 집합의 교집합으로 페이지 목록을 계산한다.
- **`null` 규약**: 레이아웃을 재계산하지 않는 경로이거나 문단→페이지 매핑을 확정할 수
  없으면 `changedPages: null`("전체를 보라"). **부분 목록 금지** — 빠뜨린 페이지가 거짓
  통과를 만들므로, 불확실하면 정밀한 척하지 않고 null 로 내린다(원칙 5).

  ```json
  // 전 (edit set-cell --json)
  { "schemaVersion": "1.0", "source": "a.hwpx", "table": 0, "row": 2, "col": 1,
    "oldText": "10", "newText": "20", "dryRun": false, "overflow": false,
    "output": "a.out.hwpx", "outputFormat": "hwpx" }
  // 후 — 필드 추가
  { "schemaVersion": "1.0", "source": "a.hwpx", "table": 0, "row": 2, "col": 1,
    "oldText": "10", "newText": "20", "dryRun": false, "overflow": false,
    "output": "a.out.hwpx", "outputFormat": "hwpx",
    "changedPages": [4] }
  ```

- 에이전트 계약: `changedPages` 가 가리키는 페이지만 M2 `hwp_doc_render_page { page: n }`
  로 렌더하면 눈검증 루프가 상수 비용으로 폐쇄된다.

**계약 테스트 초안**

- `edit_set_cell_changed_pages_contains_cell_page` — 특정 페이지 셀 수정 → 해당 페이지 포함.
- `edit_changed_pages_null_when_layout_unknown` — 매핑 불가 경로 → `null`(부분 목록 0).
- `mcp_session_edit_changed_pages_matches_stateless` — 세션판·무상태판 동형.

**무회귀 가드**: `tests/edit_set_cell_contract.rs` 등 편집 봉투 기존 계약,
`tests/dump_pages_cli.rs`(페이지네이션 계약), `tests/mcp_session_edit_contract.rs`.

### P4. nextCall — 오류 봉투의 JSON 승격과 교정 제안 (F4)

**막는 실패**: 정보 없는 `isError` 텍스트 앞에서 같은 호출을 반복하는 맹목 재시도 루프.

**대상 표면**: `mcp-serve` 의 `tool_error` 전 경로 — 세션 도구 실패(경로 없음·핸들 없음·
파싱 실패 등), 알 수 없는 도구, `run_cli_tool` 의 stdout 0바이트 실패.

**계약 명세**

- `tool_error` 텍스트를 순수 문자열에서 JSON 으로 승격한다.
  `isError: true`·`content[0].text` 구조는 불변이고, 텍스트가 JSON 직렬화 문자열이 되며
  `structuredContent` 도 동봉한다(`tool_ok_text` 와 동형 규약).

  ```json
  // 전: content[0].text = "필드를 찾을 수 없습니다: 성명2"
  // 후:
  { "error": "필드를 찾을 수 없습니다: 성명2",
    "nextCall": { "name": "hwp_fields", "arguments": { "path": "a.hwpx" },
                  "why": "실제 필드 목록을 확인한 뒤 정확한 이름으로 다시 호출" } }
  ```

- **하위호환**: `error` 필드가 **기존 메시지 원문을 그대로** 담는다. 텍스트를 사람이
  읽거나 부분 문자열로 매칭하던 기존 소비자는 `error` 필드에서 같은 원문을 얻는다.
- 1차 `nextCall` 대상([실패 사전](../manual/agent_troubleshooting_guide.md)의 산문 처방 중
  기계화 가능 항목부터):
  - 없는 필드명 → `nextCall: hwp_fields`(실제 필드 목록 확인 유도)
  - 없는 표·범위 밖 셀 좌표 → `nextCall: hwp_export_tables`(실제 격자 확인 유도)
  - 알 수 없는 도구 → P1 `didYouMean` 과 결합, 교정된 이름을 `nextCall.name` 으로 제안
- `nextCall.name` 은 생성 시점에 `tool_defs` 대조로 **실존 도구만** 허용. 교정 경로를
  제안할 수 없는 실패는 `nextCall` 필드를 생략한다(원칙 5) — 루프를 꺾는 것이 목적이지
  새 루프를 만드는 것이 아니다.

**계약 테스트 초안**

- `mcp_tool_error_text_is_json_with_error_field` — 전 실패 경로 공통·원문 보존(하위호환).
- `mcp_missing_field_error_suggests_hwp_fields` — 없는 필드 → `nextCall.name == "hwp_fields"`.
- `mcp_next_call_name_always_registered` — 제안 이름이 항상 `tool_defs` 에 실존.
- `mcp_unsuggestable_error_omits_next_call` — 제안 불가 실패 → 필드 생략(오제안 0).

**무회귀 가드**: `tests/mcp_server_contract.rs`·`mcp_session_edit_contract.rs`·
`mcp_session_query_contract.rs` 의 기존 `isError` 검사(오류 문구 매칭이 있다면 `error`
필드 기준으로 갱신 — 그 갱신 자체가 하위호환 검증이 된다).

## 5. 로드맵 연결 (#3608)

| 제안 | 편입 위치 | 근거 |
| --- | --- | --- |
| P3 changedPages | **M2 확장** | M2 의 `hwp_doc_render_page` 와 한 쌍 — 힌트가 렌더 대상을 지정해야 눈검증 루프가 상수 비용으로 폐쇄된다 |
| P2 편집 `--verify` 내장 | **M3 확장** | M3 의 `hwp_doc_verify`(저장 전 내부 ir-diff 자기검증) 항목을 무상태 편집 축·save 내장으로 확대 |
| P1 did-you-mean | **M16 인접** | `capabilities` 자기서술의 실패 경로 확장 — P4 와 함께 "오류·안내 표면" 묶음으로 M16(MCP 표면 v2) 인접 신설 |
| P4 nextCall | **M16 인접** | `isError` 계약의 v2 — 오류 봉투 JSON 승격은 M16 의 표면 개정과 같은 층위 |

네 건 모두 [에이전트 표면 플레이북](../manual/agent_surface_playbook.md)의 절차(이슈등록 →
red 계약 테스트 → 코어 재사용 구현 → green + clippy/fmt → 증적 2종 → PR)를 따른다.

## 각주 — 실증 근거

[^1]: <https://en.wikipedia.org/wiki/Slopsquatting> — 환각된 패키지 이름 선점 공격의 성립.
[^2]: <https://www.helpnetsecurity.com/2025/04/14/package-hallucination-slopsquatting-malicious-code/> — 코드 생성 표본에서 존재하지 않는 패키지 추천 비율·고유 환각 이름 규모 측정.
[^3]: <https://arxiv.org/pdf/2601.08806> — 코딩 과제 실패에서 검증 부족이 차지하는 비중, 개방 루프 실행(생성=완료 오인) 분석.
[^4]: <https://arxiv.org/pdf/2603.25764> — 표면상 성공 뒤의 조용한 의미 손상(확신에 찬 오답) 분석.
[^5]: <https://medium.com/jonathans-musings/ai-has-a-ux-validation-problem-cf8d93ea4e92> — 시각 결과를 보지 못한 채 작성되는 화면 코드의 검증 공백.
[^6]: <https://tweag.github.io/agentic-coding-handbook/WORKFLOW_VISUAL_FEEDBACK/> — 스크린샷 왕복을 통한 시각 피드백 루프의 필요성.
[^7]: <https://dev.to/alanwest/how-to-stop-your-llm-agent-from-looping-itself-into-oblivion-27eh> — 정보 없는 도구 오류가 만드는 맹목 재시도 루프와 해법(정보 있는 오류 반환).
[^8]: <https://www.agentmonth.com/fixes/agent-stuck-in-loop/> — 동일 도구 호출 반복의 원인·중단 장치 정리.
[^9]: <https://arxiv.org/html/2606.22953> — 계획이 컨텍스트에서 밀려나며 장기 작업이 붕괴하는 구조 분석.
[^10]: <https://www.morphllm.com/context-rot> — 컨텍스트 길이 증가에 따른 전 모델 출력 품질 저하 측정.
[^11]: <https://dev.to/vitramir/codex-most-common-issues-and-feature-requests-293h> — 공개 이슈 대량 분석: 토큰 한도로 인한 작업 중단·세션 소실이 반복 보고됨.
