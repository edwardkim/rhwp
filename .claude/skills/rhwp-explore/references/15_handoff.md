# 15 — 이웃 스킬로 인계

이 스킬은 라우터다. 아래 스킬의 SKILL.md 를 이 PR 이 재작성하지 않는다.

| 언제 | 스킬 | 기존 명령 |
| --- | --- | --- |
| table-extract 가 메뉴에 있다 | rhwp-table-exchange | export-tables / table-to-csv |
| form-fill 이 메뉴에 있다 | rhwp-form-fill | fields → fill-fields / batch fill |
| security-sweep 가 메뉴에 있다 | rhwp-security-sweep | inspect injection|hidden-text|unicode |
| structure / long-doc / note / triage | rhwp-doc-triage | export-structure / digest / explain |
| 메뉴를 본 뒤 여러 번 편집 | rhwp-safe-edit | run 계획서 3층 |
| 파일 수백 개 | rhwp-bulk-pipeline | batch info / export-text (explore 아님) |

## 재작성 금지

rhwp-onboarding, rhwp-mcp-session, rhwp-safe-edit, rhwp-provenance,
rhwp-form-fill, rhwp-security-sweep, rhwp-doc-triage,
rhwp-table-exchange 본문은 범위 밖이다. 여기서는 이름과 첫 명령만
가리킨다.

## 폴더

`explore` 는 파일 하나다. 두 파일을 한 줄에 주면 exit 2.
수백 건은 `rhwp-bulk-pipeline` 의 `batch info` / `batch export-text`.
폴더용 explore 명령을 만들지 않는다.
