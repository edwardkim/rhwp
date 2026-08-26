# 19 — `toolVersion` 선대조

같은 계획이어도 rhwp 버전이 다르면 산출 바이트가 갈릴 수 있다.
`reproduced: false` 를 상대 부정으로 단정하기 전에 영수증의 `toolVersion` 을
지금 바이너리와 대조한다.

```bash
rhwp replay --plan-json '<계획>' --json   # toolVersion 확인
# 불일치 → 같은 버전으로 재현하거나, 불일치를 보고서에 적고 멈춘다
```

이 스킬은 버전을 맞추는 새 플래그를 만들지 않는다. 기존 영수증 필드다.
