# Stage90: 저장 바깥 프레임의 예약·paint 일치와 회귀 보호

Issue: #7445. 부모 HEAD는 `a1c1096c38b1b29255ab35c0fd20f1ebbc7eebee`, 고정 base는
`d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다. 사용자 지정 50조건·1836쪽의
보정 범위를 확대하지 않았다. #6813 원본은 기존 기능 보존 대조군이며 새 보정 대상이 아니다.

## 예약과 실제 배치의 공통 결과

Stage88에서 앞 조각의 그림을 이유로 후속 셀의 실제 안 여백을 없애던 분기를 제거한 뒤,
기존 `issue_6712_cell_visual_row_units`의 18건 중 두 건이 3쪽/2쪽 불일치로 실패했다.
원 해결 PR #6813의 통합 `6a193a648dba3df6d5c4cffa0182bc02f3e011ff`와 보존 검사를
대조했다. 검사를 변경하거나 안 여백 제거를 복원하지 않았다.

원본 한국어 가정통신문의 셀 문단 61은 Square 중첩 표를 저장 호스트에서 1660 HU 아래에
배치한다. 실제 paint는 이미 `stored_square_table_anchor_offset`을 소비한다. 그러나
`CellUnit` 혼합 중첩 표 예약은 호스트 줄·표 높이·추정 간격을 다시 합산해 실제 점유보다
560 HU를 더 예약했다. 이어받기 끝의 빈 줄과 꼬리말 소유가 불필요한 다음 쪽으로 밀렸다.

HWP5 저장 조판의 block RowBreak 및 기존 저장 Square 그림 흐름 경로에서 같은 인증된
호스트 원점을 예약에도 사용한다. `offset + 바깥 여백을 포함한 nested_h`에서 이미 예약한
호스트 점유를 빼며, 기존 float frame이 있으면 그 공통 결과를 우선한다. 인증이 없는 경로와
다른 재조판 경로는 기존 계약을 유지한다. 문서 ID·쪽 번호·좌표 clamp나 backend별 보정은 없다.
예약은 이후 중첩 조각 높이·누적 유닛·행 수용 컷에 전달되고 실제 전체/부분 paint가 같은
저장 호스트 원점을 소비한다. 이 연결과 원본·전후 진단은 ignored Stage90 output에 보존했다.

## 실제 검증

- 기존 #6813: **16 PASS / 2 FAIL → 18 PASS / 0 FAIL**, 실행 0.165→0.161초. 검사 변경 없음.
- 보도자료 셀 안 여백·후속 표: **4 PASS / 0 FAIL**, 실행 0.174초.
- 집중 Native/fresh WASM 12조건·38쪽: 미달·쪽수 불일치 0. 보도자료 전12쪽 최저
  94.23037/94.18582%, 마지막 12쪽 99.25420/99.25121%로 Stage88 개선을 유지했다.
- #6813 한국어 원본 전2쪽도 양 backend 90% 이상·쪽수 2/2다. 2쪽은 고정 devel의
  95.65821/95.67097%에서 93.68608/93.89238%로 변해 하락폭 1.97213/1.77859%p다.
  2%p 미만이지만 동일 화면이라고 하지 않는다. 표·내용·종료 소유는 보존되며 미세 위치와
  래스터 차이가 남는다. 이 2쪽에 절대 px 검사를 추가하지 않았다.
- 현재 생산 코드의 생물독소 전205쪽은 Native/fresh WASM 모두 205/205, 최저
  **98.42761/98.42748%**, 90% 미만 0쪽이다. 11쪽 표와 80쪽 그림·캡션·뒤 본문을 양쪽
  review/overlay에서 직접 판독했다. 원본과 독립 한컴 2020 Print PDF는 변경하지 않았다.

## Stage85 경계를 검출하는 정식 검사

기존 `tac_picture_stored_rewind_next_page`의 세 검사는 유지하고 같은 원본에서 두 검사를 보강했다.
원본 SHA는 `18765ac22270a2cb4ece6a1e6539c2dea6ca6ccde2573b42fb9acce7c34fa83a`,
PDF SHA는 `fe01f1ddffc3153d015adab93178f0fa8a5e9624763ff529362d4410d50955dc`다.

1. 작은 그림 37도 저장 쪽 경계를 지키고 앞 쪽에 중복되지 않으며 같은 쪽의 캡션 앞에 배치된다.
   그림 원문 소유와 BinData를 확인한다. 해당 80쪽 양 backend raw 99.99% 이상과 직접 판독 후
   독립 PDF Image XObject의 네 경계를 96 CSS px·±2px로 보호했다.
2. 잉크 없는 유효 저장 호스트도 표를 한 번 배치하고 바깥 위 여백을 보존한다. 11쪽 양 backend
   raw 100%와 직접 판독 후 독립 PDF의 실제 상단 괘선 중심 134.413330078125 CSS px를 ±2px로
   검사한다. 잉크 경계를 글줄 상단으로 오인하지 않고 실제 Line paint와 대조한다.

테스트를 그대로 둔 채 Stage85의 두 원인 분기만 이전 계약으로 되돌려 **3 PASS / 2 FAIL**
(실행 0.638초, exit100)을 확인했다. 작은 그림의 쪽 소속과 표 괘선 130.78667px가 각각 실패했다.
소스를 바이트·해시까지 복원한 뒤 **5 PASS / 0 FAIL**(실행 0.642초, exit0)이다.
초기 테스트 작성의 필드·해시 포맷 컴파일 오류와 suite 재준비 전 0건 실행은 실패 재현으로 세지 않았다.

[전체 TSV·독립 PDF 좌표·빌드 출처·전후 검사·대표 review/overlay](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/stored-outer-frame-preservation/validation.json)를 연결한다.
Native SHA `cfe9343574bfb310ccdd03a34c47dcc59c278193da7facc1d15f6ae95cd221f0`,
fresh WASM SHA `2f2493533b96dbe603bbfa9e7a410d1db1c988d5ee70f033ad13ec8bafd9c72f`다.
WASM은 locked wrapper의 Mac `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
루트 pkg·Studio public hash는 같으며 Docker 최적화 통과를 주장하지 않는다.

## 최종 검증과 구분

이 단계는 집중 보정·전체205쪽 선행 검증·정식 회귀 검출 증거다. 현재 코드의 50조건 전체
Native/fresh WASM, 전체 nextest·필수 lint/build/Skia 및 실제 소비 경로36건의 최종 검증은
다음 단계에서 완료해야 한다. 이전 Stage83/84 전체 통과나 중단한 Stage89를 새 코드의 전체
통과로 재사용하지 않는다. 지침·원본·PDF·기존 검사 기대값과 사용자 초안은 변경하지 않았다.
