# Task #312 1단계 완료 보고서: 단별 used_height 측정 도구

상위: 구현 계획서 `task_m100_312_impl.md`, Epic #309

## 변경 요약

`dump-pages` 출력에 단별 누적 사용 높이(`used`)와 HWP 의도 사용 높이(`hwp_used`), 그 차이(`diff`)를 노출. 동작 변경 없음.

## 변경 파일

- `src/renderer/pagination.rs::ColumnContent` — `used_height: f64` 필드 추가
- `src/renderer/pagination/state.rs::flush_column` / `flush_column_always` — 단 닫을 시점의 `current_height` 저장
- `src/renderer/typeset.rs` — 동일 (TypesetEngine flush)
- `src/document_core/queries/rendering.rs::dump_page_items` — 단 헤더에 `used=`, `hwp_used≈`, `diff=` 출력
- `src/document_core/queries/rendering.rs::compute_hwp_used_height` — 신규 helper. vpos-reset 우선, 미발견 시 마지막 항목 마지막 줄 vpos+line_height 환산
- `src/renderer/layout/tests.rs` — 테스트 fixture 5건에 `used_height: 0.0` 추가

## 출력 예 (21_언어 페이지 7)

```
=== 페이지 7 (global_idx=6, section=0, page_num=7) ===
  body_area: x=117.2 y=209.8 w=888.2 h=1226.4
  단 0 (items=13, used=1062.5px, hwp_used≈1210.7px, diff=-148.2px)
    PartialParagraph  pi=115  lines=3..8  vpos=60644..67908
    ...
  단 1 (items=8, used=388.8px, hwp_used≈1030.1px, diff=-641.3px)
    FullParagraph  pi=128  ...
```

`hwp_used` 계산:
1. 단의 항목 내 첫 vpos-reset (line>0, vpos==0) 검색 → reset 직전 줄의 `vpos + line_height` 환산
2. reset 미발견 → 마지막 항목 마지막 줄의 `vpos + line_height` 환산
3. Table/Shape는 컨테이너 paragraph만으로 부정확하므로 None

## 페이지 7 21_언어 1차 분석 (2단계 데이터 시드)

| 단 | items | used (우리) | hwp_used | diff |
|----|-------|-------------|---------|------|
| 0 | 13 (pi=115..127) | 1062.5px | 1210.7px | **-148.2px** |
| 1 | 8 (pi=128..134) | 388.8px | 1030.1px | -641.3px |

**관찰**:
- 단 0: 우리 단이 HWP 가용보다 148px 적게 사용. 그러나 우리는 13개 항목, HWP는 3개 항목(pi=115(3..8) + pi=116 + pi=117 line 0)으로 끊음. **즉 우리가 같은 paragraph들을 더 작은 vertical 공간에 압축 배치**.
- 단 1: 우리는 8개 항목 (pi=128..134) 만 배치. HWP는 같은 단에 pi=117(line 1+) + pi=118..134 등 훨씬 많은 항목. 우리가 단 0에 다 채워서 단 1이 비어있는 모양.

**가설(2단계에서 검증)**: 우리 엔진이 line vertical step (line_height + line_spacing) 을 HWP보다 작게 계산하는 곳이 있다. 한 줄당 ~7px 더 작게 처리되어 같은 paragraph들이 단 0에 다 들어감.

## 회귀 검증

- `cargo build` 성공
- `cargo test`: **992 passed; 0 failed**
- 4개 샘플 페이지 수 무변화 (19/20/25/11)

## 다음 단계

2단계: 페이지 7 단 0의 line-by-line vpos 진행을 추적하여 어떤 paragraph에서 누적 차이가 시작되는지 식별.
