# 17 — fill 은 stdin 목록이 아니다

`batch fill` 은 다른 여덟 축과 입력 축이 다르다.

| | 읽기·convert 축 | fill |
| --- | --- | --- |
| 입력 | stdin 경로 목록 | `--form` + `--data` |
| 단위 | 파일 | 데이터 행 |
| 산출 | convert 만 `--out-dir` | 항상 `--out-dir` |
| 레코드 키 | source = 파일 | source = 서식, + `row` |

```bash
# 잘못 — 서식 경로 목록을 넣는 축이 아니다
find forms/ -name '*.hwp' | rhwp batch fill --json

# 맞음
rhwp batch fill --form 신청서.hwp --data 명단.csv --out-dir out/filled --json
```

`tests/batch_fill_contract.rs` 명문: 데이터는 stdin 이 아니라 `--data` 파일.
stdin 에 경로를 흘려도 fill 은 읽지 않는다 (BrokenPipe 가 정상).

## 필수 플래그

- `--form` 서식 `.hwp` / `.hwpx`
- `--data` `.jsonl` (한 줄 객체) 또는 `.csv` (첫 줄 헤더 = 누름틀 이름)
- `--out-dir` — `--dry-run` 에도 필수

헤더만 있고 데이터 0행이면 exit 2 (`empty_header_only.csv` 픽스처).
`--data` 는 UTF-8. CP949 는 `stream did not contain valid UTF-8` 로 exit 1.

## 이 스킬이 하지 않는 것

- `이름[N]` 순번 규칙의 정본 — `rhwp-form-fill`
- sanitize / insert-image
- 새 merge 명령

폴더에 서식이 수백이면 `batch fields` 로 고르고, 채울 서식 하나에
명단을 붓는다 (정지 B12).

## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `17_fill_not_stdin.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
