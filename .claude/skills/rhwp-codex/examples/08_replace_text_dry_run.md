# 08 — 치환 dry-run

갈래: **편집**. 장: `30_편집과_계획.md`. gym 아님. 새 CLI 아님.

권위는 생성 장이 아니라, 생성 장을 **읽는 순서**다. 표본 JSON 을 손대지 않는다.

## 요청

`규제 를 코덱스검증 으로. 원본은 건드리지 마.`

## 명령

```bash
rhwp edit replace-text samples/basic/issue2007_nested_cell_pagination_42065.hwp --find 규제 --replace 코덱스검증 --dry-run --json
```

`dryRun: true` 이고 산출 파일이 없어야 한다. 그 다음 `-o` 로 실행.
C4. `--in-place` 를 기본값으로 쓰지 않는다.
