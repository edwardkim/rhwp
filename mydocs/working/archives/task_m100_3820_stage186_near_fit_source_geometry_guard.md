# #3820 Stage 186 - near-fit source geometry guard

## 목적

리베이스 뒤 p94 표 28을 통째로 배치한 `near-measured RowBreak fit`의 source-frame
검증 누락을 보정한다.

## 진단

```text
DIAG_FIT pi=1000 ... declared=true near=true source=false internal=false
cur_h=613.3 total=422.3 avail=956.2
```

p94 표 28은 2px near-measured 예외만으로 declared whole-fit을 통과했다. 그러나
`table_declared_height_has_stored_cell_content_frame`과
`table_declared_object_covers_cell_row_frames`를 함께 적용한 source geometry는 false다.
즉 declared object가 current physical fragment 전체를 소유한다는 근거가 없다.

## 원인

Stage 177의 `near_measured_rowbreak_fits`는 declared object bottom과 measured table
bottom의 근접성만 검사했다. stale/minimum object frame을 배제하는 source geometry
계약을 우회해, 실제로 다음 쪽에서 시작해야 하는 마지막 행을 현재 p94에 합쳤다.

## 수정

- near-measured RowBreak whole-fit도 `declared_excess_has_source_frame`을 요구한다.
- 이 predicate는 모든 text-bearing cell의 stored frame과 table object가 declared row
  geometry 전체를 포함하는지 함께 검사한다.
- 2px은 renderer 측정의 반올림 범위로만 남고, physical fragment owner를 새로 만들지
  않는다.

## 검증 상태

직접 contract를 재실행해 p94 표 28은 `[0, 1, 2]`으로 복구됐다. 이어 p106 표 29에서
`[0, 1, 2, 3]`이 남아, source slack이 native painted footer guard를 다시 넘는 별도
경로를 확인했다. 그 충돌은 Stage 187에서 보정한다.
