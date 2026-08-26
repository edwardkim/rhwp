# 15 — batch 에 전역 `--password` 금지

capabilities 명문: batch 는 `--password` · `--password-stdin` ·
`--output-password` · `--output-password-stdin` 을 지원하지 않는다.
함께 주면 **입력을 소비하지 않고 exit 2**.

## 왜

배치 stdin 은 경로 목록 전용이다. 비밀번호를 같은 프로세스에 실으면
모든 파일에 같은 암호를 쓰게 되고, 로그·코어덤프·에이전트 대화에
자격 증명이 남는다. 암호화 산출 형식 계약도 아직 없다.

## 에이전트가 할 일

1. `batch info` 로 실패 행을 본다.
2. 암호 신호가 있는 경로만 `암호.txt` 로 뺀다.
3. 평문 목록은 그대로 batch.
4. 암호 문서는 단건:

```bash
rhwp info 보호.hwp --password-stdin < password.txt --json
rhwp export-text 보호.hwp --password-stdin < password.txt --json
```

단건 암호 규약은 cli_commands 상단(위치 자유, 한 번만, stdin BOM 허용).
이 스킬은 그 규약을 재작성하지 않는다.

## 실수 패턴

```bash
# 금지 — 평문 271건이 전부 exit 2 로 죽는다
cat 목록.txt | rhwp --password secret batch export-text --json
```

전사는 `examples/transcripts/T07.ndjson` (빈 stdout) +
`fixtures/password_reject.json`.

## 암호화 산출

`batch convert` / `batch fill` 에 출력 암호를 다는 플래그도 없다.
필요하면 단건 `convert` 계약을 문서화한 뒤의 별도 이슈다. 여기서 발명하지 않는다.

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `15_no_global_password.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
