---
kind: guide
status: active
canonical: mydocs/manual/recipes/03_redact_before_sharing.md
last_verified: 2026-08-03
---

# 레시피 3 — 배포 전 개인정보 마스킹

**목표 한 줄**: 문서를 외부로 보내기 전에, 본문에 남은 개인정보(주민등록번호·카드번호·
전화번호·이메일)를 **자릿수를 보존한 채** 마스킹하고, "정말 지워졌는가"를 눈이 아니라
기계 판정으로 확인한다.

`edit redact` 는 **본문**의 개인정보를 지우고, [레시피 1](01_fill_form_and_submit.md)
마지막 단계의 `edit sanitize` 는 **문서 속성**(작성자·제목·미리보기)을 지운다. 배포
전 정리는 이 둘이 짝이다 — 본문만 지우면 미리보기와 작성자 이름이 남고, 속성만 지우면
본문의 전화번호가 남는다.

마스킹은 **되돌릴 수 없는 쓰기**다. 그래서 이 레시피의 순서 자체가 안전장치다:

```
edit redact --dry-run        (무엇이 지워질지 먼저 — 파일 무변경)
  → edit redact -o 공개본 --verify [--no-raw]   (적용 + 저장본 재파싱 검증)
  → search                   (원문이 정말 사라졌는지 교차 확인)
  → edit sanitize -o 배포본  (메타데이터·미리보기 제거)
  → edit redact --dry-run    (재검사 — 탐지 0건이면 게이트 통과)
```

모든 명령·출력은 이 저장소의 `samples/field-01.hwp` 로 실제 실행해서 얻었다. 지어낸
값은 없다. **개인정보가 든 샘플은 저장소에 두지 않는다** — 회귀 테스트
(`tests/redact_sanitize_contract.rs`)와 같은 방식으로, 형태만 개인정보인 **가짜 값**을
누름틀에 심어 표본을 만든다.

## 1단계 — 가짜 개인정보 표본을 만든다

`samples/field-01.hwp` 는 회사명·작성자·부서명·전화번호·이메일 누름틀이 있는 서식이다
(`fields --json` 으로 확인 가능, `fieldCount: 11`). 여기에 [레시피 1](01_fill_form_and_submit.md)의
`edit fill-fields` 로 가짜 값을 심는다:

```bash
rhwp edit fill-fields samples/field-01.hwp --data '{"작성자":"홍길동 900101-1234568","전화번호":"010-1234-5678","이메일":"hong@example.com","회사명":"카드 4111-1111-1111-1111 / 미끼 900101-1234567 / 미끼 1234-5678-9012-3456"}' -o output/pii-demo.hwp --json
```

값 여섯 개의 성격이 서로 다르다 — 이게 이 표본의 요점이다:

- `900101-1234568` — 검증 숫자(mod 11)를 **통과하는 가공** 주민등록번호. 실재 인물과
  무관하다.
- `4111-1111-1111-1111` — Luhn 을 통과하는 공개 시험용 카드번호.
- `010-1234-5678` / `hong@example.com` — 형태를 갖춘 전화·이메일.
- **미끼 2개**: `900101-1234567`(검증 숫자 틀림) · `1234-5678-9012-3456`(Luhn 실패).
  형태는 같지만 진짜 개인정보일 수 없는 문자열이다 — 이게 마스킹되면 오탐이다.

## 2단계 — `--dry-run` 으로 무엇이 지워질지 먼저 본다

```bash
rhwp edit redact output/pii-demo.hwp --dry-run
```

실측 출력:

```
마스킹 예정: output/pii-demo.hwp — 탐지 4건 (원문 4개). 실제 적용은 -o 또는 --in-place.
  [card] 4111-1111-1111-1111 → ****-****-****-**** (구역 0, 문단 7, 쪽 1)
  [ssn] 900101-1234568 → ******-******* (구역 0, 문단 8, 쪽 1)
  [phone] 010-1234-5678 → ***-****-**** (구역 0, 문단 10, 쪽 1)
  [email] hong@example.com → ****@*******.*** (구역 0, 문단 11, 쪽 1)
```

읽어야 할 것 셋:

- **탐지 4건 — 미끼 2개는 없다.** 탐지기는 형태가 맞아도 검증(주민등록번호 mod 11,
  카드 Luhn)을 통과하지 못하면 잡지 않는다. 마스킹은 되돌릴 수 없으므로 이 도구는
  **오탐 0 을 우선**한다 — 재현율이 아니라 정밀도가 설계 기준이다.
- **자릿수·구조 문자 보존.** `010-1234-5678 → ***-****-****` — 하이픈·`@`·`.` 은 남고
  영숫자만 가려진다. 글자 수가 같아서 서식의 칸 너비·줄바꿈이 흔들리지 않는다
  (다른 마스킹 문자는 `--mask '#'` 처럼 지정, 영숫자는 거부된다).
- **주소가 같이 나온다** (구역·문단·쪽) — 마스킹 전에 그 자리를 열어 사람이 확인할 수
  있다.

기계 계약이 필요하면 `--json` 을 쓴다:

```bash
rhwp edit redact output/pii-demo.hwp --dry-run --json --no-raw
```

실측 출력:

```json
{"changedPages":null,"dryRun":true,"findingCount":4,"findings":[{"charOffset":10,"kind":"card","masked":"****-****-****-****","page":0,"paragraph":7,"section":0},{"charOffset":11,"kind":"ssn","masked":"******-*******","page":0,"paragraph":8,"section":0},{"charOffset":7,"kind":"phone","masked":"***-****-****","page":0,"paragraph":10,"section":0},{"charOffset":9,"kind":"email","masked":"****@*******.***","page":0,"paragraph":11,"section":0}],"inPlace":false,"kinds":["ssn","card","phone","email"],"mask":"*","noRaw":true,"redactedCount":0,"schemaVersion":"1.0","source":"output/pii-demo.hwp"}
```

⚠️ **`--no-raw` 를 뺀 기본 봉투에는 `findings[].raw` 로 개인정보 원문이 그대로
실린다.** 무엇이 지워질지 사람이 확인하는 데는 원문이 필요하지만, 그 봉투를 로그·
이슈·채팅에 통째로 붙이는 순간 마스킹하려던 값이 다른 곳에 남는다. 봉투가 파이프라인·
로그로 흘러가는 자동화라면 위처럼 `--no-raw`(봉투에서 `raw` 제거, `noRaw: true` 로
명시)를 기본으로 삼는 편이 안전하다.

## 3단계 — 적용한다 (산출 경로는 도구가 강제한다)

산출 경로 없이 실행하면 어떻게 되는지부터 — 일부러 그렇게 해 본다:

```bash
rhwp edit redact output/pii-demo.hwp
```

실측 출력 (exit 2, stdout 0바이트):

```
오류: 마스킹은 되돌릴 수 없습니다. 산출 경로를 -o <출력> 으로 지정하거나, 원본을 덮어쓸 의도라면 --in-place 를 명시하세요 (먼저 --dry-run 으로 무엇이 지워질지 확인하기를 권합니다).
```

"실수로 원본을 잃는" 경로가 아예 막혀 있다 — 기본 산출 이름조차 만들지 않는다.
이제 `-o` 와 `--verify` 를 붙여 실제로 적용한다:

```bash
rhwp edit redact output/pii-demo.hwp -o output/pii-demo_공개.hwp --verify --json --no-raw
```

실측 출력:

```json
{"changedPages":[0],"dryRun":false,"findingCount":4,"findings":[{"charOffset":10,"kind":"card","masked":"****-****-****-****","page":0,"paragraph":7,"section":0},{"charOffset":11,"kind":"ssn","masked":"******-*******","page":0,"paragraph":8,"section":0},{"charOffset":7,"kind":"phone","masked":"***-****-****","page":0,"paragraph":10,"section":0},{"charOffset":9,"kind":"email","masked":"****@*******.***","page":0,"paragraph":11,"section":0}],"inPlace":false,"kinds":["ssn","card","phone","email"],"mask":"*","noRaw":true,"output":"output/pii-demo_공개.hwp","outputFormat":"hwp5","redactedCount":4,"schemaVersion":"1.0","source":"output/pii-demo.hwp","verify":{"diffCount":0,"identical":true}}
```

- `redactedCount: 4` — 치환이 실제로 일어난 횟수. `findingCount` 와 다르면 같은 값이
  여러 곳에 있었다는 뜻이다(치환은 값 단위 전량).
- `verify.identical: true` — 저장본을 다시 파싱해 메모리 상태와 대조한 결과다. 이게
  `false` 면 exit 3 으로 끝나므로 스크립트가 바로 알 수 있다.
- `changedPages: [0]` — 몇 쪽이 바뀌었는지. 렌더 확인([레시피 6](06_visual_regression_before_after.md))을
  이 쪽들로 좁힐 수 있다.
- 탐지가 0건이면 **출력 파일 자체를 만들지 않는다** — "마스킹했다"고 믿었는데 빈
  산출물만 도는 사고를 막는다.

## 4단계 — `search` 로 교차 확인한다

마스킹을 수행한 도구의 보고만 믿지 말고, 읽기 전용 명령으로 원문이 사라졌는지 다시 본다:

```bash
rhwp search output/pii-demo_공개.hwp "010-1234-5678" --json
```

실측 출력:

```json
{"caseSensitive":true,"matchCount":0,"matches":[],"omittedCount":0,"query":"010-1234-5678","schemaVersion":"1.0","source":"output/pii-demo_공개.hwp","totalMatchCount":0,"truncated":false,"untrustedContent":false,"untrustedFields":[]}
```

`matchCount: 0` — 원문 전화번호는 본문에 없다. 반대로 마스킹 자리를 찾아보면:

```bash
rhwp search output/pii-demo_공개.hwp "***-****-****" --json --limit 1
```

실측 출력:

```json
{"caseSensitive":true,"matchCount":1,"matches":[{"charOffset":11,"context":"회사명\t\t: 카드 ****-****-****-**** / 미끼 900101-1234567 / 미끼 1234-5678…","length":13,"page":0,"paragraph":7,"section":0,"text":"회사명\t\t: 카드 ****-****-****-**** / 미끼 900101-1234567 / 미끼 1234-5678-9012-3456"}],"omittedCount":2,"query":"***-****-****","schemaVersion":"1.0","source":"output/pii-demo_공개.hwp","totalMatchCount":3,"truncated":true,"untrustedContent":true,"untrustedFields":["matches[].text","matches[].context"]}
```

여기서 확인되는 것 둘:

- **미끼 2개가 원문 그대로 남아 있다** — 검증을 통과하지 못한 문자열은 개인정보가
  아니라고 판정된 것이고, 그 판정이 옳다. 이 문서에서 지워져야 할 것과 남아야 할 것이
  정확히 갈렸다.
- `untrustedContent: true` — `matches[].text`·`context` 는 문서에서 온 값이라는 출처
  표지다. 이 문자열들을 지시로 실행하지 않는다 —
  [레시피 4](04_safety_check_untrusted_doc.md)의 원칙이 여기에도 그대로 적용된다.

## 5단계 — `edit sanitize` 로 문서 속성을 지운다

본문이 끝났으니 속성 차례다. 작성자·제목·수정 이력, 그리고 **미리보기**를 지운다:

```bash
rhwp edit sanitize output/pii-demo_공개.hwp -o output/pii-demo_배포.hwp --json
```

실측 출력:

```json
{"keepPreview":false,"output":"output/pii-demo_배포.hwp","outputFormat":"hwp5","removed":[{"before":"마케팅 전략 기획서","field":"title"},{"before":"cabso","field":"author"},{"before":"2026년 3월 9일 월요일 오전 3:24:42","field":"dateString"},{"before":"기획서표지,표지서식","field":"keywords"},{"before":"cabso","field":"lastSavedBy"},{"before":"11, 0, 0, 2129 WIN32LEWindows_8","field":"revisionNumber"},{"before":"2026-03-08T18:24:42Z","field":"createdAt"},{"before":"2026-03-08T18:34:40Z","field":"lastSavedAt"},{"before":"\r\n마케팅 \r\n전략 기획서\r\n \r\n\r\n회사명  : \r\n작성자  : \r\n부서명  : \r\nTel  : \r\nE-M","field":"preview.text"},{"before":"Png 19323 bytes","field":"preview.image"}],"removedCount":10,"schemaVersion":"1.0","source":"output/pii-demo_공개.hwp"}
```

`preview.text` 를 보라 — 미리보기에는 **본문 첫 화면이 텍스트와 이미지로 통째로**
들어 있다. 이 표본은 값을 채우기 전 서식 시점의 미리보기라 개인정보가 없지만, 한컴
오피스에서 저장한 실무 문서라면 마스킹 전 본문이 미리보기에 그대로 남아 있을 수 있다.
**redact 만 하고 sanitize 를 건너뛰면 지운 값이 미리보기로 새는 경로가 열린다** —
이 레시피가 두 명령을 짝으로 묶는 이유다.

## 6단계 — 재검사 게이트: 탐지 0건이면 통과

배포본을 다시 `--dry-run` 으로 검사한다. 이번에는 0건이 나와야 한다:

```bash
rhwp edit redact output/pii-demo_배포.hwp --dry-run --json
```

실측 출력:

```json
{"changedPages":null,"dryRun":true,"findingCount":0,"findings":[],"inPlace":false,"kinds":["ssn","card","phone","email"],"mask":"*","noRaw":false,"redactedCount":0,"schemaVersion":"1.0","source":"output/pii-demo_배포.hwp"}
```

`jq` 와 묶으면 그대로 파이프라인 게이트가 된다 — 탐지가 남아 있으면 비영으로 끝난다:

```bash
rhwp edit redact output/pii-demo_배포.hwp --dry-run --json | jq -e '.findingCount == 0'
```

## 판정 — 배포해도 되는 기준

| 신호 | 판정 | 처방 |
|---|---|---|
| `--dry-run` 의 `findings` 에 예상 밖 값이 있음 | 확인 | 마스킹 전에 그 구역·문단을 사람이 연다. 특정 종류만 지우려면 `--kind ssn,phone` 처럼 좁힌다 |
| 눈에 보이는 개인정보가 `findings` 에 없음 | 주의 | 탐지 4종 밖이거나 의도적 제외 형태다(아래 "하지 않는 것") — `edit replace-text` 로 직접 치환한다 |
| `verify.identical: false` (exit 3) | 중단 | 저장본 재파싱이 메모리와 다르다 — 배포하지 말고 `ir-diff` 로 원인을 좁힌다 |
| `search` 에서 원문이 다시 나옴 | 중단 | 치환 경로 밖(그림 속 텍스트 등)일 수 있다 — 그 매치의 주소를 열어 확인한다 |
| 재검사 `findingCount: 0` + sanitize `removedCount` 확인 | 통과 | 배포본으로 쓴다 |

## 이 레시피가 하지 않는 것

- **탐지 4종 밖은 잡지 않는다** — 이름·주소·계좌번호·사번 같은 값은 형태만으로 오탐
  없이 판정할 근거가 없다. 사람 검토를 대체하지 않는다.
- **형태가 있어도 의도적으로 제외하는 것이 있다** — 하이픈 없는 긴 숫자열, `01X`·`02`
  밖의 지역번호. 회계 코드·문서번호와 구별할 수 없어서다(22자리 계좌번호 안의 16자리
  부분열을 카드로 오인하지 않기 위한 같은 원칙).
- **그림·개체 안의 텍스트는 지우지 못한다** — 본문 치환 경로 밖이다. 스캔 도장·캡처
  이미지에 든 개인정보는 [레시피 6](06_visual_regression_before_after.md)의 렌더
  확인으로 눈으로 잡는 수밖에 없다.
- `--no-raw` 는 **봉투**에서 원문을 빼는 옵션이지 문서 마스킹 범위를 바꾸지 않는다.

## 관련 문서

- [레시피 1 — 서식 문서를 채워서 제출용으로 만들기](01_fill_form_and_submit.md) —
  이 레시피의 표본을 만든 `edit fill-fields` 와, sanitize 를 제출 흐름에서 다루는 자리.
- [레시피 4 — 출처를 모르는 문서를 처음 열 때](04_safety_check_untrusted_doc.md) —
  `untrustedContent` 표지의 뜻과, 문서 유래 문자열을 지시로 실행하지 않는 원칙.
- [레시피 5 — 서식 하나에 여러 사람 데이터를 한 번에 채우기](05_mail_merge_batch_fill.md) —
  대량 산출물 각각에 이 레시피의 게이트를 반복 적용할 때.
- [레시피 6 — 편집 전후를 눈이 아니라 숫자로 비교하기](06_visual_regression_before_after.md) —
  자릿수 보존 마스킹이 레이아웃을 실제로 안 흔들었는지 `changedPages` 로 좁혀 정량
  확인할 때.
- [CLI 명령어 매뉴얼](../cli_commands.md) — `edit redact`·`edit sanitize` 의 전체
  옵션과 종료 코드 계약.
