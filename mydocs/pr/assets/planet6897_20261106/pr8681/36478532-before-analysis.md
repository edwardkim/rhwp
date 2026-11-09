# 결재문서 36478532의 최초 Native 측정과 보정 분석

## 실제 확인한 출력

- source `46aca162bcf7fbcbc114cc42a251e6047598b371`, Native Skia·Print profile·compat 2022·96 DPI.
- 실제 측정 엔진 SHA-256 `f8d436f6bb1ac4aabdf9ce07da11563770ccd0205642a0531ca2689ca7848115`. focused 검사가 빌드한 엔진을 별도 보존해 사용했다. 이후 별도 CLI build의 다른 SHA로 재표기하지 않는다.
- [원본·독립 Print와 SHA](print-reference.md). 전체 쪽수 Native 2 = PDF 2.
- [전쪽 Native TSV](36478532-before-native.tsv): 1쪽 62.43%, 2쪽 100.00%. overlay의 반올림 전 raw는 62.43287%다.
- [같은 1쪽 review](36478532-before-native-review.png)와 [standalone overlay](36478532-before-native-overlay.png)를 직접 열어 확인했다.
- 「다. 지출방법」과 점검사항 두 행이 Native에서 잘린다. 위 표의 아래 경계와 뒤 지출결의서 표의 원점도 PDF와 다르다. 글꼴만의 차이로 분류하지 않는다.

## 원본과 공통 호출 경로

원본 XML을 엔진 모델과 별도로 읽었다. 본문 칸 문단 4의 저장 줄 vpos는 6000HU, 자리차지 표의 문단 기준 세로 오프셋은 5840HU, 선언 높이는 5159HU다. 문단 5의 두 줄은 7500/9000HU, 문단 6은 10500HU, 문단 7은 17279HU에 저장돼 있다. 원본을 수정하거나 내용을 삭제하지 않았다.

| 단계 | 현재 소비 위치 | 추가 확인할 내용 |
| --- | --- | --- |
| 원점 인증 | `stored_nested_float_placement` → `ParagraphFloatPlacement`의 저장 head/tail 호스트 생산 | 실제 반례에서 선택한 frame과 재조판 무효화 상태 |
| 측정 | `HeightMeasurer::cell_nested_controls_bottom_with_storage` → `nested_group_occupied_height` | 선언 높이와 오프셋 위 공간·실제 점유 하단의 계상 |
| 전체 셀 배치 | `table_layout.rs`의 stored float frame 원점 소비 뒤 흐름 전진 | `NextLine`만 `paragraph_end`를 쓰고 다른 frame은 `nested_y + advance`를 쓰는 경로 |
| 부분 셀·유닛 예약 | `table_partial.rs`, `cell_units_uncached` | 같은 frame·내용 소유·clip과 후속 본문 보존 |

현재는 원본·독립 Print·실제 출력으로 결함을 재현했고 소비 경로를 좁힌 단계다. 런타임 frame과 요구/예약/실제 높이를 확인한 뒤 공통 결과에서 보정한다. 코드 수정·fresh WASM 검증·회귀 추가는 아직 완료하지 않았다. 이 자료를 통과 증적으로 사용하지 않는다.

## 메인터너 보정과 남은 원인 확인

2026-11-06 Native 보정 중간 결과는 1쪽 62.43% → 67.09% → 69.31%, 2쪽은
100.00%다. 중첩 배제 표의 원점·점유 끝을 측정과 배치에서 공유하고, 뒤의 빈 저장 줄도
물리 공간에 포함해 본문·점검사항 잘림과 위 표 하단을 복구했다. 미달 상태를 완료로 판정하지 않는다.

남은 지출결의서 원점 차이는 별도 원인이다. 원본 HWPX의 최상위 문단 4는 LineSeg가 없으며
글앞 부동 표(control 0)와 TAC 표(control 1)를 같은 문단에 담고 있다. 현재 로드는 첫 앵커를
줄 구성에서 빼고 TAC 한 줄만 생성해 뒤 표를 위로 당긴다.

한컴 2020 MCP로 동일 원본을 별도 HWP에 저장한 **진단본**을 확인했다.
job `582572af-9412-476f-acbd-55787336f0c2`, 실제 한컴 `11.0.0.9136`,
진단본 SHA-256 `1b4948599b996226277ae6b9c7c1621a70d8e409a1cf828e2a38effa8585458d`다.
이는 HWP SaveAs 진단이며 Print가 아니다. 원본·기준 Print PDF를 대체하지 않는다.

| 관측 | 현재 로드 생성 | 한컴 진단 저장본 |
| --- | ---: | ---: |
| 문단 4의 줄 수 | 1 | 2 |
| 첫 앵커 줄의 원시 textpos / vpos / textheight(HU) | 0 / 36356 / 25981 | 0 / 36356 / 1100 |
| TAC 소유 줄의 원시 textpos / vpos(HU) | 0 / 36356 | 8 / 37456 |
| 다음 문단 5의 vpos(HU) | 62337 | 63437(원본 XML과 일치) |

표의 바깥 폭 51299HU는 가용 줄 폭 51024HU를 넘는다. 첫 앵커도 원시 줄의 내용이므로
뒤 TAC가 같은 줄에 들어가지 않으면 앵커의 정상 글자 줄을 남겨야 한다. 기존 누름틀 앵커의
같은 폭 판정을 공통 빈 문단 재조판에서 글앞·글뒤 개체 앵커에도 적용하는 보정을 진행한다.
개체 본체를 인라인 폭으로 더하거나 임의 px 이동으로 맞추지 않는다.

진단본은 `output/pr-review/planet6897-integration-20261106/hancom-normalization-diagnostic/`에
보관했다. 위 69.31%는 이 추가 원점 보정 **이전** 결과다. 새 빌드의 Native/fresh WASM과
원래 Print PDF를 다시 비교하고 실제 개선을 확인한 뒤 회귀·최종 리뷰를 갱신한다.

## 보정 후 실제 Native 결과

추가 원점 보정까지 적용한 producer SHA-256은
`901455bd1f38098eb574d2fdefe5830f21f06a673b95197ac7e408a5be80d369`다.
Print profile·compat 2022·96 DPI에서 [전체 TSV](36478532-maintainer-native.tsv)는
**1쪽 99.90%, 2쪽 100.00%**, 쪽수 2=2로 통과했다.
[같은 1쪽 review](36478532-maintainer-native-review.png)와
[overlay](36478532-maintainer-native-overlay.png)를 직접 열어 본문·점검사항·위 표의 경계와
뒤 지출결의서 원점의 복구를 확인했다. 앞서 기록한 69.31%는 중간 단계다.

같은 엔진에서 주 입력 stored-band는 Native 2쪽 모두 100.00%, nars-2021은
3쪽 최저 96.50%, 각주 캡션 대조군은 1쪽 100.00%다. fresh WASM과 최종 Cargo 회귀·lint는
아직 완료하지 않았다. 전체 통합 검증의 다른 보고서 미달과 대조군 하락도 별도로 보정하며,
이 Native 결과만으로 전체 PR 수용을 선언하지 않는다.
