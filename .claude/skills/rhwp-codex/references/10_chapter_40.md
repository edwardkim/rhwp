# 40장 항해 — 변환·렌더 — 형식을 넘나든다

파일: `mydocs/manual/agent_codex/40_변환과_렌더.md`
성격: **generated**

이 장은 `generated: tools/gen_agent_codex.py` frontmatter 를 가진다.
수기 수정 금지. 표본을 고치고 싶으면 생성기의 LIVE 계획을 고친다.

요청 갈래: **변환** → 이 장.

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
| `convert` | 실측 | export | convert <입력.hwp\|입력.hwpx> <출력.hwp> [--verify] [--verify-pages] |
| `export-hwpx` | 계약만 | export | export-hwpx <입력.hwp\|입력.hwpx> [출력.hwpx] [--verify] [--verify-pages] |
| `export-hml` | 계약만 | export | export-hml <입력.hml> -o <출력.hml> |
| `export-markdown` | 계약만 | export | export-markdown <파일.hwp> [옵션] |
| `export-doclang` | 계약만 | export | export-doclang <파일.hwp\|파일.hwpx> [-o <출력.xml>] [--assets-dir <디렉터리>] [--json] |
| `export-pdf` | 계약만 | export | export-pdf <파일.hwp\|파일.hwpx\|파일.hml> [옵션] |
| `export-svg` | ? | export | export-svg <파일.hwp\|파일.hwpx\|파일.hml> [옵션] |
| `thumbnail` | 계약만 | export | thumbnail <파일.hwp> [옵션] |
| `render-diff` | 계약만 | diagnostic | render-diff <파일> [--via hwpx\|hwp] [-p <페이지>] [--max-disp <px>] [--json] |
| `build-from-ingest` | 계약만 | export | build-from-ingest <ingest.json> [--media-dir <dir>] -o <out.hwpx> |
| `scaffold` | 계약만 | export | scaffold <spec.json> [--format hwpx] -o <out.hwpx> [--json] |
| `export-png-gpu` | 계약만 | export | export-png-gpu <파일.hwp\|파일.hwpx> [옵션]   (gpu feature 필요) |
| `gpu-info` | 계약만 | export | gpu-info                        (gpu feature 필요) |

실측 1 · 계약만 11 · 합 13.

## 하지 말 것

- 이 마크다운의 JSON 을 손으로 고치기
- 계약만 명령에 가짜 봉투를 지어 넣기
- 전문 스킬이 있는 일을 여기서 재구현하기
- 새 플래그·새 하위명령을 문서에만 추가하기

## 관련 스킬

깊이 있는 실행은 `rhwp-visual-regression` 가 정본이다. 이 스킬은 장 번호까지만 안내한다.
