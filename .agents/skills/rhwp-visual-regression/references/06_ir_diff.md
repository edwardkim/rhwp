# 06 — ir-diff --json

레이아웃(px)이 아니라 IR 구조(텍스트, 문단 모양, 표 필드, 컨트롤)를
비교한다. 변환 파이프라인의 내용 보존 게이트다.

```bash
rhwp ir-diff A.hwpx B.hwp --json
```

봉투 한 줄:

```
{"schemaVersion":"1.0","a","b","identical","diffCount","categories":{…}}
```

불변식: `identical` ⇔ `diffCount == 0` ⇔ `categories` 가 비어 있음.

## 종료 코드

| 상황 | --json | 텍스트 |
| --- | --- | --- |
| 동일 | 0 | 0 |
| 차이 | **3** | **0** (기존 소비자 보호) |
| 읽기·파싱 실패 | 1, stdout 0바이트 | 1 |
| 사용법 | 2 | 2 |

게이트는 반드시 `--json` 이다.

```bash
rhwp ir-diff 원본.hwp 변환본.hwpx --json || 격리처리
```

exit 3 은 실패가 아니라 **차이 검출 데이터**다. `categories` 로
어느 축(text, char_count, controls, …)인지 읽는다.

`--summary` / `--max-lines` 와 `--json` 을 같이 주면 JSON 이 이긴다.
stdout 순수성.

알 수 없는 옵션은 현재 조용히 무시된다(#3178). 게이트 스크립트는
플래그 철자를 정확히 쓴다.
