# Stage87: 다단의 복원 불가 줄간격과 독립 영점 프레임 보존

Issue: #7445. 부모 HEAD는 `01910e50f80bc5af3829047a01f46974c6209aaf`,
base는 `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`이다.
기존 50조건 안의 KTX를 보정했으며 실제 HWP 보정 대상을 추가하지 않았다.

## 선행 해결과 원인

기존 #6970의 합성 사다리 부족 계상은 `b9af4dc237`에서 회귀와 함께 보정했다.
이슈는 현재 OPEN이며 이번 작업으로 그 이슈 전체를 닫지 않는다. 이후 이 브랜치의
`d1c97ba0df943bc8c55b55ab82944d880aadfcc5`는 다단의 뒤 문단 간격을 보존했다.
저장 줄이 있다는 사실만으로 실제 간격 복원까지 보장되지 않는 경로가 남았다.

1. `spacing_trim_restorable`은 빈 float 호스트가 현재 흐름에서 lane을 예약할 때
   저장 vpos가 간격을 복원하지 못한다고 이미 판정한다. 호출자도 원점 미확립·dirty·
   복원 불가를 `spacing_restore_unavailable`로 전달한다. 그런데 다단의 조기 반환만
   이 값을 무시해 마지막 줄간격을 버렸다. KTX 문단 29는 실제 paint에서 42.6667px를
   전진하는데 측정은 26.6667px만 전진했다. 두 형제 표의 예약 원점도 16px 앞당겨졌다.
   기존 복원 불가 판정을 다단에서도 소비하도록 고쳤다. fit 문턱은 변경하지 않았다.
2. 남은 두 운임표는 앞서 수용한 첫 완전한 원본 문단의 vpos0과 물리 단 영점이
   일치한다는 `stored_zero_column_origin` 인증을 이미 갖고 있었다. 전역 텍스트의
   page/lazy base는 아직 None이었다. 추가 `== Some(0)` 조건이 이 인증을 거절해
   측정은 문단 32의 216.32에서 진행하고 paint는 독립 저장 원점 239.72에서 진행했다.
   독립 영점 인증이 있고 충돌하는 활성 축이 없으면 같은 원점을 수용한다.
   이미 확립한 비영점 축·dirty·부분 첫 항목·합성 첫 줄은 기존 제한을 유지한다.

생산·소비 연결은 `FormattedParagraph::flow_advance_height` → 확정 float lane 원점 →
`ParagraphFloatPlacement` → 실제 table paint와, `StoredTacPage`의 영점 인증 →
`stored_tac::prepare` → `InlineBoxPlacement`의 원점/전진 끝 → 측정과 paint다.
paint에 별도 보정값을 더하지 않았고 문서·쪽 번호 분기는 없다.

## 전후 실제 출력

같은 입력과 한컴 2020 Print, 글꼴, 96DPI canonical PDFium 비교를 사용했다.
2px raw 실루엣·90%·devel 대비 2%p 기준은 변경하지 않았다.
수정 전 동결 Native와 WASM의 production diff가 부모 commit과 동일함도 확인했다.

| KTX 1/1쪽 | Native | fresh WASM |
| --- | ---: | ---: |
| 수정 전 부모 production | 81.34542% | 81.37405% |
| 줄간격만 보존한 중간 후보 | 92.64917% | 92.67446% |
| 영점 프레임도 공유한 최종 후보 | 99.36494% | 99.38975% |

중간 후보에서는 아래 운임표 차이가 보였으므로 90% 통과만으로 완료하지 않았다.
최종 Native/fresh WASM의 review·standalone overlay를 전후와 직접 열어 위 시간표와
두 운임표의 원점·셀 경계를 확인했다. 제목의 자형·굵기와 일부 지도
화살표 차이는 남는다. 이 영역을 px 답안으로 고정하지 않았다.

정상 대조군을 포함한 집중 10조건·14쪽은 모두 90% 이상이다. low-slack은
99.99852/99.99851%, 저장 줄 결측은 98.02990/98.02479%이며,
sample11 2·50쪽과 sample16 HWPX 18·35쪽도 앞 단계의 수치를 유지했다.
sample11 전151쪽과 HWPX 전64쪽의 이번 head 전쪽 검증으로 확대해 보고하지 않는다.

## 페이지 98% 조건을 충족한 px 회귀

KTX는 문서 전체가 1쪽이고 양 backend가 모두 98% 이상이다. 직접 판독에서 위치가
일치하는 세 셀 배경만 골랐다. 독립 PDF 벡터 사각형을 pt×4/3으로 변환하고,
PDF 안의 서울·용산·서울 문자열과 원본 표/셀 주소를 교차 확인했다.
후보 출력에서 기대 좌표를 추출하지 않았다. 좌·상·우·하 경계에 각각 1px를 허용한다.

기존 `issue_6970_multicolumn_synth_ladder_fill`에
`stored_multicolumn_float_and_tac_cells_match_independent_print_frames`를 추가했다.
입력 SHA, 단일 쪽, 원문 표/셀의 단일 소유와 내용, 실제 칠 사각형을 함께 검사한다.

- 수정 전 같은 최종 검사: **2 PASS / 1 FAIL**, 실제 0.043초, exit 100.
  세 셀의 상/하 경계가 약16·20·39px 앞당겨진 것을 검출했다.
- 두 생산 파일만 복원한 뒤: **3 PASS / 0 FAIL**, 실제 0.042초, exit 0.
  기존 합성 줄의 단 채움·가용 높이 검사 두 건도 보존했다.
- 두 실행 사이 기대값과 test source hash는 동일하다. 최종 source hash도 동결 빌드와
  같다. 최초 새 검사의 SHA 출력은 sha2 API의 LowerHex 미지원으로 컴파일에 실패했으나
  바이트별 hex 변환으로 고쳤다. 그 컴파일 실패는 회귀 검출 증거로 쓰지 않는다.

원본 TSV 12개와 KTX 전후 Native/WASM review·overlay 8개, 독립 좌표 및 빌드/검사 출처는
[`certified-column-flow`](../pr/assets/issue7445/completed-20261009/rebase-d9fd85ed-20261023/certified-column-flow/)에
보존했다. 긴 로그·임시 진단은 ignored output의 `stage87-multicolumn-flow-v2/`와
`other-existing-lows/source-geometry-audit/`에만 둔다.

## 잔여와 제출 상태

Native 빌드와 locked wrapper의 Mac fresh WASM `--target web --out-dir pkg --no-opt`가
성공했다. 로컬 대체 빌드이며 Docker 최적화 검증이 아니다. 루트 pkg와 Studio public의
JS/WASM hash 일치는 확인했으나, 새 browser 초기화 응답의 실제 소비 검증은 남았다.

이번 단계에서 전체 50조건·전체 nextest·필수 게이트를 완료했다고 주장하지 않는다.
Stage86 전쪽 비교의 보도자료 12쪽 미달은 새 후보의 통과 근거가 아니며 후속 보정을
계속한다. 최종 전체 검증과 실제 소비 경로 검증 전까지 PR 제출을 보류한다.
