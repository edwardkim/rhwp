# Task #295 1단계 — 원인 분석 보고서

## 결론 (가설 강도: 높음)

**좌단(col 0) 첫 항목 `pi=22`(단나누기 + Paper-앵커 푸터 표 2개)의 처리에서 `y_offset`이 푸터 표 위치(약 y=1371px, 본문 하단)까지 끌려 올라간다.** 이후 같은 단의 본문 항목 `pi=23..27`(29번 문제)이 그 y에서 시작하면서 본문 하단 ~1213.3px를 초과해 LAYOUT_OVERFLOW로 기록되고, 시각적으로는 푸터 영역에 압축·겹침으로 출력된다.

## 핵심 데이터

### 1. 항목별 실제 y 좌표 (디버그 오버레이 추출)

```
좌단(col 0):              우단(col 1):
  s1:pi=22 y=1371.5         s1:pi=28 y=147.4   ← 정상 (본문 상단)
  s1:pi=23 y=1340.1   ❌    s1:pi=29 y=267.7
  s1:pi=24 y=1345.3   ❌    s1:pi=30 y=279.1
  s1:pi=25 y=1345.3   ❌    s1:pi=31 y=360.4
  s1:pi=26 y=1345.3   ❌    s1:pi=32 y=386.9
  s1:pi=27 y=1371.8   ❌    s1:pi=33 y=497.3
```

본문 영역: `y=147.4 ~ y=1360.7` (h=1213.3). 좌단 모든 본문 항목이 본문 하단 또는 그 아래에 배치됨.

### 2. `pi=22` 정체

- 텍스트 비어 있음, `[단나누기]` 마커
- 컨트롤 2개:
  - `ctrl[0]`: 표 1×1, **wrap=TopAndBottom, vert=Paper/101954 HU(≈357mm)**, size=111.0×35.6mm, "확인 사항" 본문 — **페이지 하단의 페이퍼 앵커 푸터 박스**
  - `ctrl[1]`: 표 1×1, wrap=TopAndBottom, tac=true(추정), 페이지 번호 "12" 원형 표
- `line_seg[0].vpos=0, lh=3623`

### 3. `pi=23..27` 정체 (29번 문제 본문)

- pi=23: 본문 + 수식4 (h=55.7)
- pi=24, 26: 빈 줄 (h=15.3)
- pi=25: 본문 + 수식3 (h=51.3)
- pi=27: **표 5×2 wrap=Square**(어울림) + 수식 5

총 합계는 단 가용 높이(1213.3px) 안에 충분히 들어감. PDF에서도 좌단 한 단에 깔끔히 들어감.

### 4. 오버플로 로그

```
LAYOUT_OVERFLOW page=3 col=0 para=25 type=Shape  y=1375.9 bottom=1360.6 overflow=15.3
LAYOUT_OVERFLOW page=3 col=0 para=26 type=Full   y=1371.8 bottom=1360.6 overflow=11.2
LAYOUT_OVERFLOW page=3 col=0 para=27 type=Table  y=1511.5 bottom=1360.6 overflow=150.9
LAYOUT_OVERFLOW page=3 col=0 para=27 type=Shape  y=1511.5 bottom=1360.6 overflow=150.9 (×5)
```

`y=1340..1511`은 모두 푸터 표 위치(1371) 인근 또는 그 이후에서 시작한 결과.

## 추정 메커니즘

`src/renderer/layout.rs::build_single_column`은 `col_content.items`를 순서대로 처리하며 `y_offset`을 누적한다. 페이지네이션이 col 0 항목 순서를 다음과 같이 배치:

```
pi=22 (단나누기 + Paper-푸터 표 2개)  ← 먼저
  ↓
pi=23..27 (29번 문제 본문)            ← 나중
```

`pi=22` 내부 표 `ctrl[0]`이 wrap=TopAndBottom + vert=**Paper**/101954 HU로, 절대 y=1371px에 배치된다. 이 표가 처리되면서 다음 중 하나로 `y_offset`이 올라가는 것으로 보임:

1. **shape_reserved 또는 layout_table 반환값**이 표 하단(≈1506px)을 `y_offset`으로 설정
2. 또는 `vpos_page_base/lazy_base`가 푸터 표의 vpos로 잘못 잡혀, 후속 문단 vpos 보정이 푸터 위치 부근으로 이동

`build_columns`(layout.rs:1148)는 `body_wide_reserved`로 본문 폭 80% 이상의 TopAndBottom 표를 모든 단의 시작 `y_offset`에 반영하지만, 푸터 표는 폭 105.2px(<800px)이므로 거기엔 포함되지 않는다. 따라서 시작 `y_offset`은 정상(`col_area.y=147.4`)이며, **순회 도중 pi=22 처리에서 jump가 일어남**이 가장 유력.

대조:
- 우단(col 1)은 첫 항목이 `pi=28` 본문이라 정상 시작.
- 페이지 11(다른 페이지)도 같은 푸터 구조이지만, 좌단 첫 항목이 본문이라(동일 단나누기 마커가 col 0 끝에 위치) 영향을 받지 않을 가능성.

## 부수 문제: 머리말 페이지번호 `4` ↔ `2`

- PDF: 좌상단에 `4` (장/구역 번호로 추정 — "4. 단답형"의 장 번호)
- 우리: `2` 표시 — 머리말이 어떤 정수를 다르게 산정함

본 타스크 범위에 포함시키지 말고 **별도 이슈로 분리** 권고. 12쪽 좌단 붕괴와는 독립 문제임.

## 다음 단계 제안 (2단계 구현 계획)

검증해야 할 코드 경로:

1. `layout.rs::layout_column_item` — pi=22 처리 시 `y_offset` 변화 추적
2. `layout/shape_layout.rs::calc_shape_bottom_y` — Paper-vert 표의 `bottom_y` 계산값
3. `layout.rs:1455~1480` 인근 — Paper-anchored TopAndBottom 표가 `y_offset` 누적에 기여하는지 분기 확인
4. `vpos_page_base/lazy_base` 갱신 로직 — pi=22 처리 후 비정상 baseline이 잡히지 않는지

수정 방향(가설):

- **Paper-앵커 TopAndBottom 표는 텍스트 흐름의 `y_offset`을 진행시키지 말 것** (절대 좌표 배치만 하고 본문 흐름은 유지)
- 또는 pi=22 같은 단나누기 마커 + 페이퍼 푸터 호스트 문단을 본문 흐름에서 분리

추가 검증 필요: 같은 패턴이 다른 페이지에 영향을 주는지 (예: 단답형 섹션의 다른 페이지들).

## 현 단계 작업물

- 분석용 산출물: `output/exam_math_p12/` (PDF/SVG/overlay)
- 본 보고서: `mydocs/working/task_m100_295_stage1.md`
- **소스 변경 없음**
