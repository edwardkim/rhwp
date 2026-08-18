# 09. goal=convert-hwp

```
rhwp convert <doc> out/<stem>.hwp --verify
```

편집 가능한 HWP 로 옮긴다. 게이트는 `export-hwpx` 와 같은 자기검증.

`needs:convert` — capabilities 가 `convert` 를 광고해야 한다.

## 게이트 (C19)

exit 0 + 파일 실존. 실패면 `convert --verify exit N`.

## 산출

- `out/<stem>.hwp`
- `summary`: `HWP 변환 + verify 통과`

## 표기

goal 이름은 `convert-hwp` 이고 명령 이름은 `convert` 다.
요청에 `"goal": "convert"` 만 오면 표에 없으므로 needs-agent (C06).
별칭을 코드에 넣지 않는다. 별칭이 필요하면 표에 행을 더하는 PR 이다.
