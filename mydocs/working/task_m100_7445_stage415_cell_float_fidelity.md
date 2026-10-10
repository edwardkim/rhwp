# 단계 415: 저장 그림 픽스처의 두 쪽 물리 프레임 보정

## 사전 분석과 독립 근거

대상은 [cell_float_stack_stored_vpos.hwpx](../../samples/issue5734/cell_float_stack_stored_vpos.hwpx)
2쪽이며 기준은 기존 [독립 한컴 Print PDF](../../pdf/cell_float_stack_stored_vpos-2020.pdf)다.
원본 HWPX와 PDF는 변경하지 않았다. 이 픽스처의 그림은 1×1 흰색 대체 데이터이므로 표 선과
제목의 시각 일치만으로 그림 소속을 판단하지 않았다. Print PDF의 그림 14개를 별도로 확인했다.
첫 쪽에는 10개, 다음 쪽에는 4개가 있으며 병합 칸의 마지막 문단 그림 4개 중 첫 그림만 첫 쪽에 있다.
실제 원문 전체의 그림 내용 피델리티 개선을 주장하지 않는다.

수정 전 후보는 3쪽이었으며 작은 병합 블록 분류를 복구한 뒤에도 두 쪽 비교가
50.97998% / 64.74895%였다. 고정 devel `21ed64071`도 71.11990% / 71.91953%로 미달했다.
종전 픽셀 검사는 일부 그림을 좌표와 크기로 선택해 통과했지만 독립 Print의 그림별 쪽 소속을
검사하지 못했다. 닫힌 #5734의 종전 겹침 이슈를 새 미해결 이슈 등록으로 간주하여 검사를 삭제하지 않았다.
사용자가 이 픽스처 자체를 개선하도록 지시한 범위다.

## 원인과 코드 수정

1. 표 진입은 내부 저장 경계가 없는 작은 병합 블록을 보호했지만 행 스캔은 같은 RowBreak 블록을
   행 단위로 풀었다. 같은 블록·셀 순서의 시작/끝 컷을 이어받도록 분류를 일치시켰다.
2. 저장 그림 뒤의 빈 문단 위치와 그림 높이를 중복 계상하고, 여러 그림이 있는 문단을 한 유닛으로
   묶어 다음 쪽으로 보냈다. 원본 문단·그림별 위치와 점유 끝을 공통 그림 흐름으로 만들고
   HeightMeasurer의 일반/rowspan 측정, 컷 유닛, 실제 그림 배치가 이를 소비하도록 했다.
3. `flowWithText=true`, `allowOverlap=false`인 그림들의 순서를 보존하고 셀 전체 세로 정렬을
   선택 그림 밴드에 한 번 적용했다. Top 정렬의 원본 음수 오프셋은 보존한다. 내용이 전부 셀 위에
   있는 비정상 음수 앵커, 편집·합성 줄, 텍스트·다른 개체·겹침 허용 그림에는 새 인증을 적용하지 않는다.
4. 그림 병합 칸의 일부가 남아도 완전히 소비한 형제 행의 물리 빈 공간은 보존한다.
   `row_block_cut_uses_measured_height`를 예약·행 배치·배치 후 선 처리에서 함께 확인한다.
   1쪽 마지막 행에서 줄간격 8.8px를 다시 빼던 하단 선 축소를 제거했다.
5. 오프셋 컷의 내용 높이 358.1px만 예약하고 물리 행 합 398.1px를 그리던 차이를 수정했다.
   `SelectedBlockCut::stored_picture_frame_height`의 요구 높이를 수용 전에 확인하고 같은 높이를
   누적 예약한다. 부족하면 앞 행을 보존한 채 블록을 다음 쪽으로 이월한다. 첫 행에서 더 이월할 수
   없는 경우는 기존 원자 블록 진행 경로를 유지한다. 문서명·픽셀 맞춤·공차 확대는 추가하지 않았다.

생산/소비 경로는 `StoredPictureCellFlow → HeightMeasurer 및 cell_units → RowBlockQuery /
SelectedBlockCut → scan_rowspan_block_step의 예산/컷/누적 예약 → layout_partial_table의 행 높이 →
layout_partial_table_cells의 그림 원점 → fold_last_row의 선 처리`다. 컷 범위 밖의 완전 행과
편집 후 재조판은 기존 경로를 사용한다. 다른 개체 조합의 예약 정책을 전면 승인한 것은 아니다.

## 시각 결과와 경계 검증

| 출력 | 전체 쪽수 / PDF | 1쪽 | 2쪽 | 90% 미달 |
| --- | --- | ---: | ---: | ---: |
| 최종 Native | 2 / 2 | 100.00000% | 100.00000% | 0 |
| 새 Mac no-opt WASM / 실제 Chrome | 2 / 2 | 100.00000% | 100.00000% | 0 |

이 값은 2px 이웃 관용 실루엣 일치율이며 RGB 픽셀 동일성을 뜻하지 않는다.
두 쪽 리뷰 PNG를 직접 확인했고 1쪽 하단 선이 겹친다. 색·획의 엄격 픽셀 값은 별도이며
100%를 이유로 좌표 핀 검사를 허용하는 지침 변경은 사용자가 취소했으므로 적용하지 않았다.
원본 14개 그림의 누락·중복과 첫/다음 쪽 소속은 관계형 검사로 확인한다.

Native의 전체 2쪽 layout-anomaly는 본문 넘침·용지 밖·텍스트 겹침·표 겹침·빈 쪽이 모두 0건이다.
진단용 복사본의 용지 높이만 2400HU 줄인 경계에서는 내용 358.1px가 예산 370.8px에 들어가지만
물리 행 합 398.1px는 들어가지 않는다. 이전 컷 수용의 본문 넘침 1건이 수정 후 0건으로 줄었으며
2쪽을 유지했다. 3600HU 축소 대조군도 2쪽·넘침 0건이다. 진단 입력은 ignored output에만 두었다.

기존 `issue_5734_left_cell_floats_stack_by_stored_vpos` 함수에서 고정 x/y·크기 핀을 제거하고
원본 BinData 소속, 모든 그림 한 번 표시, 셀 내부 포함, 같은 쪽 순서·비겹침, 완전히 소비한 두 행의
선언 높이 비 보존을 검사한다. 같은 함수에서 원본 선언 빈 공간으로 본문을 줄인 경계와 이월 후
모든 그림 보존·표의 본문 내부 포함도 검사한다. 검사 함수나 시험지 회귀를 추가하지 않았다.

정식 locked release-test focused nextest를 threads=8 / no-fail-fast로 실행해 **35 PASS / 0 FAIL / 1045 filtered**를 확인했다. 대상의 기존 함수와 #5729·#2220·#7333·#6782·#7048 대조군을 포함한다. 이 단계의 2쪽 검증을 전체 브랜치의
검증 완료로 확대하지 않는다. 이전 차단 재실행은 53건 중 27 PASS / 26 FAIL였으며 아직
최신 전체 nextest·27건 전체 영향 문서·lint/Skia/Studio 게이트와 PR 생성·CI·병합은 완료하지 않았다.

## 증거

- [Native 전쪽 TSV](../pr/assets/issue7445/completed-20261009/cell-float-fidelity/native-silhouette.tsv)
- [fresh WASM 전쪽 TSV](../pr/assets/issue7445/completed-20261009/cell-float-fidelity/fresh-silhouette.tsv)
- [1쪽 리뷰](../pr/assets/issue7445/completed-20261009/cell-float-fidelity/native-review_001.png)
- [2쪽 리뷰](../pr/assets/issue7445/completed-20261009/cell-float-fidelity/native-review_002.png)
- [소스·입력·빌드·WASM 해시와 검증 결과](../pr/assets/issue7445/completed-20261009/cell-float-fidelity/validation.json)

전체 로그는 ignored `output/pr-review/7445-completed-20261009/stage415-cell-float-physical-frame/`에
보관하며 커밋하지 않는다. 입력/PDF 정본은 그대로 보존한다.
