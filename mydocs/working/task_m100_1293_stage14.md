# Task 1293 Stage 14: shape987 새 미주 제목 tail advance 보정

## 목적

Stage13에서 `shape987` p12 내부 rewind 중복과 빈 spacer 뒤 미주 간격 중복은 해결했다.
하지만 `3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp` p14는 한컴/PDF와 달리
`문23)` 제목이 왼쪽 컬럼 하단이 아니라 오른쪽 컬럼 상단으로 넘어갔다.

이번 단계에서는 새 미주 제목 한 줄이 현재 컬럼 하단에 실제로 들어가는 경우에는 88% 강제 advance를
막아, 한컴처럼 제목 tail을 현재 컬럼에 남긴다.

## 현재 기준

- 직전 커밋: `de0e5662 task 1293: 미주 내부 rewind 분할과 빈 spacer 간격 보정`
- 기준 산출물: `output/task1293_stage13_all_threshold_check/summary.json`
- 대상 샘플: `samples/3-11월_실전_통합_2024-구분선위9미주사이8구분선아래7.hwp`
- Stage13 기준 `shape987`: `frame=[11, 14, 18, 19, 20, 21]`
- 한컴/PDF p14: `문23)` 제목과 첫 본문 줄이 왼쪽 컬럼 하단에 남고, 다음 줄부터 오른쪽 컬럼으로 이어진다.
- 기존 rhwp p14: `문23)` 제목부터 오른쪽 컬럼으로 넘어가 p14 우측 상단이 한컴보다 한 note 늦게 시작했다.

## 원인

`typeset`에는 compact 미주에서 새 미주 첫 문단이 컬럼 높이의 88% 이후에 오면 다음 컬럼으로
강제 advance하는 가드가 있다. p14의 `pi=644`는 현재 컬럼 사용량이 939.8/1001.6px라 이 가드에
걸리지만, 실제 제목 한 줄과 첫 본문 줄은 현재 컬럼 남은 공간에 들어간다.

또한 미주 번호 prefix `문`은 본문 텍스트가 아니라 렌더링 prefix로 붙기 때문에 `en_para.text`는
`"  22_11_실전 23) ③"`처럼 시작한다. 따라서 `starts_with('문')`으로 제목을 판정하면 해당
케이스를 놓친다.

## 수정 내용

- `src/renderer/typeset.rs`
  - 새 미주 첫 문단이 non-default compact 미주이고, 다음 컬럼이 남아 있고, 한 줄짜리 head가 현재
    컬럼 하단에 실제로 들어가면 `advance_for_fit`과 `advance_for_new_endnote`의 강제 advance를
    막는다.
  - 제목 여부는 `문` prefix 텍스트가 아니라 `ep_idx == 0`, 한 줄 head, visible text/equation,
    fit 가능 여부로 판단한다.

## 확인 결과

`dump-pages -p 13` 기준 p14는 아래처럼 바뀐다.

```text
단 0 ... FullParagraph[미주] pi=644 "문23）   22_11_실전 23) ③"
단 0 ... FullParagraph[미주] pi=645 "다항식 의 일반항은 "
단 1 ... pi=668 이후 계속
```

`output/task1293_stage14_shape987_final2_check/summary.json` 기준:

- 페이지 수: 21/21/21
- `frame_overflow_pages`: `[11, 18, 19, 20]`
- Stage13 대비 p14, p21 frame 후보 제거
- `question_title_text_overlap`: 없음
- `line_order_overlap`: 없음

시각 확인:

- `output/task1293_stage14_shape987_final2_check/2024-11-practice-shape987/compare/compare_014.png`
- p14에서 `문23)` 제목과 첫 본문 줄이 PDF처럼 왼쪽 컬럼 하단에 들어간다.

## 실패한 실험

`pi=853`의 `internal_rewind_split == Some(1)`을 첫 컬럼에서 허용해 p18 overflow를 줄이는 실험을
했지만, targeted sweep에서 p14/p21 frame 후보가 재발했다. 이 변경은 되돌렸다.

남은 후보:

- `shape987`: p11, p18, p19, p20 frame 후보
- p18의 `pi=853` 단일 line head split 문제는 별도 원인으로 다음 스테이지에서 다시 다룬다.

## 검증

- `cargo build --bin rhwp`: 통과
- `python3 scripts/task1274_visual_sweep.py --target 2024-11-practice-shape987 --out output/task1293_stage14_shape987_final2_check --rhwp-bin target/debug/rhwp`: 완료
- `cargo fmt --all -- --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 52개 통과
