# #7998 연속 셀 경계·편집의 독립 입력과 인쇄 근거

이번 자료는 실물 문서의 좌표를 베낀 golden이 아니라 명시적 PDF 연산으로 만든 작은
독립 입력이다. `generate_native_border_sources.py --output <폴더>`가 source를 만든다.
B15에서는 선행 출력 검증을 마친 solid/mixed 두 입력과 인쇄 대조만 등록했다.
생성기가 함께 만드는 periodic/occluded 부모 반례의 당시 미달 증거는 아래에 보존한다.
후속 B16의 보정·등록은 [paint-save-sources.md](paint-save-sources.md)를 따른다.

## 독립 기대와 재현

- source MediaBox 400×360pt. 부모는 `(20,130,240,180)`, 자식은 `(35,250,90,28)`인
  2×2 격자다. PDF 좌표는 왼쪽 아래 원점이며 root/child 각각 선분을 선언한다.
- source 실선은 `0.5 w`, `MoveTo/LineTo`, butt, 불투명이다. 자식 내부 경계는
  1pt 선분/1pt 실제 틈을 반복한다. 외곽선과 실제 틈을 같은 Solid로 메우지 않는다.
- 문자는 BEFORE CHILD → A/B/C/D → AFTER CHILD 및 이웃 두 셀과 표 밖 앞뒤 문장이다.
  실제 IR·편집·저장 검사는 문자 순서, 실제 Table 2개, 물리 셀 포함을 유지한다.
- `print-source-inputs/`는 위 source의 PDF import 후 모든 font slot에 Arial을 명시한
  HWPX다. 저장 줄 구성·좌표를 수동 작성하지 않았다. 이를 시작점으로 실제
  `insert_table_row_native(...row=0,after=true)` / column의 같은 API,
  또는 첫 셀의 사방 border `Solid/width=5/#d00000`를 적용한 파일이 `print-inputs/`다.
  삽입한 index 1 행/열을 삭제한 복원은 대응 source Print와 별도로 비교한다.
- 양 편집 형식의 실제 WASM 저장본은 같은 동작을 Studio bridge/API로 생성했다.
  자동 점수용으로 Native 파일을 WASM 편집본이라고 재명명하지 않았다.
- `border-tie.hwpx`는 source HWPX에서 셀 BorderFill만 빨강/파랑 checkerboard로
  지정했다(0.2mm). 줄 정보는 수동 변경하지 않았다. 독립 Print에서 공유 세로는 오른쪽
  셀의 왼쪽 선, 공유 가로는 아래 셀의 위 선이다. 네 edge의 반대 색상도 함께 확인했다.

인쇄는 2026-10-18 변환 서비스의 **한컴 2020(11.0.0.9136)**, preprocess none,
원본 용지·100%·한 쪽씩 Print(`hancom2020_pdf_driver_one_up`)다. Print 용지는 드라이버
반올림으로 399×360pt이며 오른쪽 내용이 잘린 세로 360×399pt 실패본과 구분한다.
실행 상태·명령·입력 SHA와 다운로드 로그는 로컬 증거 root의 `*-start/status/download.json`
및 `reference-jobs.json`에 있다. 인증 환경 파일과 공급 글꼴은 자료에 포함하지 않는다.

## 출력·관계 검증

96 DPI, compat 2022, Native Skia와 fresh Docker WASM을 같은 PDF에 대조했다.
source와 PDF→양 저장 재열기는 전체 219쪽 검증에 포함한다. 아래는 이 fixture의
독립 한컴 인쇄 검증이며 대표 쪽만 고른 평균값이 아니다.

| 범위 | 쪽수/backend | 최저 Native / fresh WASM |
|---|---:|---:|
| 두 부모의 행·열·테두리 편집 × HWP/HWPX | 12 | 98.41387% / 97.76556% |
| 두 부모의 삽입 행·열 삭제 복원 × HWP/HWPX | 8 | 98.41387% / 97.76556% |
| 공유 edge checkerboard × HWP/HWPX | 2 | 98.41387% / 97.76556% |

이 자료는 절대 px 좌표 golden을 등록하지 않는다. 정식 검사는 실제 cell BorderFill,
자식의 실제 틈, 새 셀 상속, 문자 읽기 순서·누락/중복, 자식·문자의 물리 셀 포함,
양 저장 재열기 및 shared edge의 기하 소유를 관계로 확인한다. 색상 기대는 위 Print다.
실제 이전 CLI/WASM에서 source/새 행·열이 None인 결과와 공유 4개 edge의 반대 색 FAIL을
보존했고, 보정 후 PASS다. 새 테스트 파일 전체를 이전 core로 재컴파일한 결과와 혼동하지 않는다.

메인테이너는 2026-10-18 비교 HTML·실제 Studio에서 **시각·편집 구조 통과**로 판정했다.
실행·제품 source 지문·테스트 22건·잔여 범위는
[stage5](../../mydocs/working/task_m100_7998_stage5.md)를 따른다.

## 보류한 독립 반례

로컬 생성 원본 `periodic-parent.pdf`, `occluded-parent.pdf`는 직접 PDF 대비 Native/WASM
원본·양 저장 재열기에서는 90% 이상이며 실제 BorderFill의 보수적 보존은 확인했다.
추가 한컴 source Print에서는 periodic은 95.04154%/95.02341%이나 위치·글꼴 차이가 보이고,
occluded는 **55.77974%/55.34269%**, 부모 선 누락·위치 차이가 보인다.
이 둘은 새 정식 fixture/회귀의 등록 범위에서 제외하고 원본·저장본·Print·실패 TSV를
그대로 로컬 보존한다. 높은 직접 PDF 점수나 연속 실선 편집의 통과로 이 문제를 면제하지 않는다.
B16의 source paint 앵커·내용 점유·한컴 저장 소비 계약에서 이어 처리하며 P5 PR 게이트도 남는다.

로컬 증거 root:
`/home/edward/mydev/rhwp-review-evidence/pdf-table-feedback-20261017/semantic-stage3/p4-native-borders`.
`negative-jobs.json`, `native-negative-results.json`, `wasm-negative-results.json`, `negative-review.png`.
미달 원본은 생성기로 재현 가능하며 이 파일에 해당 SHA도 남긴다.


## 등록 파일의 SHA-256

| 파일 | SHA-256 |
|---|---|
| `border-tie.hwpx` | `18f704a54650b2fe6ac0d595b1545f9ee2885a6e185a895e5896211512ad6db1` |
| `mixed-parent.pdf` | `9f233433a7f5896682934b802baa80c7b954b9447315e98ee97c8d97bdb8fbb8` |
| `print/border-tie-fixed-2020.pdf` | `8eed606d11778f5055c6a398d7bcf6137589b611f7feb124024af591b5b6a601` |
| `print/mixed-parent-border-2020.pdf` | `f4598f2ef3f0ddd2cf637f5c20e8721b72c040b4a3aa15b75a475dfdba11e189` |
| `print/mixed-parent-column-2020.pdf` | `94ac811d9ec24f05288bf7dd8514f76053f0dd48bbf190e3181573d72e784fc9` |
| `print/mixed-parent-row-2020.pdf` | `25a56d4a428b63727aa31976d1847b806a2bca14a4ddbfb38acccef2fae9aa3a` |
| `print/mixed-parent-source-2020.pdf` | `c758bc7e6c4f339210dfb27794919fc4d6c1e66979b1de44c8eb4d023844b103` |
| `print/solid-parent-border-2020.pdf` | `dc080813f79cb1cde4e6b5120ddf13f1da2a0cacf0939e66761136720dfb8fca` |
| `print/solid-parent-column-2020.pdf` | `21ceba720bf4c2d28b1ee8ae1e74a21730bf853bf76803268e2dc0a3922967af` |
| `print/solid-parent-row-2020.pdf` | `ac2895dfd66daf1bcdb3d885815e606ff905229deeec5e75b4147d6bb61d4595` |
| `print/solid-parent-source-2020.pdf` | `1ee553bbe946544419ed0edc1700640ce003508b52a09bbce52701831666c97b` |
| `print-inputs/mixed-parent-border.hwpx` | `b6328d7f13a81c7a255e03328a441e642608a10e41ceaa028236d37fa8cbd767` |
| `print-inputs/mixed-parent-column.hwpx` | `82af994213013b36747aa5871039999b2901575f3c1077969bdd8afea261ee63` |
| `print-inputs/mixed-parent-row.hwpx` | `e77fe2747d24959f31a35f97c5b25c0195f8a7923d19aef8d0842c7dcbf3451f` |
| `print-inputs/solid-parent-border.hwpx` | `a38e82df50044b31ce0f3ce512531a3f48b462a541aed27eb6d1f6923588655f` |
| `print-inputs/solid-parent-column.hwpx` | `b2bdb390771dd2063f1499eb317ac862bafd09875bec785c3c54777517332684` |
| `print-inputs/solid-parent-row.hwpx` | `475ce04bc14e6cd93fecaff129b967750e1815eb8451b221a77fc7b703ccaaaa` |
| `print-source-inputs/mixed-parent.hwpx` | `5436097736ddeae7ee3394cd8f7f11313ae939dbd16dca6db5b315065ce4da71` |
| `print-source-inputs/solid-parent.hwpx` | `06ba939d67db2ff863368712555f6ae193f81fffa87160c689c0d19839e2f604` |
| `solid-parent.pdf` | `1ec713222518a3e36bf99f5708c34f55d17cb589125f7f37396b1e2d55f51389` |

## 로컬 보류 원본 SHA-256

| 원본 | SHA-256 |
|---|---|
| `periodic-parent.pdf` | `f184ab9be12ff321465ac7604b05eaa8d352099d1765e274d6ebb4a8190c1899` |
| `occluded-parent.pdf` | `6bf14e9f180cc935e01a7b058086be0adf1a0cf6ef260ee8031a4cc0b33370c1` |
