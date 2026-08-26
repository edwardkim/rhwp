# 09 — form-fill

켤 때: `field_count > 0`. 우선순위 80. skill `rhwp-form-fill`.
confidence 는 항상 high.

```
rhwp fields <file> --json
```

## why

`누름틀(입력 필드) N개 — 값 채우기·명단 메일머지 대상`

필드 이름·값은 explore 봉투에 없다. 이름은 `fields` 가 준다.

## 다음

1. `fields --json` 으로 name/guide/memo 를 읽는다
2. 채움은 `edit fill-fields` / `batch fill` — form-fill 스킬
3. `textSecurity` 가 clean 이 아니면 그 스킬이 security-sweep 으로 인계

explore 가 form-fill 과 security-sweep 를 같이 켜면 보안이 위다.
채우기 전에 스윕한다.

이 장이 fill-fields 를 재구현하지 않는다. 이웃 스킬 본문을 고치지 않는다.
