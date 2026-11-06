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
