---
kind: investigation
status: completed
canonical: mydocs/working/task_m100_6662_stage10_square_following_paragraph.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 10단계: 그림 뒤 문단의 어울림 연결

Issue: #6712. 시작 HEAD: `48fe28f25`.

## 분석

- 한국어 2쪽의 셀 문단 41에는 제목과 Square Picture가 함께 있다.
  저장 좌표는 그림 오른쪽 5297 HU와 글줄 시작 5297 HU가 일치한다.
- 다음 문단 42도 같은 column_start이며 그림의 세로 영역과 겹친다. 그러나
  `stored_square_picture_wrap_anchor_for_control`은 글자가 있는 anchor의 경우
  같은 문단만 연결해서 문단 42의 어울림 배치가 빠진다.
- 문단 43은 그림의 세로 영역 밖이며 column_start=0이다. 무조건 후속 문단을
  옆으로 밀면 안 된다. 합성 LINE_SEG와 그림 위에 겹쳐 쓰는 줄도 어울림 증거가 아니다.

## 계획

1. 실물 문단 41/42의 가로 시작 일치, 43의 원위치 복귀, 합성/비인접 음성 계약을 추가한다.
2. 실제 저장 글줄이 그림 옆에 있다는 공통 판정을 후속 문단에도 적용한다.
   저장 vpos가 되감기거나 그림의 세로 영역을 벗어나면 연결을 종료한다.
3. focused, overlap/overflow/off-canvas와 기준 PDF 대조 후 결과를 별도 커밋한다.

빈 그림 anchor의 세로 전진과 footer 묶음 그림은 다음 단계다. 기존 PDF 두 개를 재사용하며,
중간 로그/PNG/SVG/JSON은 `/tmp`에만 두고 커밋하지 않는다.

## 수정 전 계약

- 신규 focused 2건: 1 통과, 1 실패, 0.073초, exit 100.
- 한국어 2쪽의 제목 시작 x=157.5467px, 다음 소제목 x=86.92px로
  저장 어울림 폭 5297/75=70.6267px가 빠졌다. 합성/비인접 줄의 음성 계약은 통과했다.

## 보정 및 검증 결과

- 글자 유무로 anchor 문단과 후속 문단을 갈라 처리하지 않고, 실제 저장 줄과 그림의
  인접 관계로 공통 연결한다. vpos 역전·그림 하단 이후·유효한 저장 줄 부재에서 종료한다.
- 82 tests passed, 2 slow, 997 filtered/skipped, 87.283초, exit 0.
  새 양성/음성 계약과 #6712, #5862/#5863/#2007, 전체 16분할
  text-overlap/overflow-cell/off-canvas를 포함한다. 기준선을 수정하지 않았다.
- 최신 CLI 별도 빌드: exit 0, 2분 27초. 바이너리 SHA-256:
  `0129fde45764bc6f5e2cba5a13da8a140e55a41f6362e1003b29743b651dd61b`.
- 사용자 지정 PDF 두 개로 한국어·중국어 1~2쪽 visual sweep과 좌표 대조를 재실행했다.
  각 첫 sweep은 Chrome exit 0 후 PNG 확인 경쟁으로 exit 1, 같은 입력의 resume은
  exit 0이었다. 실패 실행을 성공으로 합산하지 않는다. 비교 PNG 네 장을 직접 확인했다.
- 한국어 2쪽 소제목이 제목과 같은 저장 가로 시작점으로 이동해 그림과의 겹침이 없어졌다.
  그림 세로 위치 차는 0px를 유지한다. 두 문서 모두 한컴/rhwp 2쪽이다.
- 한국어 1쪽 본문 약 -21.9px, 중국어 1쪽 약 -25.6px의 잔여 전진량 차이는 여전하다.
  한국어 footer 로고/테두리 누락, 중국어 footer 문구와 아래 테두리의 교차도 남는다.
- #6712 전체 해결 완료가 아니다. 이번 단계는 후속 문단의 가로 어울림 복원이다.
  중간 비교 자료와 로그는 `/tmp`에만 두었다. PR/push/이슈 close는 하지 않았다.
