# 저장 셀의 어절 꼬리 줄 경계

Issue: #7642.

## 원본과 독립 기준

- 입력: `samples/hwp-3.0-HWPML.hwp`
  (`1121cdabe5929d7aa68704d0516488852bcdc78b4f4b368ad054e8c121d04023`).
- 독립 기준: `pdf/hwp-3.0-HWPML-2020.pdf`
  (`809622b5bcea91f210ffe2599d87a92c4a58faee29376aa9e1aeb83c14c72b2b`).
  사용자가 원본을 수정하지 않고 한컴에서 전체 122쪽을 Print했다.
  파일명과 별개로 실제 PDF 제품 정보는 한컴 2022다. 기준을 재저장하거나
  RHWP의 출력으로 대체하지 않았다.
- 전체 범위는 Native/fresh WASM 각각 1–122쪽이며 모두 PDF와 쪽수가 같다.
- 렌더 source: `0a6ada2bd29baec40b9f8c768b574bb552e568e5`.
  판정 도구는 `78f2f6bbfdfdfdf479dbc879bf8db73e83439c87`에 고정했다.
  PDFium 153.0.7999.0, pypdfium2 5.13.0, 96dpi, Print profile,
  조판 세대 2022, 색상 경계 보정 전 2px raw 실루엣을 사용했다.

| 경로 | 전쪽 최저 raw | 93쪽 raw | 90% 미만 |
| --- | ---: | ---: | ---: |
| Native Skia | 96.68211% | 99.77806% | 0쪽 |
| fresh WASM Canvas2D | 96.49315% | 99.77438% | 0쪽 |

원본 TSV와 대표 review/overlay는
`mydocs/pr/assets/issue_7642/stored-cell-rows/`에 보존한다.
전쪽 TSV의 성공은 최종 PR 승인이나 최신 devel 대비 무회귀 판정이 아니다.
다른 코드 변경 뒤에는 해당 출력 경로를 다시 검증한다.

## 독립 기대와 실제 검사

원본 section 3, 문단 215의 TABLE 속성 설명 표에서 row 5/col 4는
`true`를 저장 줄 시작 위치 0/3으로 나눈다. 독립 Print 93쪽의 실제 표시는
`tru`/`e` 두 줄이다. 새 측정에서 한 줄로 들어간다는 이유만으로 유효한
저장 줄을 무효화하면 셀과 뒤 표·캡션이 위로 압축된다.

새 검사 `issue_7642_stored_cell_row_preservation`은 원본 저장 줄과 셀 문단
소속, 글자 순서·누락·중복, 두 줄의 셀 내부 포함, 앞뒤 줄 순서와 뒤 표의
비겹침을 확인한다. 다른 시험지 회귀 파일이나 기존 정상 기대값을 바꾸지 않는다.

해당 페이지는 양쪽 raw가 99%대이며 review/overlay를 직접 판독했다.
두 줄과 표·캡션 위치가 복원됐고, 작은 글자 테두리의 raster 차이는 남아 있다.
이 페이지의 핵심 경계만 독립 PDF의 선분 교점을 ±2px로 고정한다.
문서 전체가 98% 이상이라고 주장하지 않는다.

독립 PDF의 물리 point 좌표를 96/72배 한 셀 경계는
`left=626.543, top=286.407, right=665.403, bottom=317.732`다.
PDF 선분의 두 세로선과 해당 행 위·아래 가로선의 교점에서 읽었다.
renderer의 셀 높이 계산식이나 현재 출력 좌표를 기대값으로 재인용하지 않았다.

수정 전 `9a1e7806`의 실제 93쪽은 `true` 한 줄이고, 수정 후 `0a6ada2b`는
`tru`/`e`다. ignored stage2886의 같은 관계 진단은 수정 전 FAIL/수정 후 PASS다.
정식 Rust 검사도 stage2893에서 같은 테스트 실행 파일에 수정 전/후 CLI를
각각 연결해 확인했다. 수정 전에는 `['true'] != ['tru', 'e']`의 줄 경계
검사에서 종료 코드101, 수정 후에는 두 줄·셀 소속·포함·뒤 표 순서와
독립 Print 경계를 모두 통과해 종료 코드0이다. 빌드 실패를 재현으로
세지 않았다. 테스트 source는 `337ae04f708138267d808df5416c21330d233bdc`이며
이 commit은 위 렌더 source와 생산 코드가 같다. 최종 전체 Rust와 lint,
새 devel 대비 비교는 별도 gate로 남아 있다.

## 재현 경로

`scripts/visual_sweep.py`의 Native/fresh WASM 전체 페이지 TSV 절차를
각각 실행한다. 입력·기준 PDF·DPI·Print profile을 위와 같이 유지하고,
93쪽의 review와 standalone overlay를 함께 생성한다. 구체적 실행 명령과
artifact 해시는 ignored stage2865 producer, stage2869 backend별 receipt,
stage2874 Native 및 stage2891 WASM panel receipt에 보존했다.
