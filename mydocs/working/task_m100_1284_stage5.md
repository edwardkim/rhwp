# task 1284 stage5: 21쪽 문25/26 drift와 q26 tail overflow 분석

## 배경

- stage4에서 `2024-09-between20` page 21 문23 제목이 PDF처럼 왼쪽 단 하단에 남도록 보정했다.
- 커밋 후 sweep을 다시 수행한 결과 frame overflow 후보는 없지만 `question=[18,21,22,23]`가 남았다.
- page 21의 문23 column drift는 사라졌고, 남은 큰 후보는 오른쪽 단의 문25/문26 y drift와 q26 tail overflow 로그다.

## 커밋 후 sweep 결과

- 명령:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
- 결과:
  - `flagged=20/24`
  - `frame=[]`
  - `red=[11,17,18,20,21,22,23]`
  - `line=[4,7,8,10,12,14,15,16,20,21,23]`
  - `question=[18,21,22,23]`

## 남은 후보

### page 21

- 문24:
  - rhwp y=266.0, PDF y=266.2
  - 시작 위치는 PDF와 일치한다.
- 문25:
  - rhwp y=593.2, PDF y=535.8
  - `+57.4px` 낮다.
- 문26:
  - rhwp y=875.8, PDF y=818.5
  - `+57.3px` 낮다.
- overflow 로그:
  - `pi=1082`, y=1096.4, bottom=1092.3, overflow=4.1px
  - `pi=1083`, y=1133.7, bottom=1092.3, overflow=41.5px

### page 22

- 문27:
  - rhwp y=214.3, PDF y=214.5
  - q26 이월 뒤 문27 시작은 이미 PDF와 거의 일치한다.
- 문28:
  - rhwp y=781.4, PDF y=856.9
  - `-75.5px` 높다.

## 현재 판단

- page 21은 문24 시작이 정확하고 문25/문26부터 약 57px 낮아진다.
- 따라서 page21 오른쪽 단 전체 이월 문제가 아니라 문24 내부 또는 문24 종료 후 문25 제목 앞 간격이 과하게 보존된 문제로 보인다.
- page22 문27은 PDF와 일치하므로 q26 tail의 page22 이월 자체는 크게 틀어지지 않았다.
- page22 문28 drift는 page22 단 내부에서 문27 풀이 높이가 PDF보다 작게 계산된 별도 후보일 가능성이 있다.

## 다음 분석

- `src/renderer/layout.rs`의 `prev_endnote_title_gap_px` 보존 조건이 문24→문25, 문25→문26 사이에 과적용되는지 확인한다.
- page21 render tree의 `pi=1059..1083` bbox와 PDF bbox를 비교해 어느 문단부터 누적 차이가 생기는지 고정한다.
- 수정은 page21 문25/문26 drift와 q26 tail overflow를 먼저 줄이고, page22 문28은 별도 원인인지 후속으로 판단한다.

## 원인 확인

- 최초 가설처럼 문항 사이 line_spacing만의 문제는 아니었다.
- PDF bbox와 render tree를 비교하니 문24 제목 `pi=1059`는 rhwp/PDF 모두 y≈266px로 일치했다.
- 차이는 같은 문항의 첫 본문 `pi=1060`에서 시작했다.
  - 수정 전 rhwp: y=341.7
  - PDF: y≈294
- `RHWP_VPOS_DEBUG=1 RHWP_DEBUG_TAC_CURSOR=1` 로그에서 `pi=1060`은 직전 문항 제목 `pi=1059` 뒤의 저장 vpos forward jump를 그대로 받아 y=341.7로 이동했다.
- 한컴/PDF는 큰 `미주 사이` 문서에서도 "문항 제목 → 같은 문항 본문" 사이에는 저장 vpos의 큰 forward gap을 그대로 쓰지 않고, 작은 제목-본문 간격으로 이어준다.

## 수정 내용

- `src/renderer/height_cursor.rs`
  - compact 미주 흐름에서 직전 문단이 문항 제목이고 현재 문단이 같은 문항 본문인 경우, 저장 vpos forward jump가 32~120px 범위로 과하게 벌어지면 제목-본문 gap을 10~18px로 cap한다.
  - cap으로 줄인 delta만큼 활성 vpos base도 이동해 후속 문단이 다시 저장 vpos gap을 복원하지 않게 했다.
- `tests/issue_1139_inline_picture_duplicate.rs`
  - page21 문24 본문 첫 줄이 PDF처럼 y≈294px에서 시작하는지 추가로 고정했다.
  - 문25/문26 제목 범위를 기존 넓은 허용치에서 각각 PDF 근처로 조였다.
  - 문26 tail 마지막 줄이 page21 오른쪽 단 frame 안에서 끝나는지 `max_para_content_bottom(pi=1083)`로 고정했다.

## 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate issue_1284_2024_between20 -- --nocapture`
  - 3 passed
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 54 passed
- `cargo fmt --check`
  - passed
- `cargo build`
  - passed
- `python3 scripts/task1274_visual_sweep.py --target 2024-09-between20`
  - `question=[18,21,22,23]`
  - page21 문25/문26 drift 후보가 제거됐다.
  - `LAYOUT_OVERFLOW` / `LAYOUT_OVERFLOW_DRAW` 로그는 없다.
  - page21 주요 bbox:
    - 문24 y=266.0
    - 문25 y=545.5 (PDF y=535.8)
    - 문26 y=828.1 (PDF y=818.5)
    - 문27 y=214.3 on page22 (PDF y=214.5)

## 남은 후보

- page18:
  - 문29 +79.1px
  - 문23 +78.4px
  - 문30 +71.9px
- page21:
  - 왼쪽 단 문30 +53.6px
- page22:
  - 문28 -75.5px
- page23:
  - 문30 -51.8px

## 상태

- page21 문25/문26 drift와 q26 tail overflow는 해소됐다.
- 다음 stage에서는 sweep에 남은 page22 문28 drift 또는 page18 묶음 drift 중 실제 시각 차이가 큰 항목을 골라 별도로 분석한다.
