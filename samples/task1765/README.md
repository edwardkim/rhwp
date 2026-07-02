# Task #1765 재현 샘플

## 파일

- `merged_cell_trailing_ls.hwp`
- `merged_cell_trailing_ls.hwpx`
- `merged_cell_trailing_ls-2024.pdf`

## 목적

이 샘플은 #1765의 초기 가설이었던 "병합 셀 경로의 trailing ls 초과 확장"을 검증한 뒤,
그 가설 기각이 맞음을 증명하고 #1759의 per-line 콘텐츠 측정 누적 대표 사례로 재분류하기 위해 남긴다.

## 출처

- 출처: 국가법령정보센터 `[별표 9] 위험근무수당 등급별 구분표(제13조 관련)(...).hwp`
  (공개 서식, HWP5/OLE, 2쪽)

## 판별

- 문제 후보는 2쪽 12×4 표다.
- Stage 2 행높이 대조 결과, 후보 표의 셀은 전부 `rs=1` 이며 병합 셀 경로를 지나지 않는다.
- +5.2px 는 row9 단일 행의 밀집 셀(23문단, 약 50줄)이 선언높이를 실제로 초과하는 상황에서
  rhwp가 per-line 콘텐츠 측정을 누적해 한컴보다 약 5px 더 커지는 사례다.
- trailing ls 단독 문제나 병합 셀 경로 문제로 보지 않는다.

## 검증

```bash
rhwp export-render-tree samples/task1765/merged_cell_trailing_ls.hwp -p 1
python3 scripts/task1274_visual_sweep.py \
  --file-target pr1766-merged-cell-hwp samples/task1765/merged_cell_trailing_ls.hwp samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --file-target pr1766-merged-cell-hwpx samples/task1765/merged_cell_trailing_ls.hwpx samples/task1765/merged_cell_trailing_ls-2024.pdf \
  --page 2 \
  --out output/pr1766-visual-review
```
