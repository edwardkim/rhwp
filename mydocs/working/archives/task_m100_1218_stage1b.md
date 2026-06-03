# Stage 1b 완료보고서 — Task M100 #1218 (① 위치 경로 계측 + 수정 설계)

**단계**: ①(pi=259) 위치 계산 경로 확정 + 최소·안전 수정 설계
**브랜치**: `local/task1218`

## 계측 (RHWP_DEBUG_TAC_CURSOR, 4쪽)

```
FullPara pi=257       y_in=804.3 y_out=822.4 dy=18.0
Shape pi=258 ci=1..4  y=822.4 dy=0.0          (수식, 인라인)
Table pi=258 ci=5     y_in=822.4 y_out=895.9 dy=73.6   ← 표 높이만큼 커서 전진
PartialPara pi=258    y_in=895.9 y_out=895.9 dy=0.0    ← 호스트 본문, 커서 전진 0
FullPara pi=259 ①     y_in=895.9 y_out=914.0 dy=18.0   ← 표 하단에서 시작 → 겹침
```

## 확정 근본 원인

- 렌더 위치는 pagination `current_height`(163.7px)가 아니라 **파일 vpos + HeightCursor(VPOS_CORR)** 로 계산 (typeset 누적과 별개).
- `wrap=Square` 표(pi=258 ci=5)는 커서를 **표 높이(73.6px)만큼** 전진(822.4→895.9).
- 호스트 본문 5줄은 `layout_wrap_around_paras`(반환 없는 void 렌더)로 그려지며 **커서 전진 0**.
- 본문 5줄(≈90px) > 표(73.6px) 이므로 본문 마지막 줄이 895.9 아래까지 내려오는데, ①(pi=259)이 895.9 에서 시작 → **마지막 본문 줄과 ① 겹침**.

→ **표 호스트 단락에서 커서가 `table_bottom` 까지만 전진하고 `host_text_bottom` 을 무시.** `max(table_bottom, host_text_bottom)` 이어야 함.

## 최소·안전 수정 설계

- **위치**: `layout.rs` Table item 처리(어울림 호스트) — `layout_wrap_around_paras` 호출 후 커서 new_y.
- **변경**: 어울림(wrap=Square) 호스트 표의 new_y 를 `table_bottom.max(host_text_bottom)` 로. `host_text_bottom` = 표 시작 y(822.4) + 호스트 본문 줄 높이 합(이미 `layout_wrap_around_paras` 가 렌더한 줄 범위).
- **가드**: 본문이 표보다 짧으면(기존 다수 케이스) `table_bottom` 우세 → 동작 불변. 본문이 더 길 때만 전진 증가 → 회귀 최소.

## z-표 행 압축(별도)

셀[2] `lh=825<font 900` 로 행 겹침. Table item 과 독립 — 별도 stage 에서 셀 내부 줄높이/valign 정합.

## 리스크 / 다음 단계

- 수정 대상이 HeightCursor/wrap-around 인접 코어. `max()` 만 추가하는 최소 변경이나, 다수 wrap=Square 샘플(인용 ｢｣, 학생 글상자 등) **시각 회귀 점검 필수**.
- Stage 2 에서 `host_text_bottom` 계산 정확도(줄 범위·줄높이) 검증 후 적용.
