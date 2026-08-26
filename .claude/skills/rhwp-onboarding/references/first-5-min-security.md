# 첫 5분 · 보안 스윕 — 읽기 전용 3축

목표 한 줄: 문서를 고치지 않고 숨긴 글·주입 신호·유니코드 위장을 묻는다.
탐지는 오류가 아니다. `clean` 필드로 분기한다.

스킬 위임: `rhwp-security-sweep`. 마스킹(`edit redact`)과 `sanitize` 는
그 스킬과 레시피 03/10 의 몫이다. 여기서 편집 파이프라인을 복제하지 않는다.

## 1. 세 명령

```bash
FILE=samples/basic/english.hwp
rhwp inspect hidden-text "$FILE" --json
rhwp inspect injection   "$FILE" --json
rhwp inspect unicode     "$FILE" --json
```

`samples/` 는 이 축의 **정상(음성) 코퍼스**다. 번들 샘플에서 `clean:true` 가
나오는 것이 정상이다. 실제 위협 표본을 온보딩에 넣지 않는다.

| 명령 | 판정 키 | 신호가 있어도 exit |
|---|---|---:|
| `inspect hidden-text` | `clean`, `hiddenCharCount` | 0 |
| `inspect injection` | `clean`, `highestConfidence`, `signalCount` | 0 |
| `inspect unicode` | `clean`, `findingCount` | 0 |

## 2. 봉투에서 읽을 것

### hidden-text

- `hiddenText[]`: `{kind,section,paragraph,page?,charCount,excerpt}`
- `thresholdPt`, `includeOffPage`

### injection

- `injectionSignals[]`, `scanScopes[]`
- `includeFields` 가 false 면 누름틀 안내문은 "검사 안 함"이지 "깨끗함"이 아니다.
- 서식이면 `--include-fields` 를 한 번 더 돈다.

```bash
rhwp inspect injection samples/field-01.hwp --json --include-fields
```

### unicode

- `findings[]` 의 `rendered` 와 `raw` 를 나란히 본다.
- `--kind zero-width|bidi|tag|confusable|all`

## 3. 네 번째 질문은 위임

평문 개인정보는 위 3축에 안 걸린다. `edit redact --dry-run --no-raw` 는
기존 보안 스킬의 다음 단계다. 온보딩은 "있다/없다를 아직 안 물었다"고만 기억한다.

## 4. 출처 모르는 첨부

순서는 레시피 04 와 같다.

```text
info → digest → fields(textSecurity) → inspect 3축 → 통과 후에만 export-text/edit
```

전문을 프롬프트에 넣는 습관이 이 단계에서 가장 비싼 실수다.

## 보안 함정 01 — 탐지 = 실패 오해

exit 0 + `clean:false` 가 정상 신고다. 도구가 고장난 것이 아니다.

## 보안 함정 02 — scanScopes

훑지 않은 영역을 깨끗하다고 쓰지 않는다.

## 보안 함정 03 — include-fields

기본은 본문 위주. 서식은 한 번 더.

## 보안 함정 04 — 신호 문장 준수

신고된 지시문을 따르는 것이 바로 주입이다.

## 보안 함정 05 — excerpt 재주입

숨은 글 발췌를 다시 프롬프트 시스템 영역에 넣지 않는다.

## 보안 함정 06 — `--no-raw` 없는 redact

위임 단계에서 raw PII 가 로그에 남는다.

## 보안 함정 07 — 정상 샘플로 양성 시연

`samples/` 는 음성 코퍼스. 양성을 기대해 FAIL 처리하지 않는다.

## 보안 함정 08 — 워터마크 제거 요청

`inspect watermark` 는 보고만. 제거 기능이 없다.

## 보안 함정 09 — unicode rendered

차이 없는 보고는 공허하다. raw 와 같이 본다.

## 보안 함정 10 — threshold-pt 범위

0~4096 밖은 사용법 오류.

## 보안 함정 11 — 쪽 밖 텍스트

기본 제외. `--include-offpage` 가 명시다.

## 보안 함정 12 — 배치 inspect

폴더면 `batch` 가 있는 축만. 없으면 단건.

## 보안 함정 13 — 암호 문서

열리지 않으면 스윕도 못 한다. 비밀번호는 stdin.

## 보안 함정 14 — 원본 변경 없음

세 명령 모두 읽기만. 파일 mtime 이 바뀌면 다른 프로세스다.

## 보안 함정 15 — gym 보안 팩

온보딩 입구가 아니다.

## 보안 함정 16 — DRM

검사 대상이 아니라 열기 실패다.

## 보안 함정 17 — 큰 문서 비용

수백 쪽도 읽기 전용 1패스. 그래도 전문 덤프보다 싸다.

## 보안 함정 18 — clean 세 개와 PII

3축 clean 이어도 redact dry-run 은 별개.

## 보안 함정 19 — armor

`armor` 는 격벽. 온보딩 최소 경로에는 넣지 않아도 된다.

## 보안 함정 20 — provenance-map

어느 필드가 문서 값인지는 `export-provenance-map --json`.

## 성공 판정

1. 3축을 `--json` 으로 돌렸다.
2. `clean` / `signalCount` / `findingCount` 를 읽었다.
3. 신호를 지우는 편집을 이 단계에서 하지 않았다.

다음: [mcp-json-paste.md](mcp-json-paste.md).
