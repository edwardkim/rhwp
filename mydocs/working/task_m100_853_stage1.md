# Stage 1 완료 보고서 — Task #853 (M100) — 진단

GitHub Issue: edwardkim/rhwp#853 · 브랜치: `local/task853` · 코드 미수정 (진단 전용)

## 1. 관측 데이터 (shortcut.hwp 1쪽)

`export-svg` 출력 (`output/svg/sc842/shortcut_001.svg`) ↔ `pdf/basic/shortcut-2022.pdf` (stage5 `mydocs/working/task_m100_842_stage5.md` 의 pdftotext -bbox 측정):

| 요소 | rhwp SVG | PDF (한글 2022) | 차이 |
|------|----------|-----------------|------|
| 본문영역 상단 | y=56.7 | 15mm ≈ 56.7 | 0 |
| 제목 텍스트 (baseline / top) | baseline 79.4 / top ≈ 58 | top ≈ 83.6 | **rhwp ~25px 높음** |
| "커서 이동" 헤더 띠 rect | y=103.1 ~ 126.7 (h=23.5) | 텍스트 144~160 → 띠 ~127~150 추정 | **rhwp ~25px 높음** |
| 첫 본문행 "빈칸 삽입" | y ≈ 130.5 (column rect 상단) | 텍스트 top ≈ 194.8 | **rhwp ~64px 높음** |
| 본문 행 pitch | ~20px (vpos 1500 HU) | ~20px (15pt) | 동일 |

→ 콘텐츠가 위로 갈수록 누적 압축: 제목 ~25px + 헤더 띠 ~25px(누적) + 헤더↔본문 ~+39px(누적 64px). 본문 행 pitch 자체는 정상.

## 2. 근원 규명 — 확인됨

### (A) `다단나누기` column-band-top / 섹션-top 문단에서 `spacing_before` 가 통째로 버려짐 ★ 주 원인

`src/renderer/layout/paragraph_layout.rs:745-748`:
```rust
// 단/페이지의 맨 처음 문단은 spacing_before 적용하지 않음
let is_column_top = (y - col_area.y).abs() < 1.0;
if start_line == 0 && spacing_before > 0.0 && !is_column_top {
    y += spacing_before;
}
```
- 제목 문단 0.0: `PS before=3968 HU` (= 52.9px), `LINE_SEG vpos=1984 HU` (= 26.45px). 제목은 섹션 0 의 첫 문단 → `y == col_area.y` → `is_column_top=true` → **`before` 52.9px 가 0 으로 버려짐**. 그래서 제목 baseline 이 `body_top + ascent ≈ 79.4` 에 놓임.
- 한글 2022 PDF 는 제목 텍스트 top 을 `body_top + 26.9px ≈ 83.6` 에 놓음 — 이는 `LINE_SEG vpos=1984 HU (26.45px)` 와 정확히 일치. 즉 **한글은 column/섹션 top 문단에서도 LINE_SEG.vpos(= `before/2` 인 경우)를 그대로 존중**하는데, rhwp 는 `is_column_top` 예외로 통째 버린다.
- shortcut.hwp 의 각 섹션 헤더(`커서 이동`·`지우기`·`파일`·…)는 `다단나누기` 컨트롤로 새 column band 를 시작하고, 그 band 의 첫 문단(= TAC 표 anchor 문단 또는 본문 첫 줄)이 매번 `is_column_top` 이 되어 동일하게 `spacing_before` 가 사라진다 → 모든 구분 칸 위·아래 간격이 ~20px 씩 부족 (사용자 보고 증상 1).

### (B) 페이지네이션(`height_measurer`)과 배치(`paragraph_layout`)의 `spacing_before` 비대칭

`src/renderer/height_measurer.rs:341`:
```rust
let total_height = (spacing_before + lines_total + spacing_after - clickhere_adjustment).max(0.0);
```
- `height_measurer` 는 `is_column_top` 가드가 없어 column-top 문단도 `spacing_before` 를 **항상** 높이에 포함. 반면 `paragraph_layout` 은 버린다 → 페이지네이터가 예약한 높이와 실제 렌더 높이가 어긋남.

## 3. 페이지 영역 초과 (증상 2) — 부분 규명

- 3쪽: 단0~단18(19개 zone) 누적, 마지막 zone(단17/18) `zone_y_offset = 720.2px` 인데 body_area 높이는 701.7px → 콘텐츠 y ≈ 766 > body_bottom 758.4 (SVG max y 측정). 1·2·4·5·7쪽은 본문영역 내, 6쪽 752.6px.
- (B) 의 비대칭이 직접 원인은 아닐 가능성(렌더가 페이지네이션보다 *위로* 어긋나면 콘텐츠가 더 일찍 끝남) — 3쪽 초과는 **별개 결함**으로 보임:
  - 3쪽 단3: `<편집 화면 분할에서>`(pi=94, vpos=0)와 "화면 이동"(pi=95, vpos=0)이 **둘 다 vpos=0** 으로 겹침 → 닫힌 **#768**(다단 zone 분할 행 밀림) 패턴. 이 zone 높이 오산이 3쪽 누적에 기여.
  - 페이지네이터가 누적 zone offset 이 body_bottom 을 넘는데도 다음 페이지로 break 하지 않는 경로(TAC 표 띠 + 다단 zone 조합) 의심 — 단, 한글 PDF 3쪽이 동일 콘텐츠를 모두 담는지(= 한글도 본문영역 초과 vs rhwp 가 과적재) Stage 2 에서 PDF 페이지별 콘텐츠 대조로 확정 필요.

## 4. 영향 코드 경로

| 경로 | 역할 |
|------|------|
| `src/renderer/layout/paragraph_layout.rs:745-748` | `is_column_top` 예외 — 주 수정 후보 |
| `src/renderer/height_measurer.rs:341` 외 | column-top 시 `spacing_before` 포함 여부 — `paragraph_layout` 과 정합 필요 |
| `src/renderer/layout.rs` zone 배치(`start_new_column_band` 부근), 페이지 break 판정 | 3쪽 초과 — 누적 offset > body_bottom 시 break, #768 패턴 |
| LINE_SEG `vpos`/`lh`/`bl` 해석 (`src/document_core/` / paragraph_layout 줄 루프) | 대안: column-top 에서 LINE_SEG.vpos 를 첫 줄 위치로 존중 |

## 5. Stage 2 방향 (옵션)

1. **옵션 A — `is_column_top` 예외 범위 축소**: 페이지 break 로 *연속*된 column-top 에서만 `spacing_before` 드롭, 섹션-top·`다단나누기` band-top 에서는 적용. 컨텍스트(연속 여부)를 caller 에서 전달 필요. + `height_measurer` 정합.
2. **옵션 B — column-top 에서 LINE_SEG.vpos 존중**: 파일에 기록된 vpos(= 한글이 실제 렌더한 위치)를 첫 줄 top 으로 사용. 가장 충실하나 `respect_vpos` 류 — `feedback_essential_fix_regression_risk` 경고(다단/단일 단/표분할 상호작용 회귀 위험) 적용, 광역 sweep 필수.
3. **옵션 C — column-top 에서 `spacing_before/2` 적용**: vpos=1984 = before/2 관찰 기반. shortcut.hwp 한정으로는 맞지만 일반 룰 근거 불충분 — `feedback_rule_not_heuristic` 위배 소지. 비권장.

→ **권장: 옵션 A** (예외 범위 축소 + height_measurer 정합) 를 1차 시도, 안 맞으면 옵션 B 검토. 3쪽 초과(#768 패턴)는 Stage 3 에서 별도 처리하되 한글 PDF 페이지별 콘텐츠 대조로 "한글도 초과인가" 먼저 확인.

## 6. 결정 사항 (작업지시자 승인 요청)

1. Stage 2 옵션 — A(권장) / B / C 중 선택.
2. 3쪽 초과(#768 패턴) — 본 타스크 흡수 vs #768 재오픈 분리. (Stage 1 결과: rhwp 측 zone 높이 오산이 근원으로 보이나 `다단나누기` 영역이라 #768 과 동일 근원일 가능성 — 본 타스크 흡수 권장.)
3. 부수: 제목 PUA 첫 글자 `\u{f53a}` — SVG 에는 출력되나 폰트 미지원 시 미표시(폰트 폴백 영역). 본 타스크 범위 제외 권장.
