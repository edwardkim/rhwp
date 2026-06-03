# Stage 2 진단 보고서 — #1046: 근본 원인 확정 (측정 드리프트 아님 → 배치 정책)

- 타스크: #1046 (M100), 브랜치 `local/task1046`
- 단계: 측정 통일(B) Stage 2 — 진단 우선(작업지시자 승인 "진단 먼저 1커밋 후 수정")
- 작성일: 2026-05-21
- 대상: page 28(global 27, 0-based), pi=242 (7행×3열 표, vert_offset 형)

## 1. 추가한 진단 (env 게이트 `RHWP_TABLE_DRIFT`, 동작 불변)

- `TABLE_SPLIT_AVAIL`: 분할 fragment 진입 시 가용공간 분해 — cur_h / table_avail /
  caption / host_before / vert_off / page_avail / header_oh / avail_for_rows / start_cut.
- `TABLE_SPLIT_RESULT`: walk 종료 후 fragment 경계/소비 — cursor_row / end_row /
  consumed / partial_h / split_end_limit / avail_for_rows / fits(=partial_h≤avail).

## 2. 결정적 측정 (pi=242, page 28)

```
TABLE_SPLIT_AVAIL:  pi=242 cursor_row=0 cont=false cur_h=875.7 table_avail=941.1
                    caption=0.0 host_before=4.0 vert_off=38.1 page_avail=23.4
                    header_oh=0.0 avail_for_rows=23.4 start_cut=[]
TABLE_SPLIT_RESULT: pi=242 cursor_row=0 end_row=1 consumed=34.9 partial_h=34.9
                    split_end_limit=0.0 avail_for_rows=23.4 fits=FALSE
LAYOUT_Y:           page=27 pi=242 y_after=1066.2 (body_top=105.8)
LAYOUT_OVERFLOW:    page=27 para=242 type=PartialTable y=1066.2 bottom=1046.9 overflow=19.2px
```

- `vert_off=38.1px`(wrap=TopAndBottom, vert=Para, vertical_offset=2854HU)는 페이지네이터가
  **정확히 차감** → page_avail=941.1−875.7−4.0−38.1=23.4px. **계측 정합**.
- 행0(34.9px) > avail 23.4px → 페이지네이터가 `fits=false` 로 **안 들어감을 인지**.
- 그럼에도 fragment(rows=0..1, 행0 통째)를 배치 → 렌더러 82.8px(=vert_off 38 + 행0 34.9 +
  host) 그려 본문 19.2px 초과.

## 3. 근본 원인 확정 (Stage 1·v2 초기 가설 모두 정정)

- **측정 드리프트 아님**: 행높이(cut=mt=eff), vert_offset, host_before 전부 정합.
- **결함은 배치 정책**: `typeset.rs` 일반 행 강제 분기
  ```
  if !splittable {
      if r == cursor_row { consumed += ...; end_row = r+1; }  // 강제 통째(오버플로 감수)
      ...
  }
  ```
  `r==cursor_row`(표의 현재 커서 행=행0)이면 잔여 부족(`fits=false`)에도 강제 배치한다.
  이 강제는 본래 *genuine page-larger*(행이 fresh 페이지보다 큰 경우)용인데, 그 가드
  (`block_h > base_available`)가 **블록 분할 경로**(typeset.rs ~2980)엔 있으나 **일반 행
  강제 경로엔 없다**.
- 행0(34.9px)은 fresh 페이지(941px)면 여유 → **표 전체를 다음 페이지로 이월**이 정답.

## 4. Stage 2 수정 방향 (다음 단계, 승인 대기)

비연속(`!is_continuation`) + 표 시작이 페이지 상단이 아님(current_items 비어있지 않음) +
강제 대상 행/블록이 fresh 페이지엔 들어감(`row_total ≤ base_available`)이면, fragment를
배치하지 않고 표 전체를 다음 페이지로 이월. genuine page-larger(>base_available)는 기존
강제 유지. 블록 경로의 `genuinely_page_larger` 가드와 동일 원리를 일반 행 경로에 적용.

## 5. 코드/회귀 상태
- 진단 2종(typeset.rs)만 추가, 모두 `RHWP_TABLE_DRIFT` 게이트. 동작 불변.
- overflow 18건(= in-scope 16 + page-larger 2: pi=323, pi=567) — baseline 불변.
