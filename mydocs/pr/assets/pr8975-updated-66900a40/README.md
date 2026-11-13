# #8975 갱신 head 66900a40be — 독립 검증 원장

최신 원 head 66900a40be의 필수 로컬 검증을 완료했다. 새 제출 후보의 GitHub CI·셀프 리뷰·merge는 별도로 확인한다.

## 추가분과 입력 구분

원 #8951과 #8975의 구 head는 수용 PR #9047, merge `a785e23f6838d5452e8904cae82f81eb5e185efc`에 반영됐다.
그러나 #8975는 승인 대기 중 `66900a40bec0a45d5dd3124538d824c483d631f0`로 바뀌었다. 두 기존 commit이 rebase됐고,
`fdbf9a80d1`의 셀 글줄 내용 순서·소유·포함·반례 검사, `7998010c40`의 입력 표지와 `66900a40be`의 증거가 추가됐다.

표지 입력의 원래 9개 OLE 스트림은 구 입력과 SHA-256이 모두 같다. 추가된 `/RhwpLineSegTags`는 40바이트다.
#8939의 합성 줄 표지를 제대로 읽는 입력은 구형 표지 없는 입력과 구분한다. 구 입력을 버려 #9047의 실제
간격 회귀가 사라지지 않게 `legacy-unmarked/`에 원래 bytes를 보존하고 기존 두 검사가 양쪽을 실행하게 했다.

추가로 구 입력을 실제 Windows Hancom 2020 `11.0.0.9136`에서 HWP로 저장했다. `BodyText/Section0`, `DocInfo` 등
6개 스트림이 바뀌었고 합성 표지가 없는 실제 한컴 저장본이다. 이 정상 대조군도 같은 한컴의 Print(method 0,
`hancom2020_pdf_driver_one_up`)로 전쪽 출력했다. rhwp 생성 입력을 한컴 저장본으로 표현하지 않는다.

## 전쪽 실제 결과

제품 source는 #9047과 같은 `e5a6a1ff9f`이며 현재 base `a785e23f68`의 생산 파일과 같다.
앞선 devel `180a4ec972`의 동일 producer closure와 비교했다. 96dpi·compat 2022,
pypdfium2 5.13.0, 실제 Native PNG·fresh 최적화 WASM Canvas이며 SVG 래스터를 대체하지 않았다.

| 입력 종류 | 원본별 쪽수 | 보정 전 Native 최저(e/f) | 보정 전 WASM 최저(e/f) | 현재 양쪽 전쪽 최저 |
| --- | --- | --- | --- | --- |
| 현행 표지 입력 | 각각 2 | 100.00 / 100.00% | 100.00 / 100.00% | 모두 100.00% |
| 구형 표지 없는 입력 | 각각 2 | 94.62 / 93.46% | 94.90 / 93.79% | 모두 100.00% |
| 한컴 실제 재저장 대조군 | 각각 2 | 100.00 / 100.00% | 100.00 / 100.00% | 모두 100.00% |

Native/fresh WASM 각각 12쪽이다. [보정 전후 48행](comparison.tsv)과 `tsv/`의 원본을 보존했다.
표지 입력은 보정 전에도 정상이므로 그 통과를 간격 보정의 개선으로 세지 않는다. #9047의 보정은
구형 파일에서 이미 소비한 빈 줄 간격을 다시 더하지 않는 호환 개선이며, 파서의 bit 31/합성 표지 판정을 바꾸지 않는다.
새 head는 제품 코드를 추가로 바꾸지 않는다.

세 종류 입력의 현재 Native/WASM RGB 래스터는 backend·문서·쪽마다 구 입력의 정상 출력과 모두 같다.
독립 Print 래스터도 원본별 전 2쪽이 구 입력 Print와 모두 같다. 보존한 대표 PNG를 새로 열어 행의 쪽 소속,
셀 내용 순서·포함, 표 괘선·빈 줄·뒤 본문을 직접 대조했다. 수식은 없으며 100%를 RGB 완전 동일과 혼동하지 않는다.
기여자의 웹 기준 자료는 이력으로 보존하고 현재 판정은 Desktop Print에 한정한다.

## 입력과 기준 PDF 신원

모든 HWP/PDF는 이 후보에 포함하며 실제 실행 bytes와 SHA-256을 대조했다.

| HWP(접두사 samples/issue7620/) | HWP SHA-256 | 독립 Print | PDF SHA-256 |
| --- | --- | --- | --- |
| `e-cellbreak-row-moves-next-page.hwp` | `48b74c6a93f376dec94cbbd9a0f749780566dff85128f341ffe1d5939634e6e1` | `pdf/issue7620/e-cellbreak-row-moves-next-page-tagged-2020.pdf` | `a06129e96004c6bd58591a1cace0cf0f5c56234168db2bd154f79decd0db77c5` |
| `f-rowbreak-row-splits-at-line.hwp` | `627aa762cb53ed673e4e941e46454b717a59683f4daeee9b3189edbc294c8059` | `pdf/issue7620/f-rowbreak-row-splits-at-line-tagged-2020.pdf` | `9a7344d52ff0b7d565e3e206f522f75f05f1d84aeca5b78711cfb2d8448d9fa8` |
| `legacy-unmarked/e-cellbreak-row-moves-next-page.hwp` | `e566e14f5a0b4a4b8dbebefc6ab3d233710030ab7f53ee1d7307442ddf54f141` | `pdf/issue7620/e-cellbreak-row-moves-next-page-2020.pdf` | `a144223cc49c6be43027a5b95e969867dd6ff455115030351271d091183a28c1` |
| `legacy-unmarked/f-rowbreak-row-splits-at-line.hwp` | `904520801739be5795f0dd64dc907367a15f131ab4b4a985439724fed0936e4b` | `pdf/issue7620/f-rowbreak-row-splits-at-line-2020.pdf` | `a6f8ffe6c730ac30812dc3a52f8aa6a1585d92e0f118231705c2574624e397b9` |
| `hancom2020-saved/e-cellbreak-row-moves-next-page.hwp` | `635636753ea0193f23867aa6265617563b453f8bee4f44232f8f216fdb8676c5` | `pdf/issue7620/e-cellbreak-row-moves-next-page-hancom-saved-2020.pdf` | `b2e203ae66ccf534ddcc844c4076324f8357f459b81c72915e72c505f7e45e0b` |
| `hancom2020-saved/f-rowbreak-row-splits-at-line.hwp` | `ba916320d36099c4854087eb2ff8bd65a986a2788eed7fb421f9469521e9dfec` | `pdf/issue7620/f-rowbreak-row-splits-at-line-hancom-saved-2020.pdf` | `bd35bbddd6d17f7e00aa47b66931f20184aa8a479e2376e91c9251f69efbffa9` |

Native binary SHA-256은 `cc7dbf797446b9ff58fb2305001445fa8a2de00537688579491550e07002934a`,
WASM은 `4b89b130aa5ff17899da91559ff97e20f6f2e9c4be7894b88d290a62875e0462`다.
동일 제품·빌드·보호 입력·PDF·환경의 정본 보호 3,542쪽은 [#9047의 전쪽 결과](../sacru2red-20261113/final/protection/comparison.tsv)를 재사용한다.
보호 18개 데이터셋에 이번 입력은 없으며 source/producer/해당 입력은 바뀌지 않았다. 새 재캡처로 표시하지 않는다.

## 자동 회귀와 진행 상태

기여자의 라벨 기반 내용 순서, 라벨 행·열과 실제 소유 셀, 글줄의 셀 내부 포함, 셀의 표 조각 포함,
같은 셀 글줄 비겹침과 양 열 정렬을 보존했다. 실제 배치의 인접 라벨 역전·표 밖 이동·다른 행 소유 반례를
거부하는 검사도 수용했다. 기존 빈 줄의 실제 전진(원본 1600HU)과 내용 단일 소유 검사도 유지했다.
좌표 핀·허용치 완화는 없으며 기존 test 두 개가 구형/표지 입력을 각각 실행한다.

최소 글꼴 집중 9 PASS, 새/교체 샘플 6개를 전달한 보안 검사와 IR·셀 초과·off-canvas·text overlap·본문 초과·독립 PDF 쪽수 래칫 82 PASS다. fmt·Native/WASM Clippy·workspace build·workspace all-target Clippy는 모두 exit 0이며, 최신 devel 1003349996 기준 suite manifest 검사도 통과했다. 시스템/사용자 글꼴과 네트워크를 격리하고 저장소의 공개 글꼴 5개만 노출했다. oracle TSV에는 기존 행과 주석을 보존하고 새 입력의 독립 쪽수 4행만 추가했다.


## 독립 한컴 실행 식별자

Windows Hancom 2020 `11.0.0.9136`의 실제 완료·다운로드와 입력/출력 SHA를 확인했다.
인증값·endpoint·원시 응답은 포함하지 않는다.

| 입력 | 작업 | job ID |
| --- | --- | --- |
| 표지 e | Print | `ca38d686-b2ce-46d7-bd71-3f36b80be526` |
| 표지 f | Print | `b40ce807-2fd3-4588-a7c3-94aac0b00393` |
| 구형 e | 실제 HWP SaveAs | `6718fe3d-0c6d-43a5-a65c-1613f6ca0945` |
| 구형 f | 실제 HWP SaveAs | `663a80fa-c8fb-4574-9ee0-9743b749b84f` |
| 실제 한컴 재저장 e | Print | `91dd628f-2735-410b-992e-8723e5c7eeb4` |
| 실제 한컴 재저장 f | Print | `e3af86ac-6fb2-4d7e-9f78-aa20ab27180d` |

SVG snapshot 4건도 최소 글꼴 환경에서 통과했다. 프로세스 내 반복 출력 결정성 검사를 포함하며 golden을 다시 생성하지 않았다.
