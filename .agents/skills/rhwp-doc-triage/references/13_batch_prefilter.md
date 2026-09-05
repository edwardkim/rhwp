# 13 — 폴더 선별 (batch)

문서가 여러 개면 단건 사다리를 파일마다 돌리지 않는다.

```bash
find docs/ -name '*.hwp' | rhwp batch info --json > meta.ndjson
find docs/ -name '*.hwp' | rhwp batch search --query "위임전결" --json \
  | jq -c 'select(.matchCount > 0) | {source, pages:[.matches[].page]}'
```

- stdout 은 NDJSON 만. 요약은 stderr.
- 하나라도 실패하면 최종 exit 1 이지만 나머지 행은 유효하다.
- `batch search` 는 `--query` 필수. 파일당 1000건 상한.
- 선별된 파일만 단건 사다리로 내려간다.

## 운용 F01~

1. 폴더에 파일이 1개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F01).
2. 폴더에 파일이 2개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F02).
3. 폴더에 파일이 3개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F03).
4. 폴더에 파일이 4개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F04).
5. 폴더에 파일이 5개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F05).
6. 폴더에 파일이 6개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F06).
7. 폴더에 파일이 7개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F07).
8. 폴더에 파일이 8개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F08).
9. 폴더에 파일이 9개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F09).
10. 폴더에 파일이 10개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F10).
11. 폴더에 파일이 11개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F11).
12. 폴더에 파일이 12개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F12).
13. 폴더에 파일이 13개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F13).
14. 폴더에 파일이 14개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F14).
15. 폴더에 파일이 15개 이상이거나 사용자가 '전부/일괄'을 말하면 `find` + `rhwp batch info --json` 으로 메타만 먼저 받는다. 매치가 필요한 경우에만 `batch search --query`. 실패 레코드는 `error`+`exitClass` 로 격리되고 스트림은 계속된다 (F15).
