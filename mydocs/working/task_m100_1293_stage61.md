# Task 1293 Stage 61: 0mm 미주 문12/문13 흐름 정합

## 목적

Stage60에서 `2024-11-practice-above0-between0-below0` target의 hard overflow는 제거했고,
qflow 후보도 10, 18쪽이 빠졌다. 하지만 11쪽은 실제 PDF/Hancom 흐름과 다르다.

PDF 기준:

- 왼쪽 단 하단에 문12 풀이 tail 뒤 `문13） 22_11_실전 13) ③` 제목이 남는다.
- 오른쪽 단은 문13 본문 뒤 문14가 이어진다.

Stage60 rhwp 기준:

- `pi=539` 문13 제목이 왼쪽 단에 남으면 renderer VPOS 기준으로 frame 밖에 내려간다.
- hard overflow를 막기 위해 첫 단에서는 `line_advance`까지 들어갈 때만 0mm 제목 tail을 허용했고,
  그 결과 문13 제목이 오른쪽 단 상단으로 이동했다.

이번 stage에서는 문13을 왼쪽 단에 남기되 frame 밖으로 밀리지 않게 만드는 공통 원인을 분석한다.
단순 tolerance 확대나 문항 번호 특수처리는 하지 않는다.

## 확인 대상

- 샘플: `samples/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
- 기준 PDF: `pdf/3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.pdf`
- 기존 산출물:
  - `output/task1293_stage60_zero_line_advance_guard/.../compare/compare_011.png`
  - `output/task1293_stage60_zero_line_advance_guard/.../render_tree/render_tree_011.json`

## 분석 계획

1. 문12 내부 `pi=526..538`의 renderer y와 저장 vpos를 PDF 이미지의 실제 배치와 비교한다.
2. `pi=537` non-TAC 그림/그래프와 `pi=538` 수식 line box가 한컴보다 과도하게 큰지 확인한다.
3. pagination `current_height`와 renderer VPOS y의 차이가 어느 문단에서 처음 벌어지는지 찾는다.
4. 보정은 미주 모양 공통 규칙으로 제한한다.
   - `구분선 위/미주 사이/구분선 아래=0/0/0`
   - 보이는 구분선
   - 첫 단 제목 tail
   - 직전 문단이 non-TAC 그림 또는 tall equation tail인 경우의 VPOS/line box 정합

## 검증 계획

- `cargo build --bin rhwp`
- target sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage61_zero_qflow --rhwp-bin target/debug/rhwp`
- focused regression:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - `cargo test --lib compact_endnote`

## 분석 결과

### 현재 Stage60 기준

- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0 --out output/task1293_stage61_keep_page_base --rhwp-bin target/debug/rhwp`
  실험 전/후 비교에서 page count는 21/21/21로 유지되지만, page 11 qflow 후보는 유지된다.
- `dump-pages -p 10` 기준 page 11 첫 단은 `pi=538`까지 배치되고 `pi=539` 문13 제목은
  둘째 단으로 이동한다.
- PDF/Hancom 기준 page 11 첫 단 하단에는 `pi=538` 뒤 `문13） 22_11_실전 13) ③` 제목이 남는다.

### VPOS 기준 확인

- `RHWP_VPOS_DEBUG=1` 로그에서 page 11 첫 단은 `pi=514`까지 `path=page base=150253`을 쓰다가
  `pi=516`부터 `path=lazy base≈1475xx`로 전환된다.
- 이 전환 이후 문12 후반부가 PDF보다 약 30px 낮게 누적된다.
- render tree 기준:
  - Stage60 `pi537` 그림 bbox: `y≈898.0`, `h≈155.9`
  - Stage60 `pi538` tail bbox: `y≈1072.0`, `h≈35.9`
  - PDF 하단 그림 후보는 대략 `y≈868`, `h≈151`로 검출된다.
- 즉 문13 제목만 강제로 현재 단에 남기는 것이 아니라, 문12 후반 그림/tail의 y 기준 자체가
  한컴보다 낮게 잡히는 원인을 해결해야 한다.

## 폐기한 실험

1. `layout.rs`의 TopAndBottom post-jump에서 0/0/0 미주 page-base를 보존하는 실험
   - 결과: `pi537` 그림과 `pi538` tail y가 변하지 않았다.
   - 판단: page-base 유지 가드만으로는 실제 lazy 기준 전환 원인을 해결하지 못한다.
2. `typeset.rs`의 0/0/0 새 문항 제목 tail 조건에서 `line_advance(0)` fit 요구를 제거하는 실험
   - 결과: `pi539`가 첫 단에 배치되지만 renderer에서 `LAYOUT_OVERFLOW_DRAW`가 발생한다.
   - 대표 로그: `pi=539 y=1125.9 col_bottom=1092.3 overflow=33.6px`
   - 판단: pagination만 완화하면 실제 렌더가 frame 밖으로 나간다.
3. `height_cursor.rs`에서 0/0/0 새 문항 제목을 column bottom-fit으로 당기는 실험
   - 결과: hard overflow는 사라지지만 `pi538` 수식 tail bbox와 `pi539` 제목이 겹친다.
   - 판단: 제목만 당기는 것은 overwrap을 만든다.
4. `layout.rs`의 빈 TopAndBottom 그림 문단 line advance 추가(#683)를 0/0/0 미주에서 생략하는 실험
   - 결과: `pi538` tail은 `y≈1072`에서 `y≈1054`로 올라가고 hard overflow는 0건이 되지만,
     `pi539` 제목이 여전히 frame 하단 밖으로 걸린다.
   - 판단: #683 한 줄 추가만이 전부는 아니며, lazy base 또는 그림/tail 높이 진행량을 함께 봐야 한다.

## 다음 단계

Stage62에서는 코드 변경 없이 다음 두 축을 먼저 좁힌다.

1. `pi516`에서 lazy base가 `1475xx`로 내려가는 원인
   - `pending_topbottom_post_jump`가 어떤 anchor paragraph/control에서 만들어지는지 추적한다.
   - 0/0/0 미주에서 이 anchor가 저장 vpos 기준을 한컴보다 낮게 잡는지 확인한다.
2. `pi537` 비TAC 그림의 PDF 대비 y/height 차이
   - `layout_body_picture`가 반환하는 `result_y`, 실제 image bbox, common height/margin bottom을 로그로 비교한다.
   - #683 line advance 추가가 필요한 일반 문서와 0/0/0 미주 하단 그림 문단을 분리할 수 있는
     공통 조건을 찾는다.

이번 stage의 결론은 “문13 제목을 강제로 현재 단에 남기는 패치는 폐기”이다. 문12 tail 기준을
먼저 한컴/PDF에 맞춰야 문13 제목도 overwrap 없이 현재 단 하단에 남는다.
