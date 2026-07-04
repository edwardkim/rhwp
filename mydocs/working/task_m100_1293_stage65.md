# Task 1293 Stage 65: 공식 미주 모양 contract 재정의

## 목적

Stage64까지의 실험은 특정 page tail을 맞추는 수치 보정으로는 미주 문제집 흐름을 근본적으로
해결할 수 없다는 점을 확인했다. 사용자가 지적한 것처럼 먼저 한컴 공식 “미주 모양” 의미를
renderer/typeset 공통 contract로 고정해야 한다.

이번 단계는 코드 수정 전에 공식 메뉴얼 의미와 현재 rhwp 구현을 대조한다.

## 공식 의미

한컴 도움말의 미주 모양/각주 모양 설명 기준:

- `구분선 위`: 본문과 미주 구분선 사이의 간격
- `구분선 아래`: 미주 구분선과 첫 미주 내용 사이의 간격
- `미주 사이`: 앞 번호 미주 내용과 다음 번호 미주 내용 사이의 간격
- `구분선 넣기`를 끄면 선 자체는 없어지지만, 여백 값의 의미는 사라지는 것이 아니라
  미주 블록의 상단/내용 간격으로 해석해야 한다.
- `미주 위치`: 문서의 끝 또는 구역의 끝에 미주 묶음을 배치한다.

참조:

- <https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fendnote_format.htm>
- <https://help.hancom.com/hoffice130_assistant/ko-KR/Hwp/index.htm#t=insert%2Fannotations%2Fendnotes.htm>

## 현재 모델 확인

`src/model/footnote.rs`의 `FootnoteShape` 접근자는 공식 의미와 거의 맞다.

| 접근자 | 현재 의미 | 판단 |
|---|---|---|
| `separator_above_margin_hu()` | 한컴 UI "구분선 위" | 유지 |
| `separator_below_margin_hu()` | 한컴 UI "구분선 아래" | 유지 |
| `between_notes_margin_hu()` | 한컴 UI "각주/미주 사이" | 유지 |

HWPX parser 테스트도 `aboveLine`, `belowLine`, `betweenNotes` 매핑을 이미 검증한다.

## 2024-11 샘플 note-shape 값

`target/debug/rhwp dump-note-shape` 기준:

| 샘플 | 구분선 | 구분선 위 | 미주 사이 | 구분선 아래 |
|---|---|---:|---:|---:|
| `구분선없음구분선위20미주사이20구분선아래20` | 없음 | 20.0mm | 20.0mm | 20.0mm |
| `구분선위0미주사이0구분선아래0` | 있음 | 0.0mm | 0.0mm | 0.0mm |
| `구분선위0미주사이20구분선아래2` | 있음 | 0.0mm | 20.0mm | 2.0mm |
| `구분선위0미주사이7구분선아래2` | 있음 | 0.0mm | 7.0mm | 2.0mm |
| `구분선위0미주사이7구분선아래20` | 있음 | 0.0mm | 7.0mm | 20.0mm |
| `구분선위20미주사이0구분선아래20` | 있음 | 20.0mm | 0.0mm | 20.0mm |
| `구분선위20미주사이7구분선아래2` | 있음 | 20.0mm | 7.0mm | 2.0mm |
| `구분선위9미주사이8구분선아래7` | 있음 | 9.0mm | 8.0mm | 7.0mm |

따라서 parser 단계에서 공식 UI 값은 이미 구분되어 있다. 남은 문제는 layout/typeset이 이 값을
공식 의미대로 소비하지 않고, 일부 값을 LINE_SEG trailing이나 vpos gap으로 변환해 섞는 데 있다.

## 현재 구현의 문제

### 1. separator block과 note content top이 분리되어 있지 않다

`typeset.rs`는 `PageItem::EndnoteSeparator`를 만들 때 `separator_above`, `separator_below`를
넣지만, 이후 각 미주 paragraph 흐름에서는 `betweenNotes`, vpos reset, line spacing bake가
여러 조건문에 섞여 있다.

결과적으로 다음 질문을 한 곳에서 답하지 못한다.

- 구분선이 있을 때 첫 미주 content top은 어디인가?
- 구분선이 없을 때 `구분선 위`는 어디에 적용되는가?
- `구분선 아래`는 첫 미주에만 적용되는가, 각 미주 boundary에도 적용되는가?
- `미주 사이`는 line_spacing에 굽는 값인가, note block 사이의 min gap인가?

### 2. `미주 사이`가 직전 문단 line_spacing에 직접 굽힌다

현재 `typeset.rs`는 다음 미주로 넘어갈 때:

```text
prev_para.last_seg.line_spacing = between_notes
vpos_offset += pagination_gap
```

이 방식은 "미주 사이"를 line box trailing으로 바꿔 버린다. 그래서 renderer의
`HeightCursor`는 저장 vpos, 실제 콘텐츠 하단, line spacing bake를 동시에 해석해야 하고,
0/0/0·20/20/20·구분선 없음 케이스마다 보정 분기가 늘어난다.

공식 의미에 맞추려면 `between_notes`는 “앞 미주 block의 visible bottom과 다음 미주 block의
visible top 사이의 min gap”으로 다루어야 한다.

### 3. 구분선 없음 케이스의 `구분선 위`가 명시 contract로 남아 있지 않다

사용자가 추가한 샘플:

- `3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`

이 케이스는 구분선이 없더라도 `구분선 위=20mm`가 미주 블록 상단에서 보이는지 확인하기 위한
샘플이다. 현재 구현은 `endnote_has_visible_separator(shape)`에 따라 여러 경로를 바꾸지만,
“선 없음 + 위 여백 유지”라는 공식 contract를 독립적으로 표현하지 않는다.

## 다음 수정 방향

Stage66에서는 새로운 수치 보정을 넣기 전에 `typeset.rs`에 미주 spacing contract를 작은 함수로
분리한다.

예상 함수:

```text
EndnoteSpacingContract {
  has_visible_separator,
  separator_above_hu,
  separator_below_hu,
  between_notes_hu,
  top_gap_hu,
  first_content_gap_hu,
  between_note_gap_hu,
}
```

원칙:

- `top_gap_hu`: 미주 블록 상단에서 separator line 또는 첫 content까지의 간격이다.
- `first_content_gap_hu`: separator가 보이면 `구분선 아래`, separator가 없으면 필요한 경우
  `구분선 위 + 구분선 아래`의 content-start gap으로 해석한다.
- `between_note_gap_hu`: line spacing에 굽는 값이 아니라 note block 경계 min gap으로 관리한다.
- 기존 vpos 기반 보정은 이 contract가 만든 block boundary 안에서만 동작한다.

## 검증 계획

- 먼저 note shape dump로 샘플별 contract 값을 비교한다.
- 공식 UI 변경 샘플:
  - `3-11월_실전_통합_2024-구분선위0미주사이0구분선아래0.hwp`
  - `3-11월_실전_통합_2024-구분선없음구분선위20미주사이20구분선아래20.hwp`
  - 사용자가 추가한 최신 3개 샘플
- target sweep:
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-above0-between0-below0`
  - `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-no-separator-above20-between20-below20`
- focused tests:
  - `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - `cargo test --test issue_1050_footnote_serialize -- --nocapture`
  - `cargo test --lib compact_endnote`
