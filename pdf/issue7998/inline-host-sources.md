# B14 inline 그림 host의 독립 인쇄 근거

기존 독립 연산자 원본 `cell-picture-first.pdf`와 `cell-two-pictures.pdf`를 PDF → IR로
불러온 뒤 공개 Native 편집 API를 사용했다. 수동 LineSeg나 현재 RenderTree 좌표를
정답으로 작성하지 않았다. 원본 생성기는 `generate_composition_sources.py`다.

- `cell-picture-first.pdf`: SHA-256 `c70ea86d888cf412a4a434a2779648dcab825783b042ec6e34543f95a920ad24`.
- `cell-two-pictures.pdf`: SHA-256 `33fad600a876513d0b68e1805e6912178e851d9b830a1c38b016ebf6bf04dc52`.

## 편집 입력과 독립 기준

정식 검사 `tests/cases/issue_7998_pdf_table_composition.rs::inline_host_case`에 같은 입력
생성과 편집 순서를 보존했다. `long-prefix`는 빈 그림 host의 offset 0에 `EDIT `를
12회 삽입한다. `short-prefix`는 1회, `explicit-break`는
`EDIT EDIT\nEDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT EDIT `다.
`long-suffix`와 `middle`은 다음 글자 문단을 그림 host에 merge한 뒤 각각 끝과 시작에
12회를 삽입한다. `two-pictures`는 두 그림 host를 merge한 뒤 시작에 12회를 삽입한다.
`outer-margins`는 API로 좌/우 1500 HU, 위 300 HU, 아래 375 HU를 설정한 뒤
`EDIT EDIT EDIT `를 삽입한다. 그림 크기·원래 이미지 바이트는 바꾸지 않는다.

7개 글꼴 대조군은 **편집 전에** DocInfo의 모든 글꼴 슬롯을 Arial로 선택했다.
원래 Helvetica를 지원한다고 가장하거나 제품 코드에서 이름을 바꾸지 않는다.
한컴 2020은 원래 저장본의 Helvetica를 Haansoft Batang으로 대체했으므로, 같은 글꼴의
독립 배치를 확인할 대조군을 따로 만들었다. 아래 Print에는 ArialMT가 실제 포함돼 있다.
원래 face를 유지한 `long-suffix`는 Native 88.50536%이며 글꼴·출력 판정은 P5에 남는다.
원래 재현과 별도 글꼴 대조군의 결과를 섞어 원본 전체 통과로 보고하지 않는다.

`inline-host-2020/`의 7개 HWPX는 실제 API 편집을 마친 **인쇄 요청 입력의 byte 복사본**이다.
초기 보정 producer가 저장한 입력이며 최종 후보의 좌표를 기대값으로 사용하지 않는다.
독립 한컴이 이를 열고 재조판·인쇄한 결과가 같은 이름의 `*-2020.pdf`다.
MCP 성공 결과의 engine/profile은 2020, 한컴 버전은 `11.0.0.9136`, `input_preprocess=none`,
`pdf_print_method=0`, `pdf_output_mode=hancom2020_pdf_driver_one_up`이다. 모두 1쪽이다.
보호된 연결 설정·로컬 글꼴·실행 JSON/로그는 커밋하지 않는다.

| 사례 | Native HWP/HWPX 최저 % | fresh WASM HWP/HWPX 최저 % | 인쇄 job ID |
|---|---:|---:|---|
| long-prefix | 92.53509 | 91.93310 | 6360e253-211d-4d0b-a82c-0d0ba7d2b7d5 |
| short-prefix | 91.24106 | 90.35264 | a76db775-3187-4e8d-a931-968c9874adc7 |
| explicit-break | 92.46134 | 91.80429 | 3ece955c-9fd5-4dd1-be15-455ada5134e9 |
| long-suffix | 92.40972 | 91.73719 | aadcea6e-87ae-4ffc-9f77-0a46227b1b2b |
| middle | 92.53350 | 91.93147 | 223d9ab9-e963-4f13-9403-27b50cdfd64c |
| two-pictures | 92.80098 | 92.06842 | b862b505-c46d-40b3-9dc0-6f6381987fb2 |
| outer-margins | 91.51764 | 90.63913 | b0b13f7d-2968-4c01-9dac-8c24aae1338b |

| 파일 | SHA-256 |
|---|---|
| inline-host-2020/long-prefix.hwpx | `a30833689eeaaa789f69383d6a51372eee310fc1691d18b2ebef3d82e1a8662d` |
| inline-host-2020/long-prefix-2020.pdf | `5316068e37dbd2d9218760eda60504597b708b0b878451ea815436e6f03e87d1` |
| inline-host-2020/short-prefix.hwpx | `b1641fa6a588e4f8d520788e645208163c1fcc01975e1990b472cb2d97cc1466` |
| inline-host-2020/short-prefix-2020.pdf | `2104c6c185bd010c2b955c80b30380521fde6f996acc8384acc307374cdb5bbe` |
| inline-host-2020/explicit-break.hwpx | `57b6cff7b6bf2d2976f45c729f1ed572c1a51abe1af581f2a001c4ba8b8e6174` |
| inline-host-2020/explicit-break-2020.pdf | `0b447a7530f1dcca605a4b89480fab681563a266ecf7a1a4981b145e4c861628` |
| inline-host-2020/long-suffix.hwpx | `45c658301af03b2ba303d0f2dd7699df0d7ce5f54842c0a4819aa361eeb46220` |
| inline-host-2020/long-suffix-2020.pdf | `6c2add49c5e2b71f6035e4483909bfa0c02b28f66fc8580b423af771e6808d46` |
| inline-host-2020/middle.hwpx | `b06c1b04c6330a63e25583604decbca1cbea197732fdd8cd609bcf78943b3117` |
| inline-host-2020/middle-2020.pdf | `aafe700defa4fb71578de9cb5c19bc439ce6f367284e44550efa30378783ec3a` |
| inline-host-2020/two-pictures.hwpx | `91a393322256ea288295364f39fa4cc73c50db52a104d3aff864f13c47451b8f` |
| inline-host-2020/two-pictures-2020.pdf | `dbf2afe8930321f9651f4265749c9fd174acc660db98bf5989709cfce5c76169` |
| inline-host-2020/outer-margins.hwpx | `d20ebea62cf599d6b1f607a22cde295d266dac9f37c766489ad8d65f22b21eaf` |
| inline-host-2020/outer-margins-2020.pdf | `fc4e0c0448d22905f375fa8564b25eed94c77ecbf22a5bd15d34eb66fd267d23` |

## 원래 재현과 복원

원래 face의 실제 편집 재현은 이전 Native producer의 3.60px 셀 넘침과 raw 68.57022%에서
현재 Native 91.74413%, 실제 Studio WASM 90.96021%로 바뀌었다. 양 저장 형식 모두
그림이 물리 셀 안에 남는다. `original-long-prefix-2020.pdf`는 별도의 한컴 2020 Print다
(job `3bf3a9f3-1c9b-4ec7-beb7-df691e866dca`). `original-long-prefix.hwpx`는 같은 편집을
재현한 앞 절편 저장본의 보존 복사본이다. 후속 진단으로 재생성한 작업 파일과 최초 Print
요청 파일의 byte 동일성까지 주장하지 않는다. 원본·편집 순서·당시 로그와 실제 Print를
대조한 증거는 단계 보고에 연결했다.

원래 긴 삽입을 삭제한 Native 및 실제 WASM 복원본은 원본 PDF와 각각
99.99115% / 99.99108%다. 이는 빈 그림 host 재조판 경계를 확인하는 별도 2형식 검사다.
다른 삭제 대조군의 API·소유·저장 보존과 이 원래 PDF의 시각 복원을 구분한다.

- `original-long-prefix.hwpx`: SHA-256 `addf3f34efcca7c85141714fe867356633beb0b85383226f679b2ede9591cfa5`.
- `original-long-prefix-2020.pdf`: SHA-256 `3a24ce648528002ddaf5f2cf7cacf9b6633ea5b553d4c4e89594e7097b4c5696`.

## producer와 판정

시작 checkpoint `9ee2b9dc03773d9ffd040afd7117b3b08d6c2e26` 위 런타임 3개 파일을 수정했다.
source manifest 2,444개 SHA-256:
`6a59224df090e9fea8a5991e3c89bfb86f26831906268eef12e6fef62d020057`.
Native 실행 파일: `501539465924136c6c8de5760a0677c931af60aa3b2916e9746fae81267fcfa6`.
fresh Docker WASM: `7ff8e7f6ddb12f7d9eba58e16965c71c93df130c4ad69ef006b9c6beab98022f`.

2026-10-18 메인테이너의 **“시각·편집 구조 통과”** 판정을 받았다.
그림의 실제 셀 포함·글자/그림 비중첩·원문/이미지 바이트 보존·양 저장 형식과 삭제 복원을
관계 검사로 보호한다. 측정·배치가 공유하는 raw control 슬롯과 실제 LineSeg 소속을
[단계 보고](../../mydocs/working/task_m100_7998_stage4.md#p4-세-번째-절편--b14-긴-텍스트와-inline-그림-host)에 추적했다.
이번 판정은 원래 글꼴의 남은 차이나 P5의 최종 PR gate 예외 승인이 아니다.
