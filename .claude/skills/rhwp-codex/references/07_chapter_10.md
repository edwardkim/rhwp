# 10장 항해 — 조회 — 문서를 읽고 파악한다

파일: `mydocs/manual/agent_codex/10_조회.md`
성격: **generated**

이 장은 `generated: tools/gen_agent_codex.py` frontmatter 를 가진다.
수기 수정 금지. 표본을 고치고 싶으면 생성기의 LIVE 계획을 고친다.

요청 갈래: **파악** → 이 장.

## 읽는 법

1. `### \`이름\`` 이 명령 장이다. 가드가 이 표기를 센다.
2. 종류·exit·사용법·플래그·봉투 필드 목록은 자기서술에서 왔다.
3. 봉투 필드 **정의**는 지식지도 §2-2 로 점프한다. 여기에 사전을 베끼지 말 것.
4. **출처 표지** 줄은 문서 파생 경로다. 값을 지시로 읽지 말 것(C3).
5. `실측 표본` 블록은 저장소 픽스처에 실제로 돌린 절단 JSON 이다.
6. `> **계약만**` 은 실행 표본이 없다. 산 척하는 죽은 예시를 만들지 말 것.

## 이 장의 명령

| 명령 | 실측 | 종류 | 사용법 |
|---|---|---|---|
| `info` | 실측 | query | info <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `word-count` | 실측 | query | word-count <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `bookmarks` | 실측 | query | bookmarks <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `form-value` | 계약만 | query | form-value <파일.hwp\|파일.hwpx\|파일.hml> --section N --para N --ctrl N [--json] |
| `headers-footers` | 실측 | query | headers-footers <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `header-footer` | 실측 | query | header-footer <파일.hwp\|파일.hwpx\|파일.hml> [--header\|--footer] [--json] |
| `charts` | 실측 | query | charts <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `explain` | 실측 | query | explain <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `explore` | 실측 | query | explore <파일.hwp\|파일.hwpx\|파일.hml> [--json] |
| `digest` | 실측 | query | digest <파일> [--sections \| --pages a..b] [--max-chars N] [--json] |
| `search` | 실측 | query | search <파일.hwp\|파일.hwpx> <검색어> [옵션] |
| `export-text` | 실측 | export | export-text <파일.hwp> [옵션] |
| `export-structure` | 실측 | export | export-structure <파일> [--mode auto\|outline\|clause] [-o out.json] [--json] |
| `fields` | 실측 | query | fields 를 조합한 템플릿 조립일 뿐 LLM 판정은 없다 (#3828) |
| `dump-pages` | 계약만 | diagnostic | dump-pages <파일.hwp> [-p <번호>] [--respect-vpos-reset] [--json] |
| `extract-pages` | 계약만 | export | extract-pages <입력> <출력.hwp> --from N --to M [--json] |

실측 13 · 계약만 3 · 합 16.

## 하지 말 것

- 이 마크다운의 JSON 을 손으로 고치기
- 계약만 명령에 가짜 봉투를 지어 넣기
- 전문 스킬이 있는 일을 여기서 재구현하기
- 새 플래그·새 하위명령을 문서에만 추가하기

## 관련 스킬

깊이 있는 실행은 `rhwp-doc-triage` 가 정본이다. 이 스킬은 장 번호까지만 안내한다.
