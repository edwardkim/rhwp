---
kind: investigation
status: active
canonical: gym/packs/serialization/README.md
last_verified: 2026-08-18
---

# serialization pack 예외·가장자리 노트

이 문서는 저장·변환 과제가 **일부러 박제하지 않는 값**과, 현장에서 반복되는
가장자리 실패를 기록한다. 여정 지도는
[gym/packs/serialization/README.md](../../../gym/packs/serialization/README.md),
작업 계보는 [gym_serialization_pack.md](gym_serialization_pack.md).

값은 환경·재조판·폰트·직렬화 순서에 흔들린다. 그래서 과제 JSON에 숫자를
넣지 않고, 채점 시점에 rhwp가 같은 명령을 다시 돌린다. 아래는 "흔들린다"가
정확히 무엇을 뜻하는지의 목록이다.

## E1. extract-pages 재조판

`extract-pages` 는 쪽 단위로 자르되 **문단 단위로** 지운다.

- 여러 쪽에 걸친 문단은 한 쪽이라도 `--from/--to` 안이면 남는다.
- 남긴 문단을 다시 조판하면 쪽 수가 요청 폭과 달라질 수 있다.
- 구역·DocInfo·BinData 는 그대로 남는다. 그림이 많은 문서는 첫 쪽만
  남겨도 파일 크기가 크게 줄지 않을 수 있다.

그래서 SR10/SR28/SR31 은 `pagesAfter` 를 라이브로 읽고, SR14/SR22/SR29 는
`pagesBefore`(원본 쪽수, 비교적 안정)를 읽는다. `pagesAfter == (to-from+1)`
을 단언하지 않는다.

중첩 셀 표본(`issue2007_nested_cell_pagination_42065.hwp`, SR22)은 이
가장자리를 드러내려고 골랐다. 셀이 쪽을 넘기면 문단 단위 삭제가 쪽 가위와
어긋난다.

## E2. 쪽 축이 명령마다 다르다

같은 단어 "page" 가 세 가지 기준을 가리킨다.

1. `extract-pages --from/--to` — **1 기준**. 첫 쪽이 1.
2. `export-pdf -p`, `export-text` 의 `pages[].page`, `search` 의
   `matches[].page` — **0 기준**. 첫 쪽이 0.
3. `pageCount` / `renderedCount` / `pagesBefore` / `pagesAfter` — 개수.
   기준이 아니라 합이다.

에이전트가 `search` 결과 `page: 1` 을 `--from 1` 에 넣으면 둘째 쪽이
아니라 첫째 쪽이 남는다. 오류 메시지 없이 한 쪽 밀린 문서가 나온다.
SR18 은 둘째 쪽(`--from 2 --to 2`)을 일부러 넣어 이 함정을 밟게 한다.

`export-pdf -p` 는 이 pack 의 과제에 넣지 않았다. 0 기준과 1 기준을 한
과제 안에 섞으면 채점 힌트가 힌트 역할을 못 한다. 단일 쪽 PDF가 필요하면
별도 과제에서 `-p 0` 을 명시해야 한다.

## E3. PDF 바이트는 정답이 아니다

`export-pdf` 산출물은 다음이 바뀌면 바이트가 달라진다.

- 시스템 폰트 목록, `RHWP_FONT_PATH`, `--font-path`
- `--backend svg|direct` (direct 는 `native-skia` feature)
- `--text-as-paths`
- fallback family (Windows 바탕/맑은 고딕, Linux Noto, macOS Apple)
- 수식 폰트

SR11/SR15/SR19/SR23/SR32–SR35/SR49/SR50 은 바이트를 비교하지 않는다.
`file_exists` + `differs_from_input` + `format`/`backend`/`pageCount`/
`renderedCount` 만 본다.

`pageCount` 와 `renderedCount` 는 전체 문서 내보내기에서 보통 같다.
SR23/SR34 가 `renderedCount` 를 따로 묻는 이유는 필드 혼동을 가르기
위해서다. 두 값이 갈라지는 경우는 부분 렌더(`-p`)나 렌더 실패인데, 이
pack 은 전체 문서를 보낸다.

## E4. convert 는 "배포용 해제만" 이 아니다

입력이 이미 편집 가능한 HWP5 여도 `convert` 는

- 산출 경로에 파일을 쓰고
- `wasDistribution` 을 봉투에 실어
- `--verify` / `--verify-pages` 를 옵션으로 받는다.

SR13/SR26 은 이 계약을 묻는다. `wasDistribution: true` 를 박제하면
편집 가능 표본에서 틀린다. 배포용 표본을 이 pack 에 넣지 않은 이유는
암호·배포 플래그 조합이 보안 pack 과 겹치기 때문이다.

`convert` 출력 확장자는 `.hwp` 만 허용한다. `.hwpx` 를 주면 입력을
읽기 전에 exit 2 다. HWPX 가 필요하면 `export-hwpx` 다 (SR01/SR06/
SR40–SR42/SR51).

## E5. 검증 실패는 산출물을 삼키지 않는다

`--verify` 가 IR 차이를 보면 exit 3, `--verify-pages` 가 쪽수 차이를
보면 exit 4. **파일은 남아 있다.**

이 규약을 어기면 파이프라인이 "실패 = 산출 없음" 으로 잘못 갈라진다.
과제는 `file_exists` 를 먼저 두고, 그다음 `answer_eq` 로 검증 객체를
읽는다. `expect_exits` 에 0과 3(또는 4)을 같이 넣는다.

`verify` / `verifyPages` 객체는 해당 옵션을 준 경우에만 생긴다.
옵션 없이 `verify.identical` 을 읽으면 경로가 `null` 이라 실패한다.
SR17/SR21/SR27/SR41/SR42/SR56 의 기준풀이는 옵션을 켠 명령을 그대로
다시 돌린다.

## E6. ir-diff 의 세 갈래

1. **자기대조** (SR16/SR36/SR53): 같은 경로를 두 번 읽는다. `identical`
   은 true 여야 한다. false 면 파서 비결정성이므로 pack 이 아니라
   코어 이슈다.
2. **짝 파일** (SR12/SR20/SR37): 저장소에 있는 HWP·HWPX 쌍. 동일할 수도
   있고 차이가 날 수도 있다. 값을 박제하지 않는다.
3. **다른 문서** (SR24/SR38) 또는 **연도 쌍** (SR39): 보통 차이가 나고
   exit 3 이다. `identical` 과 `diffCount` 를 나눈다.

stdout 은 차이가 있어도 JSON 한 줄이다. 읽기 실패만 stdout 을 비운다.
에이전트가 exit 3 을 "명령이 죽었다" 로 읽으면 봉투를 버린다.

`categories` 필드는 이 pack 이 묻지 않는다. 카테고리 이름이 바뀌면
과제가 전부 흔들린다. 규모는 `diffCount` 면 충분하다.

## E7. DocLang 손실은 실패가 아니다

`export-doclang` 은 표현할 수 없는 정보를 `lossCount` 로 보고하고
변환 자체는 성공한다. SR03/SR45/SR52 는 그 숫자를 맞추는 과제이지
0 을 요구하는 과제가 아니다.

`format` 은 `"doclang"`, `doclangVersion` 은 `"0.6"` 근처다. 후자를
과제에 박제하지 않고 SR44 가 봉투에서 읽게 했다. 스키마 버전
(`schemaVersion: "1.0"`)과 섞으면 틀린다.

입력은 HWP5/HWPX 만 받는다. HML·HWP3·DRM·빈 파일은 사용법 오류다.
이 pack 의 DocLang 과제는 그 거부 경로를 시험하지 않는다.

## E8. export-hml 은 원본 형식이 이미 HML 일 때만

SR02 는 `samples/hml/formatting_table.hml` 만 받는다. HWP 를 넣으면
거부된다. "아무 문서나 HML 로" 가 아니라 "HML 원본을 다시 HML 로" 다.
계약을 읽고 대상을 고르는 것 자체가 과제다.

산출물은 `info` 로 열려야 하고 (`pageCount >= 1`) 원본과 바이트가
달라야 한다. HML 직렬화는 속성 순서·공백이 달라질 수 있어 해시 비교는
쓰지 않는다.

## E9. IR 스키마는 문서를 받지 않는다

SR05 의 `export-ir-schema` 는 입력이 필요 없다. 과제 `input` 필드는
스키마상 필수라 `samples/table-001.hwp` 를 적었지만 명령은 그 파일을
열지 않는다. `differs_from_input` 은 스키마 JSON 이 HWP 가 아님을
확인할 뿐이다.

`dialect` 는 JSON Schema 2020-12 URI 다. 이 문자열을 과제에 박제한
이유는 스키마 방언이 바뀌는 것이 곧 계약 파기이기 때문이다. 쪽수처럼
환경에 흔들리는 값이 아니다.

## E10. 최소 바이트와 빈 껍데기

`file_exists` 의 기본 `minBytes` 는 1 이다. 변환 산출은 256, 스키마는
128 로 올렸다. `%PDF-` 나 `PK` 시그니처만 있는 수 바이트 가짜 파일은
여기서 걸린다. 진짜 시그니처 검사는 `info` / `export-pdf --json` 의
`format` 이 맡는다.

`xml_root_eq` 는 SVG 과제용이라 이 pack 에 쓰지 않았다. PDF 는 XML이
아니다.

## E11. 자리표

기준풀이는 `{sub:conv.hwp}` 처럼 제출 폴더 자리표를 쓴다. 과제의
`cmd` 는 `{file:conv.hwp}` 다. 둘을 섞으면 채점기가 파일을 못 찾는다.
구조대 초안과 기존 SR09–SR12 가 이 구분을 이미 지키고 있어 후속
과제도 그대로 따랐다.

다세대 계획서에서 `{sub:}` 가 여러 번 나오면 **전부** 치환해야 한다.
첫 하나만 바꾸면 다음 세대가 입력을 잃는다. 이 pack 의 기준풀이는
한 세대(변환 한 번 + 답 한 번)라 그 함정이 작지만, SR48 은 convert
run 다음에 info answer 가 이어지므로 같은 `conv.hwp` 자리표를 두
단계에서 쓴다.

## E12. unavailable 과 0점

`pack.json` 의 `requires.commands` 에 없는 바이너리로 이 pack 을
돌리면 점수는 0 이 아니라 `unavailable` 이다. 부재를 실패로 위장하지
않는 것이 gym 계약이다.

이 확장이 `convert` · `export-pdf` · `extract-pages` · `export-hwpx` 를
requires 에 넣은 이유다. 오래된 바이너리는 이 pack 을 건너뛰고, 새
바이너리만 채점한다.

## E13. 과제 ID 와 프로파일

`SR*` 는 전역 고유해야 한다. 다른 pack 이 SR 접두사를 쓰면
`audit.py` 가 충돌로 막는다. 프로파일(`profiles/*.json`)은 pack id
목록이지 과제 id 목록이 아니다. 과제를 늘려도 프로파일 JSON 은
바꾸지 않는다.

`gym/README.md` 의 "serialization 8과제 · 만점 19" 는 이 확장 이후
숫자가 틀린다. 집계 문서는 별도 커밋으로 고친다. 이 PR 범위는
pack 내부와 계약 테스트다.

## E14. 윈도우 경로와 한글 표본

표본 경로에 공백·한글이 있는 파일(`2025 행정업무운영 편람` 등)은
고르지 않았다. 채점기 자리표 치환은 견디지만, 문서와 셸 예제가
깨지기 쉽다. ASCII 경로의 작은 표본만 썼다.

Windows 에서 `os.path.join` 이 백슬래시를 만든다. 과제 JSON 의
`samples/...` 는 슬래시로 적는다. 채점기가 저장소 루트 상대 경로로
연다.

## E15. 이 pack 이 의도적으로 안 밟는 가장자리

- 입력==출력 경로 거부 (원본 보호). gym 제출은 항상 다른 폴더다.
- `--output-password` 왕복. 보안·암호 pack 범위.
- DRM·배포용 전용 표본. 위에서 말한 겹침.
- `export-pdf --backend direct` 강제. feature 없는 CI 가 깨진다.
- 한컴 정본 PDF 픽셀 비교. fidelity harness 범위.
- 빈 파일·잘린 OLE 손상. robustness 도구 범위.

이 목록을 과제로 만들고 싶으면 새 CLI 없이 기존 명령의 거부 경로를
`expect_exits: [2]` 로 받는 과제를 따로 설계해야 한다. 이번 확장은
성공 경로의 필드 지목에 집중했다.

## 빠른 대조표

| 흔들리는 것 | 박제? | 대신 하는 일 |
|---|---|---|
| PDF/HWP 바이트 | 금지 | `file_exists` + `differs_from_input` |
| `pagesAfter` | 금지 | `answer_eq` 실측 |
| `pageCount` / `renderedCount` | 금지 | `answer_eq` 실측 |
| `backend` | 금지 | `answer_eq` 실측 |
| `wasDistribution` | 금지 | `answer_eq` 실측 |
| `lossCount` / `diffCount` | 금지 | `answer_eq` 실측 |
| `doclangVersion` | 금지 | `answer_eq` 실측 |
| `format` (`hwp5`/`hwpx`/`pdf`/`doclang`) | 허용 (`value_eq`) | 계약 문자열 |
| IR 스키마 `dialect` | 허용 (`json_value_eq`) | 계약 URI |
| `--from/--to` 가 1 기준 | 문서에 고정 | instructions + 테스트 |

## 관련 테스트

`scripts/tests/test_gym_serialization_pack.py` 가 아래를 고정한다.

- SR01–SR56 이 빠짐없이 있고 기준풀이와 id 가 같다
- 산출 과제는 `differs_from_input` 을 가진다
- `extract-pages` 과제는 1 기준 인자를 쓴다
- `ir-diff` / `--verify` 는 exit 0·3, `--verify-pages` 는 0·4
- 과제 JSON 에 골든 해시·고정 쪽수가 없다
- 모든 `input` 경로가 저장소에 실재한다
- README 가 여정과 실패 모드를 한국어로 적는다
