---
kind: guide
status: active
canonical: mydocs/manual/recipes/01_fill_form_and_submit.md
last_verified: 2026-08-03
---

# 레시피 1 — 서식 문서를 채워서 제출용으로 만들기

**목표 한 줄**: 누름틀이 있는 빈 서식(`.hwp`/`.hwpx`)을 받아서, 값을 채우고, 필요하면
도장/서명 이미지를 붙이고, 메타데이터를 지운 **제출 가능한 최종 산출물**을 만든다.

이 레시피는 명령 하나로 끝나지 않는다 — "값이 들어갔다"와 "사람이 제출할 수 있는
산출물이다"는 서로 다른 명제이고, 각 단계마다 기계로 확인할 수 있는 판정이 있다.
심화 함정(반복 필드, `ambiguous`, overflow)은 [서식 자동화 심화
가이드](../form_filling_guide.md)가 다룬다. 여기서는 **처음부터 끝까지 한 번에
실행 가능한 순서**를 실측 출력과 함께 보여준다.

절차 요약:

```
fields (필드 확인)
  → edit fill-fields (값 채우기)
  → edit fill-fields --verify (재파싱 대조)
  → [선택] edit insert-image (도장/서명)
  → edit sanitize (메타데이터 제거)
```

모든 명령·출력은 이 저장소의 `samples/form-01.hwp`(명령 단추 예제 문서, 누름틀
`myMsg01` 1개 포함)로 실제 실행해서 얻었다. 지어낸 값은 없다.

## 0단계 — 이 서식이 누름틀 기반인지 먼저 확인한다

`fields`는 문서를 **읽기만** 한다(수정 없음). 누름틀이 하나도 없으면
`fieldCount: 0`이 나오고, 그 경우는 이 레시피가 아니라 `edit set-cell`(표 좌표
채우기) 레시피 대상이다 — [레시피 0 축 선택
표](../form_filling_guide.md#0-축-선택--이-서식은-무엇으로-채우나) 참조.

```bash
rhwp fields samples/form-01.hwp --json
```

실측 출력:

```json
{"fieldCount":1,"fields":[{"command":"Clickhere:set:48:Direction:wstring:6:여기에 입력 HelpState:wstring:0:  ","editableInForm":true,"fieldId":2110609883,"fieldType":"ClickHere","guide":"여기에 입력","location":{"nested":[],"paragraph":10,"section":0},"memo":"","name":"myMsg01","value":""}],"schemaVersion":"1.0","source":"samples/form-01.hwp","textSecurity":{"status":"clean"}}
```

읽어야 할 것 셋:

- `fieldCount: 1` — 누름틀 1개, `fill-fields` 축이 맞다.
- `fields[].name`: `"myMsg01"` — `--data` JSON의 키로 그대로 쓴다.
- `fields[].value`: `""` — 아직 비어 있다(채워진 서식이면 값이 들어 있을 수 있다).
- `textSecurity.status: "clean"` — 이 필드 안에 숨은 콘텐츠·프롬프트 주입 신호가
  없다는 뜻이다. `"clean"`이 아니면 값을 채우기 전에
  [레시피 4](04_safety_check_untrusted_doc.md)로 먼저 점검한다.

## 1단계 — `edit fill-fields`로 값을 채운다

```bash
rhwp edit fill-fields samples/form-01.hwp \
  --data '{"myMsg01":"홍길동 귀하"}' \
  -o form-01_filled.hwp --json
```

실측 출력:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"홍길동 귀하"}],"filledCount":1,"notFound":[],"output":"…/form-01_filled.hwp","outputFormat":"hwp5","schemaVersion":"1.0","source":"…/form-01.hwp","verify":null}
```

판정: `notFound`와 `ambiguous`가 둘 다 빈 배열이면 요청한 필드가 정확히 한 번씩
채워졌다는 뜻이다. 이 둘 중 하나라도 비어 있지 않으면 아직 끝난 게 아니다 —
자세한 원인별 대응은 [서식 자동화 심화
가이드 1-1·1-2절](../form_filling_guide.md#1-fill-fields-심화)을 본다.

`--data`는 인라인 JSON 대신 파일도 받는다(`@경로` 형식). 값이 많거나 외부에서
생성한 데이터를 쓸 때 이쪽이 낫다:

```bash
echo '{"myMsg01":"파일에서 읽은 값"}' > data.json
rhwp edit fill-fields samples/form-01.hwp --data @data.json -o out.hwp --json
```

실측 출력(값만 다름을 확인):

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"파일에서 읽은 값"}],"filledCount":1,"notFound":[],"output":"…/form01_fromfile.hwp","outputFormat":"hwp5","schemaVersion":"1.0","source":"…/form-01.hwp","verify":null}
```

### 먼저 무엇이 바뀔지 미리보기 — `--dry-run`

디스크에 아무것도 쓰지 않고 채움 가능 여부만 판정한다. 필드 이름 오타를 커밋 전에
잡을 때 쓴다.

```bash
rhwp edit fill-fields samples/form-01.hwp --data '{"noSuchField":"x"}' --dry-run --json
```

실측 출력 — 오타 필드가 `notFound`에 그대로 잡힌다:

```json
{"ambiguous":[],"changedPages":null,"confusable":[],"dryRun":true,"filled":[],"filledCount":0,"notFound":["noSuchField"],"schemaVersion":"1.0","source":"…/form-01.hwp"}
```

## 2단계 — `--verify`로 재파싱 대조한다

`edit fill-fields`는 값을 쓴 뒤 산출물을 저장한다. `--verify`를 붙이면 **저장 직후
그 산출물을 다시 읽어** IR을 비교하고, 요청한 값이 실제로 화면에 나타나는지
확인한다. "커맨드가 성공을 반환했다"와 "값이 실제로 들어갔다"는 다른 명제이고,
`--verify`가 그 간극을 메운다.

```bash
rhwp edit fill-fields samples/form-01.hwp \
  --data '{"myMsg01":"홍길동 귀하"}' \
  -o form-01_verify.hwp --verify --json
```

실측 출력 — `verify.identical: true`, `verify.diffCount: 0`이 통과 신호다:

```json
{"ambiguous":[],"changedPages":[0],"confusable":[],"dryRun":false,"filled":[{"name":"myMsg01","occurrence":0,"value":"홍길동 귀하"}],"filledCount":1,"notFound":[],"output":"…/form-01_verify.hwp","outputFormat":"hwp5","schemaVersion":"1.0","source":"…/form-01.hwp","verify":{"diffCount":0,"identical":true}}
```

파이프라인 게이트로 쓸 때는 이렇게 좁힌다:

```bash
rhwp edit fill-fields samples/form-01.hwp --data @data.json -o out.hwp --verify --json \
  | jq -e '.verify.identical and (.notFound|length==0) and (.ambiguous|length==0)' \
  > /dev/null || { echo "채움 실패 — 자세히 보려면 --json 없이 다시 실행"; exit 1; }
```

## 3단계(선택) — 도장·서명을 이미지로 붙인다: `edit insert-image`

관공서 서식은 값만 채워서는 끝나지 않는다 — 직인·서명 이미지가 특정 좌표에
들어가야 완성이다. `edit insert-image`는 mm 단위 좌표로 그림을 앉힌다.

```bash
rhwp edit insert-image form-01_filled.hwp \
  --image seal.png \
  --page 0 --x 100 --y 100 --width 30 --height 30 \
  -o form-01_sealed.hwp --json
```

실측 출력(이 저장소의 표본 PNG로 실행):

```json
{"binDataId":1,"changedPages":[0],"dryRun":false,"height":30,"image":"…/seal.png","output":"…/form-01_sealed.hwp","outputFormat":"hwp5","overflow":[],"page":0,"schemaVersion":"1.0","source":"…/form-01_filled.hwp","verify":null,"width":30,"x":100,"y":100}
```

`x`/`y`/`width`/`height`는 모두 mm, 페이지 좌상단 기준이다. `--page 0`은 첫 페이지
(0부터 시작). `edit fill-fields`와 마찬가지로 `--verify`를 붙일 수 있다 — 저장
직후 재파싱해서 그림이 실제로 그 좌표에 들어갔는지 대조한다.

`overflow` 배열이 비어 있지 않으면 이미지가 페이지 여백을 벗어났다는 뜻이다(삽입
자체는 막지 않는다 — 판단은 호출자 몫).

## 4단계 — `edit sanitize`로 제출 전 메타데이터를 지운다

여기까지는 **보이는 내용**만 다뤘다. HWP 파일에는 작성자명·회사 PC 사용자명·최초
작성 시각·수정 이력·미리보기 썸네일처럼 **화면에 안 보이는 메타데이터**가 같이
저장된다. 관공서·거래처에 제출하기 전에는 이것도 지운다.

```bash
rhwp edit sanitize form-01_sealed.hwp -o form-01_final.hwp --json
```

실측 출력 — 이 저장소 표본 파일에서 실제로 제거된 8개 필드:

```json
{"keepPreview":false,"output":"…/form-01_final.hwp","outputFormat":"hwp5","removed":[{"before":"edward","field":"author"},{"before":"2026년 3월 16일 월요일 오전 6:56:40","field":"dateString"},{"before":"edward","field":"lastSavedBy"},{"before":"8, 0, 0, 466 WIN32LEWindows_7","field":"revisionNumber"},{"before":"2026-03-15T21:56:40Z","field":"createdAt"},{"before":"2026-03-16T08:41:55Z","field":"lastSavedAt"},{"before":"\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n\r\n여기에 입력\r\n\r\n","field":"preview.text"},{"before":"Gif 1415 bytes","field":"preview.image"}],"removedCount":8,"schemaVersion":"1.0","source":"…/form-01_sealed.hwp"}
```

읽는 법: `removed[].before`는 **지워지기 전 실제 값**을 그대로 봉투에 담아 반환한다
— "무엇이 새어나가고 있었는지"를 사람이 확인하고 나서 지우기로 결정할 수 있게
하기 위해서다. 조용히 지우면 원래 뭐가 있었는지 아무도 모른다.

`preview.text`(미리보기 텍스트)와 `preview.image`(썸네일 GIF)까지 지워지는 것에
주의한다 — 탐색기 미리보기가 사라진다는 뜻이다. 미리보기를 남기고 싶으면
`--keep-preview`를 붙인다.

## 실패했을 때 무엇을 보고 어떻게 고칠지

| 신호 | 원인 | 처방 |
|---|---|---|
| `fields --json`의 `fieldCount: 0` | 이 서식은 누름틀이 아니라 표 칸이다 | `edit set-cell` 축으로 전환 — [레시피 0 축 선택](../form_filling_guide.md#0-축-선택--이-서식은-무엇으로-채우나) |
| `fields[].textSecurity.status`가 `"clean"`이 아님 | 필드 안내문·현재값에 숨은 콘텐츠 신호 | 채우기 전에 [레시피 4](04_safety_check_untrusted_doc.md) 절차로 먼저 점검 |
| `fill-fields` 응답의 `notFound`에 필드명이 남음 | `--data`의 키가 실제 필드 이름과 다름(오타/공백) | `fields --json`의 `name`을 그대로 복사해 쓴다 |
| `ambiguous`가 비어 있지 않음 | 같은 이름의 필드가 문서에 여러 번 있음 | `이름[0]`, `이름[1]`처럼 순번을 붙여 재지목 — [심화 가이드 1-1절](../form_filling_guide.md#1-1-반복-필드--같은-이름이-여러-번-나올-때-이름n) |
| `--verify`의 `identical: false` | 저장 후 재파싱한 값이 요청과 다름 — 문서 구조가 특이한 케이스 | `--verify` 없이 산출물을 `export-svg`로 육안 확인, [레시피 6](06_visual_regression_before_after.md)의 `render-diff`로 정량화 |
| `insert-image`의 `overflow`에 항목이 있음 | 이미지가 페이지 여백을 벗어남 | `--width`/`--height`를 줄이거나 `--x`/`--y`를 조정 후 재실행 |
| `sanitize`의 `removedCount: 0` | 이미 정리된 문서이거나 애초에 메타데이터가 없음 | 정상 — 재확인하려면 `-o` 없이 원본을 그대로 다시 sanitize 해도 안전(멱등) |

## 관련 문서

- [서식 자동화 심화 가이드](../form_filling_guide.md) — 반복 필드·`ambiguous`·
  overflow 판정 규칙의 canonical 문서.
- [CLI 명령어 매뉴얼](../cli_commands.md) — `fields`·`edit fill-fields`·
  `edit insert-image`·`edit sanitize`의 전체 옵션.
- [레시피 4 — 출처를 모르는 문서를 처음 열 때](04_safety_check_untrusted_doc.md) —
  0단계의 `textSecurity.status`가 `"clean"`이 아닐 때 밟는 절차.
- [레시피 3 — 배포 전 개인정보 마스킹](03_redact_before_sharing.md) — sanitize와
  짝을 이루는 본문 PII 마스킹(`edit redact`)을 처음부터 끝까지 실측으로 따라가는 절차.
- [에이전트 실무 대체 예제집 1절](../agent_task_playbook.md) — 사람 업무 관점의
  더 넓은 시나리오.
