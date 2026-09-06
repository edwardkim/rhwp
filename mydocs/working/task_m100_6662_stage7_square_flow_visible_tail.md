---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage7_square_flow_visible_tail.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 7단계: 어울림 셀의 본문 소실

Issue: #6712. 시작 HEAD: `7043cc636`.

## 재현

직전 전체 회귀는 9074건 중 4건 실패했다. 한국어 가정통신문의 예방수칙 4줄이
첫 페이지 y=1134.0/1155.4/1176.7/1198.0에 배치되어 용지 높이 1122.5px를 넘었다.
중국어본도 2줄이 넘었다. overflow-cell 및 off-canvas 각 2건 실패다.
기준 한컴 PDF에는 이 문장이 모두 1쪽에 있다. 2쪽이라는 쪽수 일치만으로는 통과가 아니다.

## 분석 및 실행 계획

1. 셀 분할 unit에서 제거한 nested-table 뒤 빈 어울림 문단 높이가 렌더 cursor에
   다시 더해지는지 확인한다. 동일 문단 판정은 공통 함수로 통일한다.
2. 실제 문서 계약에 전 페이지 overflow=0과 대표 꼬리 문장의 가시성 검사를 추가한다.
3. 집중 계약, overflow/off-canvas 래칫, 한컴 PDF visual sweep을 재실행한다.
4. 기준선 면제 없이 수정·분석 결과를 별도 커밋한다. 다음 이슈는 다음 단계에서 진행한다.

임시 로그/SVG/JSON은 `/tmp`에만 둔다. 최종 코멘트에서 사용하지 않는 PNG도 커밋하지 않는다.

## 수정과 중간 검증

- `table_partial.rs`는 실제 두 가정통신문의 분할 셀 렌더 경로다. 일반 셀에만 적용한
  어울림 anchor와 그림의 flow 중복 제외가 여기에는 없었다. 두 경로가 같은 함수를 사용하게 했다.
- nested table 뒤 저장된 빈 어울림 띠 판정을 공통화했다. RowBreak + 실제 Square 인접 글줄,
  연속 빈 문단 2개 이상, 앞쪽 중첩 표, 실제 저장 LINE_SEG, 양수 가로 시작값을 요구한다.
  명시적 쪽 나눔과 합성 LINE_SEG는 제외한다.
- 첫 번째 focused red는 실물 2건 실패/합성 6건 통과였다. 일반 셀만 수정한 중간 후보도
  실물 2건에 실패했다. 분할 경로까지 연결한 후보는 8/8 passed(0.061초)였다.
- `cargo build --locked --profile release-test --target-dir target/pr-review --bin rhwp`:
  exit 0, 2분 36초. 이 바이너리로 한국어·중국어 각 1~2쪽을 다시 렌더했다.
- 두 문서의 fidelity text-only 전수 및 visual sweep은 모두 exit 0이다. 한국어 text delta는
  1·2쪽 모두 0이다. 중국어는 1쪽 delta 0, 2쪽 footer 16자 누락이 남는다.
- 4개의 review PNG를 직접 판독했다. 한국어 예방수칙 4줄은 1쪽에 복구됐지만, 중국어
  예방수칙 일부 글줄의 겹침과 2쪽 그림/제목 정렬, footer 누락은 다음 단계 수정 대상이다.
  따라서 이 단계의 통과는 본문 overflow 개선에 한정하며 #6712 해결 완료가 아니다.
- 사용자 확인(2026-09-06): 지정한 한국어 `-2020.pdf`와 중국어 `-2024.pdf` 두 문서가
  검증 대상이다. 사내 ID `18981443` 대응 미확정을 별도 대기 사유로 두지 않는다.
- overflow-cell 16분할, off-canvas 16분할, #5862 clip 보호 3건은 모두 통과했다.
  첫 확대 실행은 44건 중 43통과/1실패(65.844초): 새 가시성 검사가 2쪽에도 실제 존재하는
  공통 손씻기 문구를 잘못 중복으로 판정했다. 2쪽 부재 조건을 1쪽 고유 눈병 문구로
  한정한 재실행은 #6712 9/9 passed(0.171초, exit 0)다. 실패를 통과로 합산하지 않는다.
- 원래 실패한 한국어·중국어의 overflow-cell/off-canvas 4건을 포함해 baseline 파일은
  수정하거나 면제하지 않았다. 모든 코어 회귀와 push 전 lint는 최종 후보에서 추가로 수행한다.

## 다음 단계 경계

저장된 문단/줄의 세로 좌표, 같은 문단 안 Square 어울림, continuation의 그림 anchor,
마지막 셀 footer 소유를 별도 분석한다. 현재 단계의 중간 비교 PNG는 최종 코멘트 자료가
아니므로 커밋하지 않는다. 최종 비교가 준비된 뒤 필요한 before/after만 보존한다.
