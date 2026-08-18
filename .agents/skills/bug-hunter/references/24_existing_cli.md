# 24 — 기존 CLI 만

이 스킬은 명령을 발명하지 않는다. 허용 목록은
`fixtures/skill_index.json` 의 `allowedCommands` 와
`fixtures/command_ladder.json`.

## 허용 (헌팅에서 부르는 것)

- info, fields, export-tables
- export-svg, export-png, export-pdf, export-text
- export-hwpx, export-render-tree
- edit set-cell, edit fill-fields, edit replace-text
- ir-diff, render-diff
- dump, dump-pages, capabilities, search, convert
- thumbnail, inspect

도구: `venv/bin/python tools/fidelity_compare/fidelity_compare.py`
(CLI 하위명령 아님).

위 목록에 없는 **기존** 명령(batch, digest, …)을 여정이 필요하면
쓸 수 있다. 다만 새 이름을 만들지 않는다.

## 금지 (발명)

`fixtures/skill_index.json` 의 `inventedCommandsForbidden`:

- hunt (전용 하위명령)
- oracle 점검 전용 하위명령
- fidelity 대조를 rhwp 하위로 흡수
- 정답지 전용 하위명령
- gym 헌팅 하위명령

문서에 위 발명 호출이 나타나면 계약 시험이 실패한다.

## DocumentCore / 엔진

경로를 인용(`file.rs:LINE`)하는 것은 허용이다. 그 파일을 고치는
것은 F12. 이 PR 의 범위가 아니다.

## gym/

`gym/` 트리에 여정·과제·채점기를 추가하지 않는다. 이 스킬 디렉터리
아래에도 `gym` 경로를 두지 않는다.
