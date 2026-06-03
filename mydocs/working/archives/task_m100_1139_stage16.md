# Task M100 #1139 Stage 16

## 목적

Stage 15 커밋 이후에도 `3-09월_교육_통합_2022.hwp` 후반 미주 페이지 경계가 한컴오피스 정답지와 다른 문제를 분석한다.

## 작업지시자 기준

- 첫 번째 스크린샷이 한컴오피스 정답지다.
- 두 번째 스크린샷이 rhwp-studio 결과다.
- 격자 설정은 동일하다.
  - 격자 간격: 가로 `3.00mm`, 세로 `3.00mm`
  - 격자 기준 위치: `종이`
  - 가로 `9.00mm`, 세로 `24.00mm`

## 커밋 기준

Stage 15는 다음 커밋으로 고정했다.

- `b0aca563 task 1139: 미주 사이 간격 기준 보정`

## 현상

한컴오피스 기준 13쪽 오른쪽 단의 `문20) 80` 풀이는 `두 함수 f(x)=...` 대목까지만 남고, 다음 쪽 머리말 아래에서 `g(x)=4|x|+k...` 내용이 이어진다.

rhwp-studio 기준 13쪽 오른쪽 단에는 `g(x)=4|x|+k...`와 그 다음 줄까지 이미 들어가며, 14쪽은 `x>0일 때 ...` 근처부터 시작한다. 즉 rhwp가 한컴보다 13쪽 오른쪽 단에 더 많은 줄을 남긴다.

## 재현 명령

- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 12`
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 13`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_boundary_probe -p 12 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_boundary_probe -p 13 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp export-text samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_text -p 12`
- `target/debug/rhwp export-text samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_text -p 13`

## 관찰

`dump-pages -p 12` 결과:

- 13쪽 왼쪽 단 마지막:
  - `FullParagraph[미주] pi=708 vpos=457625 "문17)   16"`
  - SVG 렌더 로그: `pi=708 line=0 y=1123.2`, 본문 하단 `1092.3`, overflow `30.9px`
- 13쪽 오른쪽 단 마지막:
  - `FullParagraph[미주] pi=734 "문20)   80"`
  - `pi=735 "에서"`
  - `pi=737 "이므로 에서 또는"`
  - `pi=738 "이때 함수 의 증가와 감소를 표로 나타내면 다음과 같다."`
  - `pi=740 vpos=523404..511992 "따라서 함수 는 ... 두 함수 , 의 ..."`
  - SVG 렌더 로그: `pi=740 line=4 y=1105.5`, 본문 하단 `1092.3`, overflow `13.2px`

`export-text -p 12` 결과를 보면 rhwp 13쪽 텍스트에 다음 내용까지 포함된다.

- `의 그래프가 만나는 점의 개수가 이기 위해서는`
- `그림과 같이 인 부분에서 두 함수 ,`
- `의 그래프가 접해야 한다.`

작업지시자 한컴오피스 스크린샷에서는 이 대목이 다음 쪽으로 넘어가야 한다.

`dump-pages -p 13` 결과:

- rhwp 14쪽은 `pi=741 "(빈)"`, `pi=742 "일 때 이므로"`부터 시작한다.
- 한컴오피스 기준 14쪽은 머리말 아래 `g(x)=4|x|+k...`부터 시작한다.

## 현재 추정

Stage 15까지의 `미주 사이`/`구분선 아래` 해석은 페이지 수와 9쪽 기준에는 맞지만, 후반 미주에서 문단 내부 줄 분할이 부족하다.

일반 본문은 `typeset_paragraph()`가 `PartialParagraph { start_line, end_line }`로 줄 단위 분할을 수행한다. 반면 현재 미주 삽입 루프는 endnote paragraph를 `PageItem::FullParagraph`로만 추가하고, `en_fit`이 현재 단에 맞지 않을 때 문단 전체를 다음 단/쪽으로 넘기는 수준이다. 이 때문에 `pi=740`처럼 한 문단 내부의 앞 줄은 13쪽에 남고 뒤 줄은 14쪽으로 넘어가야 하는 케이스가 통째로 13쪽에 렌더되어 overflow가 발생한다.

## 구현 내용

`src/renderer/typeset.rs`에 미주 전용 줄 분할 경로를 추가했다.

1. 미주 paragraph에서 내부 `LINE_SEG.vertical_pos`가 앞 줄보다 작아지는 지점을 찾는다.
   - `pi=740`은 원본 lineSeg가 `14266, 16788, 0, 1352, 2854` 형태로 3번째 줄에서 0으로 되감긴다.
   - 이는 한컴이 같은 미주 문단을 다음 쪽/단에서 이어 그리는 신호로 해석할 수 있다.
2. 모든 vpos 되감김을 split하지 않는다.
   - `pi=522`처럼 내용이 없는 빈 미주 문단도 내부 vpos 되감김이 있어, 이를 분할하면 9쪽 이후 pagination이 과하게 흔들린다.
   - 따라서 가시 텍스트가 있고 단 하단 근처(`current_height > available * 0.75`)인 미주 paragraph에만 적용한다.
3. 미주 전용 `typeset_split_endnote_paragraph()`를 추가해 `PageItem::PartialParagraph`를 생성한다.
   - `pi=740`은 13쪽 `lines=0..2`, 14쪽 `lines=2..5`로 분리된다.
   - 일반 본문용 `typeset_paragraph()`를 직접 재사용하지 않고, 기존 미주 vpos 누적/단 전환 상태를 유지한다.
4. 회귀 테스트에 `pi=740` split 경계를 추가했다.

## 검증 목표

- `3-09월_교육_통합_2022.hwp` 전체 23페이지 유지
- 13쪽 오른쪽 단에서 `g(x)=4|x|+k...`가 다음 쪽으로 넘어감
- 14쪽 머리말 아래 시작 위치가 한컴오피스 스크린샷과 가까워짐
- 9쪽 `문8)` 10쪽 시작, `[다른 풀이]` 표시, 개체 속성 진입 회귀 없음
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
- `wasm-pack build --target web --out-dir pkg`

## 검증 결과

- `cargo fmt --check`
- `cargo build`
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 7 passed
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`
  - 23페이지 유지
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 12`
  - `PartialParagraph pi=740 lines=0..2`
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 13`
  - `PartialParagraph pi=740 lines=2..5`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_endnote_split -p 12 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_endnote_split -p 13 --show-grid=3mm --grid-origin=9mm,24mm`
- `wasm-pack build --target web --out-dir pkg`

## 작업지시자 추가 피드백

자동 검증 후 작업지시자가 10쪽, 13쪽 등 다른 페이지도 한컴오피스와 다르다고 확인했다.

추가 스크린샷을 기준으로 보면 차이는 `pi=740` 뒷부분만의 문제가 아니라 `문20)` 앞의 `문19)` 말미가 어느 쪽/단에 남는지부터 누적되어 있다. 따라서 `pi=740`만 `PartialParagraph`로 분할하는 Stage 16 1차 보정은 원인보다 뒤쪽을 건드리는 특수 처리로 판단하고 폐기한다.

## 2차 시도 결과

미주 조판 루프의 `internal_vpos_rewind` 처리에서 최소 높이를 첫 줄 높이로 낮추면, 내부 `LINE_SEG.vertical_pos` 되감김이 있는 다줄 미주 paragraph가 지나치게 낮은 높이로 fit 판정된다. 이 경우 후반 미주가 한컴보다 앞쪽 페이지/단에 더 많이 들어가 누적 페이지 경계가 달라진다.

이에 따라 다음처럼 수정하는 방향을 검토했다.

- `pi=740` 전용 내부 줄 분할 경로 제거
- 내부 `vpos` 되감김이 있어도 fit/advance 최소 높이는 정상 `height_for_fit`을 사용
- 9쪽 `문8)` 시작, 2022 원본 23쪽, `미주사이20` 24쪽 기준이 유지되는지 다시 검증

하지만 실제 검증 결과 이 2차 방향은 회귀를 만들었다.

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 5 passed
  - 2 failed
  - 실패: 원본 `3-09월_교육_통합_2022.hwp` 페이지 수가 24쪽으로 증가
  - 실패: `구분선 아래 20mm` 기준 파일 페이지 수가 24쪽으로 증가

따라서 2차 방향은 폐기하고, Stage 15의 `internal_vpos_rewind` 최소 높이 처리는 유지한다.

재확인 결과:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 7 passed
- `cargo build`: 성공
- 새 `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지

## 상태

Stage 16의 `pi=740` 전용 분할 및 2차 최소 높이 변경은 모두 폐기한다. 현재 소스는 Stage 15 기준을 유지하고, 후반 페이지 경계 차이는 다음 단계에서 더 앞쪽 누적 원인을 다시 찾는다.

## 3차 분석: 내부 vpos 되감김 일반화

작업지시자에게 요청한 비교 샘플 중 `문20)` 앞에 강제 텍스트 줄을 넣은 한컴오피스 화면을 확인했다. 화면상 파일명은 `3-09월_교육_통합_2022-stage16-문20앞강제줄.hwp`였으나, 현재 macOS 작업 경로와 repo `samples/`에서는 아직 파일이 발견되지 않았다.

샘플 화면의 의미는 `문20)` 직전 줄 추가가 오른쪽 단 하단 경계를 직접 밀어내는지 확인하는 것이다. 이에 따라 `pi=740` 전용 처리가 아니라 다음 조건을 만족하는 미주 문단만 부분 분할하도록 다시 좁혔다.

- 다단 미주 문단이다.
- 문단 내부 `LINE_SEG.vertical_pos`가 감소하는 지점이 있다.
- 문단에 보이는 텍스트가 있다.
- 되감김 전 앞부분은 현재 단에 들어간다.
- 문단 전체 렌더 높이(`fmt.total_height`)는 현재 단 하단 안전 여백을 넘는다.

이 조건이면 빈 내부 되감김 문단(`pi=522`, `pi=992`)은 제외되고, 하단에서 실제 렌더 높이가 넘치는 문단만 `PartialParagraph`로 나뉜다. 자동 검증 후 작업지시자 시각 검증을 대기한다.

## 3차 검증 결과

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 7 passed
- `cargo build`: 통과
- `wasm-pack build --target web --out-dir pkg`: 통과
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp`: 23페이지 유지
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 12`
  - `PartialParagraph  pi=740  lines=0..2`
- `target/debug/rhwp dump-pages samples/3-09월_교육_통합_2022.hwp -p 13`
  - `PartialParagraph  pi=740  lines=2..5`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_endnote_split_generalized -p 12 --show-grid=3mm --grid-origin=9mm,24mm`
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_endnote_split_generalized -p 13 --show-grid=3mm --grid-origin=9mm,24mm`

남은 자동 관찰:

- 13쪽 왼쪽 단 `pi=708`에는 기존 overflow 로그가 남아 있다.
- 이번 3차 수정은 오른쪽 단 `문20)` 내부 되감김 분할을 일반화한 것이므로, 왼쪽 단 경계 차이는 작업지시자 시각 검증 결과에 따라 후속 보정한다.

## 작업지시자 추가 피드백: 10쪽 경계 불일치

작업지시자 한컴오피스 화면과 rhwp-studio 화면 비교 결과, 13쪽뿐 아니라 10쪽에서도 이미 경계가 다르다.

- 한컴오피스 기준 10쪽 왼쪽 단은 `문10)` 풀이 끝부분에서 멈춘다.
- rhwp 기준 10쪽 왼쪽 단에는 `문11)` 일부가 추가로 들어간다.
- rhwp 기준 10쪽 오른쪽 단도 한컴보다 더 많은 풀이가 들어간다.

`export-svg -p 9 --debug-overlay` 확인 결과, page 10에서 실제 렌더가 이미 하단을 초과한다.

- 왼쪽 단: `pi=560`, `pi=561`, `pi=562` overflow
- 오른쪽 단: `pi=590`, `pi=591` overflow

`RHWP_DEBUG_TAC_CURSOR=1` 확인 결과, renderer의 실제 항목 진행 높이는 `TAC_CURSOR`에서 `dy`로 정상 누적된다. 문제는 typeset 미주 루프가 일부 단 전환에서 `current_height`를 음수로 시작시키는 보정 때문에 page item을 더 많이 담는 데 있다.

검토한 실패 방향:

- 미주 `vpos` 되감김 묶음 단 전환 시 `reclaimed` 보정을 완전히 제거
  - 10쪽 경계는 일부 가까워졌지만 원본 2022가 24페이지로 증가
  - 테스트 실패
- `reclaimed * 0.5`로 절반만 보정
  - 원본 2022가 여전히 24페이지로 증가
  - 테스트 실패

따라서 `reclaimed` 보정 제거/감쇠는 폐기하고 원래 값으로 복구했다. 현재 자동 검증은 다시 통과한다.

## 4차 시도: page 10 overflow와 page 13 경계 동시 보정

작업지시자 승인 후 page 10 overflow를 직접 줄이는 방향을 추가 검토했다.

확인한 `reclaimed` 발생 지점:

- `before_pi=523`, 이전 page 9 오른쪽 단에서 page 10 왼쪽 단으로 넘어가는 지점
- `before_pi=1129`, 후반 긴 미주 묶음 경계

실패한 실험:

- 새 페이지로 넘어가는 `before_pi=523`에서만 `reclaimed`를 버림
  - page 10은 일부 가까워졌지만, page 13에서 `문20)`이 한컴 기준보다 뒤로 밀림
  - 기존 `pi=740 lines=0..2` 검증 실패
- `current_height`는 압축한 채 fit 판정에 음수 보정 부채를 별도로 유지
  - page 10 overflow를 줄이는 방향이지만, 역시 page 13 `문20)` 경계를 뒤로 밀어 실패

결론:

- page 10 overflow는 단순히 `reclaimed`를 제거하거나 fit 판정을 보수화하는 문제만은 아니다.
- `reclaimed`는 후반부 23쪽 유지와 page 13 `문20)` 위치에 영향을 주므로, page 10만 따로 고치면 뒤쪽 경계가 깨진다.
- 현재 통과 상태는 유지하고, 다음 원인 후보는 미주 paragraph 자체의 실제 렌더 높이와 pagination 누적 높이 차이다.

현재 자동 검증:

- `cargo fmt --check`: 통과
- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`: 7 passed

## 5차 재분석: 미주 묶음 시작과 내부 vpos 되감김

작업지시자 추가 피드백에 따라 10쪽과 13쪽을 다시 확인했다.

실패한 방향:

- `ColumnContent.start_height`로 renderer 시작 y를 음수 보정하는 방향은 자동 테스트는 통과했지만 한컴오피스 화면과 더 멀어졌다. 이 변경은 폐기했다.

확인한 핵심은 세 가지다.

1. `미주 사이` 값은 원본 `3-09월_교육_통합_2022.hwp`의 기본 7mm가 이미 LINE_SEG/vpos 흐름에 상당 부분 녹아 있다. 이전 보정은 직전 미주 paragraph의 마지막 `line_spacing`을 UI 값 전체로 바꾸어 renderer에만 추가 간격이 붙는 중복을 만들 수 있었다.
2. 10쪽 왼쪽 단에서 `문11)` 새 미주 묶음이 단 하단에 걸쳐 시작하면서 실제 렌더 높이가 하단을 넘었다. 한컴은 이처럼 새 미주 묶음 전체가 남은 공간보다 충분히 클 때 다음 단으로 넘기는 것으로 보인다.
3. `문13)` 내부에는 `LINE_SEG.vertical_pos`가 큰 폭으로 되감기는 paragraph가 있고, 이 되감김은 다음 단/쪽 continuation 신호로 처리해야 한다. 다만 너무 이르게 넘기면 뒤쪽 13쪽 경계가 깨지므로 75~85% 구간에서는 남은 높이를 `reclaimed`로 보존한다.

이에 따라 다음처럼 수정했다.

- `미주 사이`는 전체 UI 값을 마지막 줄 spacing으로 덮어쓰지 않고, 기본 흐름 7mm를 초과하는 값만 `vpos_offset`과 렌더 `line_spacing`에 더한다.
- 새 미주 묶음 시작 시 현재 단이 80% 이상 차 있고, 묶음 전체 렌더 높이의 부족분이 단 높이의 10%를 넘으면 다음 단/쪽으로 넘긴다. 이때 남은 단 높이는 `reclaimed`로 보존해 전체 23쪽 흐름을 유지한다.
- 같은 미주 묶음 내부 `vpos` 되감김은 75% 이후부터 다음 단/쪽으로 넘긴다. 단, 75~85% 구간에서는 동일하게 `reclaimed`를 적용해 뒤쪽 경계가 과도하게 밀리지 않게 한다.
- fit 판정에서는 음수 `current_height`가 단에 내용을 과도하게 허용하지 않도록 `current_height.max(0.0)` 기준으로 비교한다.

## 5차 검증 결과

- `cargo test --test issue_1139_inline_picture_duplicate -- --nocapture`
  - 7 passed
- `cargo build`
  - 통과
- `cargo fmt --check`
  - 통과
- `wasm-pack build --target web --out-dir pkg`
  - 통과
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_verify_page10 -p 9 --show-grid=3mm --grid-origin=9mm,24mm --debug-overlay`
  - 23페이지 유지
  - 10쪽 SVG 생성 완료
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_verify_page13 -p 12 --show-grid=3mm --grid-origin=9mm,24mm --debug-overlay`
  - 13쪽 SVG 생성 완료
- `target/debug/rhwp export-svg samples/3-09월_교육_통합_2022.hwp -o output/task1139_stage16_rewind_reclaimed -p 9 --show-grid=3mm --grid-origin=9mm,24mm --debug-overlay`
  - 10쪽 왼쪽 단은 `문10)`까지 남고 `문11)`은 오른쪽 단에서 시작
  - 10쪽 SVG에서 하단 overflow 로그 없음

현재 상태는 자동 검증 통과 및 WASM 갱신 완료다. 다만 최종 완료 여부는 작업지시자의 한컴오피스 기준 시각 검증 후 판단한다.

## 6차 재분석: 12쪽 문14 시작 위치

작업지시자 시각 검증에서 12쪽 하단 페이지가 한컴오피스와 다시 다르다는 피드백을 받았다.

증상:

- rhwp-studio는 12쪽을 `문14)`로 바로 시작했다.
- 한컴오피스는 직전 `문13)` 미주 후반의 `그러므로/따라서` 흐름이 먼저 이어진 뒤 `문14)`가 시작한다.

`dump-pages`로 확인한 현재 경계:

- 기존 rhwp:
  - 11쪽 오른쪽 단 끝에 `pi=640..645`까지 포함
  - 12쪽 왼쪽 단은 `pi=646 문14)`부터 시작
- 한컴 기준:
  - 11쪽 오른쪽 단은 `pi=639`까지
  - 12쪽 왼쪽 단에서 `pi=640..645`가 먼저 이어진 후 `pi=646 문14)` 시작

원인 판단:

- `문13)` 내부에는 `LINE_SEG.vertical_pos`가 큰 폭으로 되감기는 구간이 여러 번 있다.
- 첫 단 중간의 되감김은 유지되어야 하지만, 마지막 단 후반의 되감김은 한컴에서 다음 쪽 continuation으로 처리한다.
- 기존 조건은 모든 단에서 75% 이후만 다음 단/쪽으로 넘겼기 때문에 마지막 단 후반 `pi=640` 되감김(`current_height≈722px`, `available≈1002px`)이 11쪽에 남았다.

수정:

- 다단 미주에서 `local_vpos_rewind`가 마지막 단에서 발생한 경우 임계값을 70%로 낮췄다.
- 첫 단은 기존 75% 조건을 유지해 앞쪽 `문13)` 흐름을 과도하게 밀지 않는다.
- 회귀 테스트에 11쪽 `pi=639` 유지, 12쪽 `pi=640` 이후 `pi=646 문14)` 시작 조건을 추가했다.
