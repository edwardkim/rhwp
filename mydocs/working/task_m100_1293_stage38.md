# Task 1293 Stage 38: 구분선위20 미주사이7 TAC 그래프 tail 묶음 보정

## 배경

Stage37에서 `2024-11-practice-above20-between7-below2`의 문26 제목과 초기 풀이가
오른쪽 단 상단으로 이동했다. 기존 `pi=914` overflow 3건은 사라졌지만,
같은 실제 19쪽 오른쪽 단 하단에 `pi=932` 7.1px overflow가 1건 남았다.

## 목적

문28 초반 수식 줄(`pi=932`) overflow를 단독으로 넘기는 대신, 18쪽/19쪽에서
처음 흐름이 갈라지는 TAC 그래프 묶음을 찾아 한컴/PDF의 미주 흐름과 맞도록
공통 pagination 조건을 보정한다.

## 분석 대상

- target: `2024-11-practice-above20-between7-below2`
- sample: `samples/3-11월_실전_통합_2024-구분선위20미주사이7구분선아래2.hwp`
- focused output: `output/task1293_stage37_focused3`
- 실제 문서 위치: 19쪽 문28 초반
- 잔여 overflow:
  - `LAYOUT_OVERFLOW: page=18, sec=0, col=1, para=932, type=FullParagraph, y=1099.4, bottom=1092.3`

## 확인 계획

1. `render_tree_019.json`에서 `pi=929~932`의 bbox와 line y를 확인한다.
2. `dump-pages -p 18`로 pagination이 문28 제목과 본문을 어느 단/쪽에 배치하는지 확인한다.
3. PDF compare image에서 문28이 현재 쪽에 남아야 하는지 다음 쪽으로 넘어가야 하는지 확인한다.
4. 7px 수준의 수식 line overflow가 실제 frame 밖 bleed인지, 수식 bbox height 과대인지 구분한다.
5. 수정 후 focused sweep으로 stage37 target과 회귀 target을 확인한다.

## 현재 확인 결과

- `output/task1293_stage37_focused3/.../analysis/annotated_019.png`를 직접 확인했다.
- 자동 overflow는 `pi=932` 7.1px 1건만 남지만, 실제 PDF와 비교하면 더 큰 흐름 차이가 있다.
  - PDF/Hancom 기준 18쪽 오른쪽 단 하단에는 문30 풀이 텍스트까지만 남고, 그래프는 19쪽 좌상단으로 넘어간다.
  - rhwp stage37은 같은 그래프(`pi=882`)를 18쪽 오른쪽 단 하단에 고아처럼 배치했고,
    이어지는 문23~문25/문26 흐름이 PDF보다 위로 당겨졌다.
- 따라서 `pi=932` 한 줄만 다음 쪽으로 넘기는 방식은 근본 해결이 아니다.

## 실패한 시도

- 새 미주 제목 head 묶음 판정을 마지막 단에도 적용해 문28을 다음 쪽으로 넘기려 했다.
- `take(3)`을 `take(4)`로 넓히고 `allow_default_question_title_tail` 예외를 무시하는 조건도 시험했다.
- 결과:
  - 문28은 그대로 남았다.
  - 오히려 `pi=800` 신규 overflow가 생겼다.
- 위 실패 코드는 모두 원복한 뒤 아래 최종 수정만 적용했다.

## 판단 전환

- 다음 수정은 문28 개별 이동이 아니라, 19쪽 이전부터 누적되는 미주 흐름 drift를 찾아야 한다.
- 우선 PDF와 rhwp의 같은 페이지에서 문항 marker 순서가 어디서 처음 갈라지는지 역추적한다.
- `line_band_drift`와 `large_ink_region_drift`가 같이 잡힌 page 19는 단순 하단 overflow가 아니라
  앞쪽 TAC 그림/그래프/미주 경계의 누적 위치 차이를 의미한다.

## 최종 원인

- `pi=882`는 텍스트가 비어 보이지만 실제로는 객체 대체 문자만 가진 TAC 그림 문단이다.
- 기존 `para_is_treat_as_char_picture_only()`는 `text.trim().is_empty()`만 확인해 이 문단을
  picture-only 문단으로 인식하지 못했다.
- 또한 단 하단에서 TAC 그림 자체 높이만 보면 남은 공간에 들어가는 것처럼 보이지만,
  한컴은 그림만 하단에 두지 않고 같은 미주 안의 뒤따르는 tail 문단 묶음과 함께 다음 쪽으로 넘긴다.

## 수정

- `para_is_treat_as_char_picture_only()`가 객체 대체 문자를 보이는 텍스트로 보지 않도록
  `!para_has_visible_text(para)` 기준으로 바꿨다.
- 다단 미주 마지막 단에서 TAC picture-only 문단의 저장 vpos가 현재 단 시작보다 앞으로 되감기고,
  `TAC 그림 + 같은 미주 tail 문단 묶음`이 남은 공간을 넘으면 다음 단/쪽으로 advance한다.
- 이 조건은 보이는 텍스트가 없는 TAC 그림/도형 문단에만 적용하므로 일반 텍스트 미주와
  새 미주 제목 이동 조건에는 영향을 주지 않는다.

## 검증

- `cargo build --bin rhwp`
- `cargo test --lib compact_endnote -- --nocapture`
  - 28 passed
- focused visual sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above20-between7-below2 --target 2024-11-practice-above20-between0-below20 --target 2024-11-practice-shape987 --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage38_focused_final --rhwp-bin target/debug/rhwp`
  - 4개 target 모두 SVG/PDF/render tree 21/21/21
  - 4개 target 모두 frame overflow 없음
- 직접 확인:
  - `compare_018.png`: rhwp 18쪽 오른쪽 단 하단에서 `pi=882` 그래프가 사라지고 PDF처럼 문30 텍스트 뒤 빈 공간으로 남는다.
  - `compare_019.png`: rhwp 19쪽 좌상단이 `pi=882` 그래프로 시작해 PDF/Hancom 흐름과 같은 방향으로 이동했다.
  - `dump-pages -p 18`: page 19 좌단이 `pi=882` TAC 그림으로 시작하고 문23~문26 흐름이 뒤로 밀렸다.
