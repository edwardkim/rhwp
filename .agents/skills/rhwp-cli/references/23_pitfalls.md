# 함정

실측에서 에이전트가 반복하는 실수다.

## P01 — 페이지 1부터를 -p 에 그대로

한컴 4쪽 → -p 3. -p 4 는 범위 초과(2) 또는 다음 쪽.

## P02 — dump -p 와 dump-pages -p 혼동

전자는 문단, 후자는 페이지.

## P03 — 자기 라운드트립 = 한컴 호환

세 층이 다르다. 19장.

## P04 — oracle/generated 순서 뒤집기

힌트가 반대로 나온다.

## P05 — export-png 부재를 성공으로

exit 2. 재빌드.

## P06 — ir-diff 텍스트 모드 차이 = 실패

텍스트는 0, --json 만 3.

## P07 — 실패 JSON 을 jq

stdout 0바이트. exit 먼저.

## P08 — info 표 개수 = 실제 표

글상자·머리말 안 표는 놓친다. export-tables.

## P09 — export-markdown 으로 병합 표

빈 칸이 생긴다. export-tables.

## P10 — thumbnail 을 렌더로

PrvImage. 화면과 다를 수 있다.

## P11 — convert 출력을 .hwpx

exit 2. export-hwpx.

## P12 — extract-pages --from 에 0 기준

그 명령만 1 기준.

## P13 — profile 없이 인쇄 PDF

legacy 는 editor_only 를 보여 줄 수 있다.

## P14 — --profile + --embed-fonts

exit 2.

## P15 — overflowCellLines 무시

셀 줄이 쪽 밖에 있다.

## P16 — HU 와 px 를 1:1

1px=75 HU, 1인치=96px=7200 HU.

## P17 — 없는 명령을 스킬에 추가

새 CLI 금지.

## P18 — gym 점수로 레이아웃 판정

이 스킬은 gym 이 아니다.

## P19 — 암호 없음(2)과 틀림(1) 혼동

NeedPassword vs WrongPassword.

## P20 — pdf --backend direct 부재를 2로

그 경로는 1.
