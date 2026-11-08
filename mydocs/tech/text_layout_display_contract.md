---
kind: reference
status: active
canonical: mydocs/tech/text_layout_display_contract.md
last_verified: 2026-10-23
---

# 텍스트 레이아웃의 모델과 표시 문자열

`getPageTextLayout(page)`와 native `get_page_text_layout_native(page)`는 같은 JSON 계약을 쓴다.
읽기 전용 CLI `rhwp-q text-layout <파일> --page 0 --json`의 `runs`에도 적용된다.

| 필드 | 의미 |
| --- | --- |
| `text` | 기존 모델 문자열. 자동번호/필드 자리표시 문자를 보존한다. |
| `charStart` | 기존 모델 공간의 런 시작 오프셋. 기존 문서·셀 좌표와 함께 사용한다. |
| `charX` | 기존 모델 문자열의 런 상대 X 경계. 기존 계약을 유지한다. |
| `displayText` | 표시 치환이 있을 때만 제공되는 문자열. 빈 문자열도 유효하다. |
| `displayCharX` | `displayText`와 함께 제공되는 표시 문자열의 런 상대 X 경계. |
| `x`, `y`, `w`, `h` | 기존 렌더 트리 런의 페이지 좌표와 크기. |

표시용 문자열은 `run.displayText ?? run.text`, 표시용 경계는
`run.displayCharX ?? run.charX`로 선택한다. `||`는 빈 표시 문자열을 모델 문자로
되돌리므로 사용하지 않는다. 경계 배열은 해당 문자열의 Unicode scalar N개에 N+1개이며,
페이지 X는 런의 `x`에 상대 경계를 더한 값이다. JavaScript UTF-16 `string.length`와
scalar 수가 다를 수 있다. 기존 출력처럼 좌표는 소수점 한 자리로 직렬화한다.

필드 하나가 파일 이름 여러 글자로 표시되거나 한 런에서 여러 자동번호가 치환될 수 있다.
따라서 표시 인덱스를 `charStart`에 더해 편집 위치를 만들지 않는다. 이 API는 표시 글자별
모델 대응표를 제공하지 않는다. 선택·커서·편집에는 기존 모델 좌표와 해당 hit-test/selection API를
사용한다. 표시 문자열을 `text`에 덮어쓰거나 `displayCharX`를 모델 경계로 전달하면 안 된다.

표시 치환이 없는 문단은 두 새 필드가 없고 기존 JSON 필드를 그대로 유지한다.
추가 필드를 무시하는 기존 소비자는 모델 동작을 유지한다. 표시 치환은 자동 쪽번호에만
한정하지 않고 렌더 트리에 이미 있는 `display_text`를 공통으로 내보낸다.
이 export는 조판·SVG·Canvas·저장 모델을 바꾸지 않는다.

## 소비자 직접 확인

1. [#7472 공개 ZIP](https://github.com/user-attachments/files/32746158/repro_footer_picture.zip)의
   두 HWP를 열어 `JSON.parse(doc.getPageTextLayout(0))`를 얻는다.
2. `displayText`에 자동 쪽번호 `1`이 있는 런의 `x`, `y`, `displayCharX`를 확인한다.
3. 일반 꼬리말 문구가 유지되는지 확인한다. 표시 문자열을 모을 때 `displayText ?? text`를 쓴다.
4. 파일명 필드처럼 모델 1자가 여러 글자로 표시되는 입력에서 기존 hit-test로 필드 뒤를
   선택하고 글자를 삽입한다. 표시 문자열 길이 대신 모델 오프셋을 사용했는지 확인한다.

[이슈 #7909](https://github.com/edwardkim/rhwp/issues/7909)의 API 소비자 검토 항목은
표시 문자열과 편집 오프셋을 실제 앱에서 구분하는지 확인하는 것이다.
그림·사각형 배치(#7910)나 독립 한컴 Print PDF와의 시각 일치는 이 계약의 검증 주장이 아니다.
