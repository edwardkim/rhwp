# Stage 1 — 분석·재현·기준선 캡처 완료 보고서

**Task**: #577 — 셀 내부 단독 TopAndBottom 이미지 1라인 오프셋
**브랜치**: `local/task577`
**단계**: 1/4 (분석·기준선)

---

## 1. 재현 확인

빌드 직후 `samples/exam_science.hwp` 1페이지 baseline 추출:

```
$ rhwp export-svg samples/exam_science.hwp -o output/svg/task577_baseline/
LAYOUT_OVERFLOW: page=1, col=0, para=46, type=Table, y=1374.0, bottom=1364.4, overflow=9.5px
```

(LAYOUT_OVERFLOW 메시지는 별도 표 — Task #566 baseline 과 동일하므로 본 이슈와 직접 연관 없음.)

페이지 1, 보기 표(3행×5열) 좌표:

| 보기 | 셀 클립(y, h) | 이미지(y, h) | 이미지 bottom | 셀 bottom | 오버플로 |
|------|---------------|---------------|---------------|-----------|----------|
| ① | 779.84, 56.83 | 798.95, 41.28 | 840.23 | 836.67 | **+3.55** |
| ② | 779.84, 56.83 | 798.95, 49.28 | 848.23 | 836.67 | **+11.55** |
| ③ | 836.67, 59.60 | 855.77, 40.96 | 896.73 | 896.27 | +0.46 |
| ④ | 836.67, 59.60 | 855.77, 52.80 | 908.57 | 896.27 | **+12.30** |
| ⑤ | 896.27, 54.05 | 915.37, 45.76 | 961.13 | 950.32 | **+10.81** |

전 5개 모두 cell-clip 영역을 일정한 19.10 px 오프셋만큼 아래로 밀려 있고, 이미지 height가 큰 ② ④ ⑤가 시각적으로 잘림.

## 2. 좌표 산식 검증

`table_layout.rs:1413` 의 `para_y_before_compose = para_y` 시점 anchor 좌표:

```
para_y_before_compose = cell_y + pad_top
                      = 896.27 + 3.78 = 900.05  (셀 ⑤ 기준)
```

`layout_composed_paragraph` 가 빈 anchor 라인 1줄(line_height=1150 HU = 15.32 px) 만큼 advance:

```
para_y (after compose) = 900.05 + 15.32 = 915.37
```

`compute_object_position` (vert_rel_to=Para, vert_align=Top, v_offset=0):
```
pic_y = para_y = 915.37
```

→ 관측값 정확히 일치. **버그 산식 확정.**

## 3. 적용 범위 결정

수정 조건: `pic.common.text_wrap == TopAndBottom` **AND** `pic.common.vert_rel_to == VertRelTo::Para`.

이미지-only 단락 추가 한정 여부 검토:

- HWP 의미상 TopAndBottom 이미지는 anchor 라인을 displace 하므로, 텍스트 + 이미지 혼합 단락에서도 anchor 좌표 기준이 맞다.
- `para_y_before_compose` 는 단락 시작 절대 y 이며, 다중 라인 텍스트를 거쳐 이미지가 나타난다 해도 anchor 라인은 항상 첫 라인(또는 line_segs[0])이다. 본 이슈의 셀 단락은 모두 `paras=1, ctrls=1, text_len=0`.
- 추가 한정은 보수성을 높이지만, 산식 자체는 단순해야 회귀 분석이 쉽다.

→ **조건 1·2 만 적용**, 추가 한정 없음. 단계 3 시각 회귀 검증으로 위험 통제.

## 4. 회귀 검증 대상 샘플 선정

| 샘플 | 선정 사유 |
|------|----------|
| `samples/exam_science.hwp` | 본 이슈 대상. 표 안 비-TAC TopAndBottom 이미지 5개 |
| `samples/mel-001.hwp` | 21페이지 중 다수 표·이미지 포함, baseline LAYOUT_OVERFLOW 8건 (확장 회귀 검증용) |

baseline SVG 보관 위치:
- `output/svg/task577_baseline/` (exam_science)
- `output/svg/task577_baseline_other/` (mel-001)

## 5. 다음 단계

Stage 2 — `table_layout.rs:1624..1648` 에서 anchor_y 도입, build·test·clippy 통과 확인.

## 6. 승인 요청

Stage 1 결과 검토 후 Stage 2 진행 승인 부탁드립니다.
