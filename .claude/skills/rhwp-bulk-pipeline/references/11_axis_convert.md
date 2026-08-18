# 11 — batch convert

## 한 줄

HWP5 일괄 변환. 이름 예약. CLI 전용(MCP 제외).

## 호출

```bash
rhwp batch convert --json [옵션]
```

- 입력: stdin 한 줄 = 경로
- 단건 동형: convert --json
- 플래그: `--json`, `--threads`, `--out-dir`, `--verify`, `--verify-pages`
- 성공 키: `schemaVersion`, `source`, `format`, `output`, `bytes`


## 언제

편집 가능한 HWP5 를 한 폴더에 모을 때. CLI 전용 쓰기 축.
MCP `hwp_batch` 에는 없다 (batch.mcp.excluded).

```bash
printf '%s\n' "samples/2025 행정업무운영 편람(최종).hwpx" \
  | rhwp batch convert --out-dir out/bulk --json
```

레시피 9 실측: 387쪽, 428ms, `bytes: 9083392`,
`output: out/bulk\\2025 행정업무운영 편람(최종).hwp`.

## 필수

`--out-dir <폴더>`. 이름은 `<out-dir>/<입력이름>.hwp`.
`-` 로 시작하는 폴더는 `./-결과`.

## 이름 예약

쓰기 **전에** 모든 산출 이름을 예약한다. 같은 stem, 대소문자만 다른
이름, 다른 폴더의 같은 파일명은 충돌 → **exit 2, 한 파일도 안 씀**.
상세는 `16_convert_name_reservation.md`.

## 검증

- `--verify` — IR 자기검증. 차이만 있으면 집계 3
- `--verify-pages` — 페이지 수. 불일치만 있으면 집계 4
- error 행이 하나라도 있으면 둘 다 이기고 집계 1

산출 파일은 verify 실패해도 남는다. "실패면 없음"으로 지우지 말 것.

## 원본

원본은 읽기만. `--in-place` 가 없다. `out/` 아래로 분리.


## 권위

- `mydocs/manual/cli_commands.md` §batch
- `mydocs/manual/cli_json_pipeline_guide.md`
- `mydocs/manual/recipes/09_bulk_extract_convert.md`
- `mydocs/manual/recipes/05_mail_merge_batch_fill.md`
- `rhwp capabilities` 의 batch 항목
- 픽스처: `fixtures/` · 이 장: `11_axis_convert.md`
- 이슈 #5311. gym 아님. 새 CLI 아님.
