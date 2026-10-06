# PR #7200 한컴 재조판 대조군

3회차에서 정상 `width-*.hwp`를 원래 `samples/stored-nested-content-flow/width-*.hwp`
경로로 옮겨 입력 자체를 보정했다. 중복 원문을 피하기 위해 이 폴더의 width 사본은 제거했다.
아래 width 파일명·해시는 모두 samples 경로를 가리킨다. 기준 PDF는 기존 것을 그대로 재사용한다.
원래 200자/1줄의 합성 입력과 실패 증적은 commit `78fd17cf8`에 보존한다.
기존 실패 PDF `pdf/pr7200/pr7200-width-*-2020.pdf`는 그 과거 입력에만 대응한다.

## 생성 절차

1. 기존 `width-top.hwp`를 HWP MCP의 engine 2020으로 HWPX에 저장했다.
2. `Contents/section0.xml`의 `hp:linesegarray`만 제거했다. 텍스트, 셀 크기, 표 위치,
   테두리와 글자 스타일은 유지했다. Center/Bottom은 바깥 셀 `hp:subList@vertAlign`만 바꿨다.
3. 이 HWPX들을 같은 엔진에서 HWP로 저장하여 한컴이 유효한 줄 캐시를 생성하게 했다.
4. 최종 HWP 각각을 같은 엔진에서 PDF로 변환했다. 실제 엔진은 Hancom **12.0.0.4605**다.
   start → status `succeeded`/`terminal=true` → download 순서로 완료를 확인했다.

정상 생성된 긴 문단은 textpos `0/45/90/135/180`, vpos `0/1688/3376/5064/6752`의
5줄이며, `line_height=1000`, `text_height=1056`, `line_spacing=632`다.
단일 줄 재조판 계약은 `damaged_width_nodes`가 정상 5줄 저장본을 메모리에서 1줄로 손상시켜 검사한다.

| 최종 HWP | SHA256 |
| --- | --- |
| `width-top.hwp` | `335e54e427a2aa5c8f8c4f5cbc32c37c591553aabd65ec1d984a1561e4317857` |
| `width-center.hwp` | `b5c2bb3e6e9986fc18aa4f69309d59b7c46498f7220fbb251aa06efe1afb6009` |
| `width-bottom.hwp` | `367d526c1ced41a32b8654cf79776d79b68b70af05a47bf699f912df16ef6e50` |

## 독립 기준과 제한

기준 PDF는 `pdf/pr7200/width-{top,center,bottom}-recomposed-2020.pdf`다.
`tests/cases/stored_nested_content_flow.rs::hancom_recomposed_lines_and_nested_table_match_pdf_positions`
검사는 PDF에서 추출한 5줄과 `End`의 96 DPI 위치, 별도 셀/본문 문단 테두리를 확인한다.
1px 좌표 허용 범위는 PDF 글리프 상자와 renderer 줄 상자의 차이를 허용하며 80px 가로 이동이나
줄마다 56HU가 누적되는 이전 오차를 숨기지 않는다. 픽셀 전체 일치를 뜻하지 않는다.

최종 시각 판정과 남은 차이는 `mydocs/pr/archives/pr_7200_review.md`를 따른다.
최종 samples 입력의 통과를 과거 합성 HWP/PDF의 시각 일치로 보고하지 않는다.

| 출력 | HWP 변환 job | PDF 변환 job |
| --- | --- | --- |
| Top | `23ad8834-17de-4200-a27e-49b4eac77752` | `304aefea-c842-4258-9572-9a24b606cb88` |
| Center | `a989b91d-b907-498e-b10d-20a9c75d5406` | `1a81ff29-129f-450e-9854-af0f46dc55a4` |
| Bottom | `9c01d2bb-9782-49dc-a3eb-5f352d0cfe06` | `d5a624d6-5e37-4bee-8c09-66580754ee33` |


## 추가 테두리의 원인 분리와 보정

`host-char-border-off.hwpx`는 위 재조판 직전 HWPX에서 바깥 표를 포함하는 run의 글자 테두리만
없는 스타일로 바꾼 통제 입력이다. 텍스트와 표 크기·셀/문단 테두리는 그대로이며 표를 숨기거나 크기를
줄이지 않았다. `pdf/pr7200/host-char-border-off-2020.pdf`에서는 바깥 표 아래의 추가 외곽선이
사라진다. 따라서 이 선을 셀/문단 테두리의 중복으로 삭제할 수 없다. 1회차에서 누락했던 객체 글자 테두리는
2회차에서 아래 독립 여백 계약에 따라 복원했다. 통제군 PDF를 원본의 정답 PDF로 대체하지 않는다.
변환 job: `2d1152d4-867c-4b3f-b526-42a33618b53c`, engine 2020, terminal succeeded.

## 객체 글자 테두리의 최소 여백 통제군 (2회차)

`margin-min-700.hwp`, `margin-min-800.hwp`, `margin-both-1000.hwp`,
`margin-both-2000.hwp`는 재조판 직전 HWPX에서 바깥 표의 `outMargin`만 바꾼 입력이다.
`min`은 아래 여백만, `both`는 위·아래를 함께 바꾼다. 각 입력을 engine 2020으로
HWP에 저장해 유효한 줄 정보를 생성하고, 최종 HWP에서 출력한 PDF를
`pdf/pr7200/<이름>-2020.pdf`에 보존한다.

| 입력 | 변환 job | 글자 테두리 하단 − 실제 표 하단 (96 DPI px) |
| --- | --- | ---: |
| `width-top.hwp` (기존 정상 대조군) | 위 표 참조 | 18.699 |
| `margin-min-700.hwp` | `8a0a8515-27a6-4429-bda1-967c98edaac6` | 18.716 |
| `margin-min-800.hwp` | `35abd657-1e14-42dd-b938-39e6e8eb886e` | 19.996 |
| `margin-both-1000.hwp` | `ab6ca4e5-9475-44c2-bd2e-7dda9973a68a` | 13.597 |
| `margin-both-2000.hwp` | `1573f273-06cb-4da8-9776-478a011acada` | 26.875 |

이는 단독 TAC 표 run의 테두리 높이에 위·아래 **각각 최소 2.5mm(708 HU)**가 적용되는
한컴 출력 계약을 분리한다. 경계 아래·위, 양쪽 증가가 있어 고정 14pt 하단 보정으로는 통과할 수 없다.
테두리 높이는 실제 표 높이 + max(위 여백, 708) + max(아래 여백, 708)이며, 테두리 원점은
표의 실제 위 여백을 빼서 줄 원점으로 환산한다. 표 배치·페이지 흐름을 바꾸는 여백이 아니다.
PDF 프린터의 가는 선 양자화 차이(1px 미만)는 남는다. 2.5mm는 파일 사양에 명시된 항목이라고
주장하지 않으며, 독립 한컴 출력에서 확인한 paint 호환 규칙이다.

여러 텍스트/객체가 같은 run에 있는 연결 테두리와 쪽 분할 조각은 이번 단독 객체 테두리의
검증 범위가 아니다. 기존 텍스트 테두리 처리에 단독 객체 사각형을 중복해서 덧그리지 않는다.

HWP 저장 job은 순서대로 `0a4d426b-5092-453b-8c03-807922e5de05`,
`ee52f6a0-5d7e-4a35-9ea6-2df607af472d`, `e733135d-fb6f-4caa-9626-da8156177f1a`,
`df791f40-ded7-467e-a2d1-185c2258ee9e`다. 모든 job의 succeeded/terminal=true를 확인했다.
이 재생성은 PDF 형식 버전을 바꾸려는 절차가 아니다. 저장 줄이 없는 실험 HWPX와 실제 저장 줄을
갖춘 최종 HWP를 구분하고, 최종 검증 입력 자체에 대응하는 PDF를 확보한 것이다.

## 최종 검증 입력과 기준 PDF SHA256

| 입력 / 기준 PDF | 입력 SHA256 | PDF SHA256 |
| --- | --- | --- |
| `width-top.hwp` / `width-top-recomposed-2020.pdf` | `335e54e427a2aa5c8f8c4f5cbc32c37c591553aabd65ec1d984a1561e4317857` | `fd92cc8be8ced88c955767d13affcc825dd8d10f5e1293ccd6cc23956a738c72` |
| `width-center.hwp` / `width-center-recomposed-2020.pdf` | `b5c2bb3e6e9986fc18aa4f69309d59b7c46498f7220fbb251aa06efe1afb6009` | `15f651d35c725fbbeca561989b9c2e76e0fa51e592932148ac401b97ad1188b9` |
| `width-bottom.hwp` / `width-bottom-recomposed-2020.pdf` | `367d526c1ced41a32b8654cf79776d79b68b70af05a47bf699f912df16ef6e50` | `04ee54ff37b4691c75689092d2e939c75826f901b518740eefb6491a57aefe5e` |
| `margin-min-700.hwp` / `margin-min-700-2020.pdf` | `4dcc25e2935a35f88deffe2b6f165d2887857dcbafea49c2c364fcb2dd706b16` | `df5ea073e6bbb6a01d54e273545f9c0cf8183466930fb4597505c58f41a4333a` |
| `margin-min-800.hwp` / `margin-min-800-2020.pdf` | `06d0ed2742162e2b0468147ce8ed6ce5c13b1d592a3a38e958ae8aaf3360bd2a` | `1c8653208f2e28cded6ecf648f8b33c315b0531b66ce45ebb39680300ecd8f8e` |
| `margin-both-1000.hwp` / `margin-both-1000-2020.pdf` | `a63cb77e05389a89cdf95422796ca7bd7c3747e4dd90cb9b53831a0fec51f223` | `62c45f2f93f4ca9b7cad88c81d1b63157f05fc98147c35032028f1474312de62` |
| `margin-both-2000.hwp` / `margin-both-2000-2020.pdf` | `2f2bf633aaad7f8ced305c5658da42a2946cb5e3d71083c4bdf0caf7d955ee1f` | `ecd9e6d638a564b989d0aa3fe8b8804012ad6ad91d1643477d2c503566d635ad` |
