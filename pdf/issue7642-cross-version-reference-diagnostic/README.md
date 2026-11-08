# 정렬 충돌의 한컴 2024 정상 저장·Print 대조

Issue: #7642

기존 카탈로그·구조 HWP를 한컴 2024에서 정상 저장한 뒤 해당 저장본을
같은 제품의 Print로 출력했다. 실제 제품은 `13.0.0.3901`, engine `2024`,
`pdf_print_method: 0`, `frame_print_to_pdf_ex_one_up`이다.
기존 2020 HWP와 독립 Print PDF를 보존하며 기준 PDF를 교체하지 않는다.
이 진단의 통과를 원본의 통과로 보고하지 않는다. 정식 회귀 fixture가 아니다.

현재 Source269의 catalog 재저장본 18쪽 Native도 최저 46.99045%,
미달 13·14·17·18쪽으로 기존 2020 기준과 같다. 구조 재저장본 30쪽은 최저 78.07245%, 미달 1·3·4쪽으로 기존 기준과 같다.
2024 정상 저장·인쇄만으로 기존 정렬 차이가 해소되지 않음을 확인했다.
스크립트·행의 소속과 실제 표시를 대조해 원인을 계속 분석해야 한다.

| 파일 | MCP job | SHA-256 |
|---|---|---|
| `pdf/issue7642-cross-version-reference-diagnostic/catalog-resaved-hancom-2024.hwp` | `e4c01264-75c0-409a-980e-ad1c31863d7b` | `9731668ada6cc6f1519e5c313dd7f8b17ca9e67ccf71fc0b3c3b4c50f6ae65df` |
| `pdf/issue7642-cross-version-reference-diagnostic/catalog-resaved-hancom-2024.pdf` | `d595989b-d085-479e-a04b-30975489541a` | `f1d8e59cb155e4de32b1ce98ea8804a670aa8f759f27f09f9dc968db3c77b708` |
| `pdf/issue7642-cross-version-reference-diagnostic/structural-resaved-hancom-2024.hwp` | `748ad8ba-b6b3-4879-b99d-e80635fe7a23` | `87e3a6eb044b44d55f3d4283409fe7af0234634f546d74af312d910e2032925c` |
| `pdf/issue7642-cross-version-reference-diagnostic/structural-resaved-hancom-2024.pdf` | `7ab5d196-ba33-4c55-9ded-597f69321d40` | `16822e4b742153dbb31f276e4d450a7e9055a0b5b58c94032eec620c07f54734` |

MCP 상태·호출 정보·전체 TSV는 ignored
`output/pr-review/issue7642/failed-inputs-20261008/stage15-current-validation/stage271-cross-version-print/`
및 `stage277-*`에 보존한다. endpoint·token·서버 내부 경로는 기록하지 않는다.
