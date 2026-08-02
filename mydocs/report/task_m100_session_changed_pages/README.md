---
kind: report
status: active
canonical: mydocs/report/task_m100_session_changed_pages/README.md
last_verified: 2026-08-02
---

# 세션 편집 도구의 changedPages — 눈검증 루프를 세션 안에서 닫는다 (#3719 §6-1)

## 문제

무상태 편집 3종(`edit fill-fields`/`replace-text`/`set-cell`)은 #3712 이후 봉투에
`changedPages` 를 담아 "어느 쪽을 보라"를 지정한다. **세션 편집 3종은 못 냈다.**

세션 쪽이 더 아픈 자리다. `hwp_doc_render_page` 의 도구 설명이 약속하는 문장이
정확히 "편집 직후 눈검증(VLM) 루프가 세션 안에서 닫힌다" 인데, 렌더할 쪽을 지정하는
표면이 없으니 에이전트에게 남는 선택지는 둘뿐이었다.

- 전 페이지 렌더 — 편집 한 번마다 문서 크기에 비례하는 비용.
- 검증 생략 — #3630 실패 유형 3(시각 확인 불가)이 그대로 통과.

#3712 처리 기록도 이 공백을 "**#3704(세션 재조판) 머지 후 후속 적층**" 으로 남겨
두었다. #3704 가 머지되어 세션 편집 3종이 같은 재조판 규약을 공유하게 됐으므로,
이제 그 규약 위에 쪽 지정만 얹으면 된다.

## 구현

새 계산은 없다. 무상태 판이 쓰는 코어 질의
[`DocumentCore::pages_covering_paragraphs`](../../../src/document_core/queries/changed_pages.rs)
를 그대로 재사용하고, **변경 문단을 무엇으로 잡느냐**도 무상태 판과 같은 출처를 쓴다.

| 세션 도구 | 변경 문단 근거 | 무상태 대응 |
|---|---|---|
| `hwp_doc_fill_fields` | `collect_all_fields()` 순회에서 모은 `FieldLocation`(section·para) | `edit fill-fields` 와 동일 |
| `hwp_doc_replace_text` | **치환 전** grep 매치 주소 — 문자열 치환은 문단을 새로 만들지 않아 인덱스를 밀지 않는다 | `edit replace-text` 와 동일 |
| `hwp_doc_set_cell` | `resolve_table_cell` 이 돌려준 표 호스트 문단 | `edit set-cell` 과 동일 |

두 경로가 같은 근거·같은 질의를 쓰므로 **답이 갈라질 수 없다**. 계약 테스트도 값을
직접 적어 두지 않고 두 봉투를 맞대어 본다(§검증).

### 호출 시점이 계약의 절반이다

세션은 편집 뒤에도 같은 인스턴스가 살아 있다. **재조판 전에 쪽을 계산하면 편집 전
레이아웃을 보고한다** — #3704 가 조회 4종에서 고친 스테일과 같은 함정이다. 편집이
쪽을 늘린 경우 그 값은 단순히 부정확한 정도가 아니라 "존재하지 않는 쪽" 이거나
"방금 만들어 낸 쪽을 빠뜨린" 목록이 된다.

`pages_covering_paragraphs` 는 진입에서 `paginate_if_needed()` 를 부르므로, 편집 →
질의 순서만 지키면 시점 규약이 저절로 지켜진다. 세 도구 모두 기존
`repaginate_if_needed()` 지점(fill/replace) 또는 셀 편집 코어의 재조판(set_cell)
**뒤에** 질의를 둔다. 이미 조판이 맞은 두 번째 호출은 dirty 구역이 없어
`paginate_pass` 가 전 구역을 `continue` 로 건너뛴다 — 사실상 무비용이다.

### null·빈 목록 규약

- 대상 문단이 **하나라도** 조판 커버리지 밖이면 전체 `null`. 부분 목록 금지 —
  빠뜨린 쪽이 거짓 통과를 만든다(#3630 P3, 원칙 5).
- **변경이 없으면 빈 목록**이지 `null` 이 아니다. `null` 은 "확정 불가, 전체를 보라"
  라서, 치환 0건·notFound 뿐인 호출마다 전수 렌더를 유도하게 된다. 무상태
  `replace-text` 가 0건에서 `null` 을 내는 것은 **산출 파일이 없다**는 별개 사유이고
  (`wrote_output` 분기), 세션에는 그 사유가 없다. 무상태 `fill-fields` 는 채운 것이
  없을 때 이미 `[]` 를 낸다 — 세션 3종은 이 축을 하나로 통일한다.

### 자기서술

세션 도구는 `capabilities --mcp` 의 `tools[]` 에 실리지 않는다(그 목록은 무상태
도구 전용이고, 세션 도구는 `mcp_serve::served_tools()` 가 이름·설명·inputSchema 만
내보낸다 — `outputFields` 축이 아예 없다). 그래서 **세션 도구가 산출을 자기서술하는
유일한 채널인 `description`** 에 `changedPages` 규약을 적었다. 없는 필드를 세션
도구에만 새로 만들면 `tools/list` 가 무상태 도구와 다른 모양이 되어
`tools_list_matches_capabilities_manifest` 가 지키는 단일 출처가 흐려진다.

## 실측 (evidence.txt 원문)

| # | 확인 | 결과 |
|---|---|---|
| 1 | 세션 fill (`회사명`, field-01.hwp 3쪽) | `changedPages:[0]` |
| 2 | 세션 replace (`마케팅`→`기획`) | `changedPages:[0]` |
| 3 | 무변경 2종 (치환 0건 · notFound) | `changedPages:[]` |
| 4 | 무상태 `edit fill-fields`/`replace-text` 대조 | 둘 다 `[0]` — **동형** |
| 5 | 세션 set_cell vs 무상태 set-cell (table-001.hwp) | 둘 다 `[0]` — **동형** |
| 6 | 재조판 시점: `회사명`에 5,000자 | 3쪽 → **11쪽**, `changedPages:[0..8]` |
| 6' | ⑥ 의 8쪽을 곧바로 `hwp_doc_render_page` | `isError:false`, SVG **182,206 bytes** |

⑥ 이 이 작업의 핵심 실측이다. 편집 전 문서는 3쪽이라 3~8쪽은 **존재하지 않았다** —
재조판 전에 계산했다면 낼 수 없는 값이고, 그 쪽들이 그 자리에서 렌더된다는 것이
"눈검증 루프가 세션 안에서 닫힌다"의 실물 증거다.

필드 이름·표 좌표·치환어는 전부 `rhwp fields --json` / `export-tables --json` 으로
문서에서 읽어 썼다(계약 테스트도 같은 방식 — 하드코딩하면 샘플이 바뀔 때 시험이
조용히 껍데기가 된다).

## 검증

- 신규 [`tests/mcp_session_changed_pages_contract.rs`](../../../tests/mcp_session_changed_pages_contract.rs) 6건:
  - fill·replace·set_cell 각각 **무상태 봉투와 changedPages 가 같은지** (값을 적지 않고 맞대어 본다)
  - 지정한 쪽이 **그 자리에서 렌더되는지** — 쪽을 늘리는 채움으로 재조판 시점을 함께 고정하고,
    "이 채움이 쪽을 늘린다"를 별도 assert 로 둬서 샘플이 바뀌어 전제가 깨지면
    **시험이 무의미해진 사실 자체가 실패로 드러난다**
  - 무변경 2종이 `[]` 인지 / 세 도구 설명이 `changedPages` 를 광고하는지(선언↔봉투 드리프트 가드)

| 게이트 | 결과 |
|---|---|
| `cargo build --release --bin rhwp` | 성공 (rhwp v0.8.2) |
| 신규 `mcp_session_changed_pages_contract` | **6/6** |
| 세션 무회귀 `mcp_session_edit`(7) · `query`(6) · `setcell`(6) · `view`(5) | 24/24 |
| 드리프트 가드 `mcp_server_contract`(22, `tools_list_matches_capabilities_manifest`·`every_declared_input_property_is_wired_to_the_cli` 포함) · `mcp_arg_validation_contract`(9) · `agent_profile_router_contract`(7) | 38/38 |
| 무상태 축 무회귀 `cli_json_contract`(26) · `changed_pages_contract`(5, #3712) | 31/31 |
| **합계** | **99 passed / 0 failed** |
| `cargo clippy --release -- -D warnings` | 경고 0 |
| `cargo clippy --release --test mcp_session_changed_pages_contract -- -D warnings` | 경고 0 |
| fmt (변경 파일 2개) | clean |

fmt 은 이 PC 에서 `cargo fmt --all` 이 `os error 206` 으로 죽으므로 변경 파일만
`rustfmt --edition 2021 --config-path rustfmt.toml --config newline_style=Auto --check`
로 돌렸다.

## 남은 것

- **`null` 강등은 이 저장소 코퍼스에서 한 번도 재현되지 않았다.** 샘플 348개에
  `search` 를 돌려 조판 쪽이 붙지 않은 매치를 찾았고(0건), 누름틀이 있는 샘플 30개에
  `fill-fields` 를 돌려 `changedPages:null` 을 찾았다(0건). 경로는 구조적으로 살아
  있다 — `pages_covering_paragraphs` 는 대상 문단이 `PageItem` 어디에도 없으면 `None`
  을 낸다 — 지만 실측 근거는 아직 없다. 실물이 나오면 그 문서를 픽스처로 고정하는
  것이 다음 단계다.
- 세션 편집 도구가 늘어나면 "편집 → 재조판 → `changed_pages_value`" 순서를
  체크리스트에 넣어야 한다(#3704 가 남긴 규약에 한 줄 추가).
