# #3820 Stage 184 - internal rewind painted-row boundary

## 목적

최신 `upstream/devel` 리베이스 뒤 전체 `release-test`에서 발견된 #3820 p94 native
RowBreak 회귀를 source frame 계약으로 보정한다.

## 재현

`issue_3820_rewinding_rowbreak_uses_painted_first_fragment_boundary`에서 연구보고서
p94 표 28의 첫 fragment가 기대 `[0, 1, 2]` 대신 `[0, 1, 2, 3]`을 소유했다.
동일 테스트는 최신 `upstream/devel`에서 통과했다.

## 원인

native HWP5 RowBreak의 painted-row first-fragment 조건은 표 **다음 문단**의 saved
`vpos` rewind만 검사했다. p94 표 28의 물리 fragment 경계는 다음 문단이 아니라 표
셀 안 `lineSeg`의 양수 좌표에서 0으로 되감기는 전환에 저장되어 있었다.

이 source evidence를 놓치면 선언 `common.height` whole-fit 경로가 선택되어 실제
paint 행 경계를 한 행 넘어선다.

## 수정

- 다음 문단 rewind 또는 표 내부 stored `vpos` rewind 중 하나가 있으면 native HWP5
  first fragment의 whole-row fit을 measured paint row footprint로 판정한다.
- 파일명·쪽 번호·고정 pixel allowance를 추가하지 않았다.

## 검증 상태

Stage 184 커밋 뒤 #3820 직접 계약, #3820/#3930 focused regression, 전체 release-test
재실행 결과를 차례로 기록한다.
