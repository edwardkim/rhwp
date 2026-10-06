# Task M100 #1686 구현 계획서

## 접근

선행 RowBreak 표가 같은 문단 안에서 continuation을 만든 뒤, 후행 co-anchored 표를 같은 문단의 잔여
control로 즉시 배치하는 경로를 좁게 제어한다.

추가 확인 결과 HWP 기준 PDF의 총 페이지 수는 35쪽이며, page 5도 기준 PDF처럼 `동일 기간에
경력이 중복될 경우...` 문장으로 시작해야 한다. 따라서 page 3 co-anchored 표 순서 보정만으로
완료하지 않고, RowBreak 표 셀 vpos reset, 빈 guide 문단 누적, visible host text 렌더 순서까지
같은 샘플 안에서 함께 검증한다.

## 구현 후보

1. `typeset_block_table`이 표 분할 중 page advance를 발생시켰는지 호출자에게 알릴 수 있게 한다.
2. `typeset_table_paragraph`는 한 문단의 선행 비-TAC RowBreak 표가 분할 continuation을 만들면,
   같은 문단의 후행 비-TAC 표를 현재 control loop에서 계속 배치하지 않는다.
3. 단, 후행 표를 영구 누락하면 안 되므로 별도 pending control 모델이 필요하다.
4. pending 모델이 커지면 위험하므로, 우선 #1686 구조에 맞는 보수적 가드와 회귀 테스트를 통해
   최소 변경 범위를 찾는다.
5. HWP RowBreak 분할 행의 미세 overflow는 2px까지만 허용한다. HWPX는 기존 64px 보정 범위를
   유지하되, HWP의 큰 overflow를 허용하지 않아 #1156 계열 회귀를 막는다.
6. 선행 RowBreak 표 조각 뒤 빈 guide 문단이 다음 실질 앵커보다 큰 vpos를 갖는 경우에는 flow
   높이로 누적하지 않는다.
7. visible host text가 있는 RowBreak 표는 첫 조각 앞이 아니라 마지막 continuation 조각 뒤에
   렌더한다.

## 주의할 회귀

- #1510: visible host co-anchored float tables의 문서/control 순서 유지
- #1535: visible host 후행 float가 선행 float 점유 영역을 침범하지 않아야 함
- #1639: 빈 host + 음수 offset 표의 문서 순서 유지
- #1658: RowBreak 분할 표의 page count/clip gate

## 초기 RED 기준

`samples/hwpx/pr-1674.hwpx`와 `samples/pr-1674.hwp`의 page 3 render tree에서
`다. 우대요건 등 [원서접수 마감일 기준]`가 보이지 않는 현재 동작을 실패로 고정한다.
추가로 HWP/HWPX page count 35쪽, HWP page 5의 첫 경력 문장 포함 및 `☞ 임용예정직위...`
안내문 미포함을 회귀 테스트로 고정한다.
