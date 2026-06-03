# Task #1195 Stage 1 완료 보고서 — 결함 정밀 계측 (코드 무수정)

- **이슈**: #1195
- **브랜치**: `local/task1195`
- **단계**: Stage 1 / 4 (계측 — 소스 무수정)

## 계측 결과 (hcar-001 1쪽, output/poc/issue1195/stage1/)

### SVG 좌표 (rhwp 현재)
| 요소 | y (px, 96dpi) |
|------|---------------|
| 본문 마지막 줄("※ …제출") | 618.2 |
| **4×1 "1.동의" 표 top** | **631.8** |
| **제목 "1. 개인정보…동의[필수]" 글자줄** | **639.8** |
| 표 첫 행 영역 | 631.8 ~ 662.9 |
| 5×1 "2.위탁" 표 top | 779.9 |

### 결함 정량 확정
- **제목 글자 y=639.8 이 표 첫 행(631.8~662.9) 내부** → 표가 제목을 8px 덮어 겹침(역전).
- 한컴 정답지(`pdf_p1-1.png`)는 제목 아래 **빈 줄 간격 후** 표 배치 — 제목과 표 분리.
- 즉 제목과 표 사이 "표 앞 빈 줄"(p[9] line_seg, "     ") 높이(약 1줄 ≈ 28px)가 누락되어
  표가 제목 위치까지 끌려 올라옴.

## 근본 원인 (코드 확정 — Stage 1 에서 경로 정정)

- 당초 의심한 `layout_inline_table_paragraph`(354)는 **셀 안 표에 호출 안 됨**
  (`RHWP_LAYOUT_DEBUG=1` 로그 0건 확인).
- 실제 경로: 셀 다문단 루프(`table_cell_content.rs:683`) → `layout_composed_paragraph` →
  **`paragraph_layout.rs:3099~3135` 인라인 TAC 표 배치**:
  ```rust
  let table_y = (y + baseline + om_bottom - table_h).max(y);   // L3104
  ```
  - `y` = 표 anchor 문단 줄 흐름 y(= 제목 직후), `table_h` = 표 높이(큼).
  - 표 하단을 `y+baseline`에 맞추려다 `table_h`가 커서 `(… − table_h)` 가 작아짐 → `.max(y)` 로
    **표 top 이 `y`(제목 줄)로 클램프** → 표가 제목 위로 겹침.
  - **표 anchor 문단 안의 "표 앞 빈 줄"(line_seg vpos 흐름)이 `y` 에 반영되지 않는 것**이 본질.
    작업지시자 지적("빈 줄을 조판에 안 씀")의 코드 실체.

## Stage 2 보정 방향 (다음 단계)

- 인라인 TAC 표(`paragraph_layout.rs:3099`)에서 **표 anchor 문단의 표-앞 빈 줄 높이를 `table_y`
  기준 y 에 반영**. line_seg vpos(표 line_seg.vpos − 문단 첫 line_seg.vpos) 또는 표 앞 빈 줄
  line_height 합산 중 PDF 정합 방식 채택.
- 구조 가드: **표 앞에 빈 줄/빈 세그먼트가 있는 경우** 한정 → 표가 문단 첫 줄인 기존 케이스 무회귀.
- `.max(y)` 클램프가 표를 제목 위로 올리는 동작을 표-앞 빈 줄 높이만큼 하향 보정.

## 산출물
- rhwp SVG: `output/poc/issue1195/stage1/hcar-001_001.svg`
- 한컴 정답지: `output/poc/issue1195/stage1/pdf_p1-1.png`
- 계측 로그: 제목 y=639.8 vs 4×1 표 top y=631.8 (8px 겹침)
