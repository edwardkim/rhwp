# 30장 항해 — 편집·계획 — 원본 무훼손 변경

파일: `mydocs/manual/agent_codex/30_편집과_계획.md`
성격: **generated**

이 장은 `generated: tools/gen_agent_codex.py` frontmatter 를 가진다.
수기 수정 금지. 표본을 고치고 싶으면 생성기의 LIVE 계획을 고친다.

요청 갈래: **편집** → 이 장.

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
| `edit` | 계약만 | edit | edit fill-fields <파일.hwp\|파일.hwpx> --data <JSON\|@파일> [-o <출력>] [옵션] |
| `edit replace-text` | 실측 | edit | edit replace-text <파일.hwp\|파일.hwpx> --find <문자열> --replace <문자열> [옵션] |
| `edit set-cell` | 실측 | edit | edit set-cell <파일> --table <번호> --row <행> --col <열> --text <문자열> [옵션] |
| `edit fill-fields` | 실측 | edit | edit fill-fields <파일.hwp\|파일.hwpx> --data <JSON\|@파일> [-o <출력>] [옵션] |
| `edit insert-image` | 계약만 | edit | edit insert-image <파일> --image <그림> [옵션] |
| `edit redact` | 실측 | edit | edit redact <파일.hwp\|파일.hwpx> [--kind …] [--dry-run] [--no-raw] [-o <출력>\|--in-place] |
| `edit sanitize` | 계약만 | edit | edit sanitize <파일.hwp\|파일.hwpx> [--keep-preview] [-o <출력>] [--json] |
| `run` | 실측 | edit | run <계획.json> [--json]              선언적 편집 계획 실행 (#3703) |

실측 5 · 계약만 3 · 합 8.

## 하지 말 것

- 이 마크다운의 JSON 을 손으로 고치기
- 계약만 명령에 가짜 봉투를 지어 넣기
- 전문 스킬이 있는 일을 여기서 재구현하기
- 새 플래그·새 하위명령을 문서에만 추가하기

## 관련 스킬

깊이 있는 실행은 `rhwp-safe-edit / rhwp-form-fill` 가 정본이다. 이 스킬은 장 번호까지만 안내한다.
