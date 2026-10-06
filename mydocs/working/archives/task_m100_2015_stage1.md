# #2015 Stage 1 완료보고서 — 기준선 하네스 + 코드 국소화

- 이슈: #2015 / 브랜치: `fix/2015-saved-bounds-rowbreak-overflow` (base `origin/devel`)
- 범위: 소스(렌더러) **무수정**. 회귀 테스트 하네스 추가 + 오차 발생 지점 코드 국소화.

## 1. 재현 고정 (base=origin/devel)

```
LAYOUT_OVERFLOW: page=3, sec=0, col=0, para=52, type=PartialTable, first=false,
                 y=1117.7, bottom=1026.5, overflow=91.2px
dump-pages -p 3:  단 0 (items=7, used=806.0px)
```

- 부수 관찰: p2 `pi=26 overflow=1.6px`(sub-2px, 같은 계열 가능성). Stage 4에서 동반 여부 확인.

## 2. 기준선 하네스

`tests/issue_2015_saved_bounds_rowbreak.rs` 추가. `get_page_render_tree(3)`(public) JSON을 파싱해
**Body 서브트리 content 노드(Table/Cell/TextLine/TextRun/Line/Shape/Picture)의 최대 바닥 ≤ Body 바닥**
불변식을 검사. 매직넘버·텍스트매칭 없이 geometry 로만 판정.

| 테스트 | 역할 | 현재 |
|---|---|---|
| `issue_2015_stage1_documents_current_overflow` | 기준선 계측(항상 통과) | body_bottom=1026.5, worst=1118.2(Table), **overflow=91.7px** |
| `issue_2015_page4_rowbreak_table_stays_in_body` | 정답 불변식 | `#[ignore]`(red). Stage 2에서 ignore 제거 |

- `#[ignore]`로 CI green 유지. `--include-ignored` 실행 시 정답 테스트가 91.7px 초과로 fail(기준선 red 확인).
- Body/Column 컨테이너 노드는 clamp(바닥=1026.5)되므로 content 타입만 대상.

## 3. 코드 국소화

### 발원지 ① 부동 RowBreak 표 (pi=52) — 포맷 공통, Stage 2 대상

- 표 속성(`dump -s0 -p52`): `treat_as_char=false`, `wrap=자리차지`, `vert=문단(10823=38.2mm)`,
  `size=48214×8555`(저장 114px) vs 셀[5] 내용 `h=21111`(281px, 별표 문단 6개).
- `ir-diff -s0 -p52` **차이 0건** → 파싱/포맷 아님, 순수 레이아웃 회계.
- typeset(`src/renderer/typeset.rs`): 자리차지 float 표는 `apply_visible_float_exclusions`(≈1664)
  기반 예약으로 처리되어 선형 `current_height`(=`used`, 806.0px)에 **저장바운드 기준**만 반영.
- layout(`src/renderer/layout/table_layout.rs`): PartialTable 프래그먼트를 **내용 파생 높이**(row-cut)
  로 그려 앵커+offset 위치에서 1117.7px까지 내려가 body 바닥 초과.
- 결론: **typeset 예약 높이(저장바운드)와 layout 실측 row-cut 높이의 불일치**가 91.2px 오차.

### 발원지 ② HWPX 인라인 표 (pi=50) — HWPX 전용, Stage 3 대상

- `ir-diff -s0 -p50` **line_segs A=0(HWPX)/B=1(HWP)** → HWPX 표 컨테이너 lineSeg 미저장 → 합성 `lh=28769`.
- 합성 셀 내부 별표 줄 피치: 문단 내 1818HU(24.24px)/문단 사이 +500HU(≈30.9px) 교대 → 한컴 균일 렌더와 불일치.
- 대상: `src/renderer/composer.rs` / `table_layout.rs` HWPX 합성 lineSeg 높이·줄 피치.

## 4. 다음 단계

Stage 2: 발원지 ① — typeset 자리차지 float 예약 높이를 layout row-cut 실측과 정합하여
`para=52` 오버플로우 소거. 완료 후 정답 테스트 `#[ignore]` 제거. 광범위 회귀(1749/1035/1139) 필수.
