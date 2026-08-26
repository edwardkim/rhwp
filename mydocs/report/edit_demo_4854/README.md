# [#4854] 전/후 증빙 — 세션 검색의 이어보기

`cursor-before-after.png` 는 **같은 문서·같은 검색어·같은 창 크기로 똑같은 요청 3건**을
두 바이너리에 던진 결과다.

- 문서: `samples/hwp3-sample.hwp` · 검색어 `"의"` · 전체 매치 276건 · `maxMatches=3`
- BEFORE: `upstream/devel` `627c8c49a` 와 `src/mcp_serve.rs`·`src/main.rs` 가 동일한 빌드
- AFTER: 이 브랜치 빌드

| | offset=0 | offset=3 | offset=6 | 도달 |
|---|---|---|---|---|
| BEFORE | 앞 3건 | **같은 앞 3건** | **같은 앞 3건** | 3 / 276 |
| AFTER | 1~3번째 | 4~6번째 | 7~9번째 | 276 / 276 |

BEFORE 는 `offset` 을 모르므로 값을 바꿔도 응답이 같고 `nextOffset` 도 없다 — 루프가 1홉에서
끝나 나머지 273건은 이 도구로 도달할 수단이 없다. AFTER 는 창이 매 홉 전진하고,
`nextOffset` 이 사라질 때까지 따라가면 전수에 닿는다.

재현은 `tests/mcp_result_cursor_contract.rs` 가 그대로 한다(창 1·2·3·7 에서 이어 붙인 결과가
전수와 정확히 일치하는지까지 검사).
