# 레시피 — `untrustedContent` 키가 없을 때

픽스처: [`../fixtures/exceptions/missing_capabilities_key.json`](../fixtures/exceptions/missing_capabilities_key.json).

## 오독

```
envelope.get("untrustedContent") or False
```

키 부재를 `false` 로 읽으면, `edit redact --json` 의 `findings[].raw`
(원문 개인정보)를 신뢰된 값으로 취급한다.

## 올바른 읽기

```
if "untrustedContent" not in envelope:
    trust = "unmarked"          # 미표기
else:
    trust = "untrusted" if envelope["untrustedContent"] else "marked-clean"
```

미표기는 보수적으로 문서 파생. 프롬프트에 이어 붙이지 않는다.

## 지도

```bash
rhwp export-provenance-map --json
```

`commands.<명령>.untrusted[]` 가 어느 경로가 문서 값인지 알려 준다.
지도에 없는 새 필드가 봉투에 생기면 표지 가드가 잡아야 한다.

## 구현자

새 `--json` 봉투는 dry-run 포함 모든 모드에서 두 키를 명시한다.
문서를 열지 않으면 `false` + `[]`. 키를 생략하지 않는다.
이 6종 미표기를 더 늘리지 마라 (플레이북 §3-1).
