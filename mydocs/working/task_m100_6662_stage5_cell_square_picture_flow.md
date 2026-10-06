---
kind: working
status: completed
canonical: mydocs/working/task_m100_6662_stage5_cell_square_picture_flow.md
last_verified: 2026-09-06
---

# 열린 이슈 재검증 5단계: 셀의 저장 어울림 그림

## 분석

- Issue: #6712, 상위 현황 #6662. 직전 단계 `247de8ab9`는 셀 분할의 동일 글줄 조각을 묶었다.
- 한국어/중국어 공개 원본은 수정 전 rhwp 3쪽, 보존한 한컴 PDF는 각각 2쪽이다. 한국어
  기준 PDF는 한컴 2022, 중국어 기준 PDF는 한컴 2024로 확인했다. 이슈 본문과 정정
  코멘트를 다시 읽었다. 최상위 typeset에 파일명·페이지 번호 가드를 추가하는 방식은
  사용하지 않는다.
- 셀 렌더러의 `cell_has_square_float`는 저장 `column_start`/`segment_width`를 이미 사용한다.
  그러나 `cell_units_uncached`는 Square 그림 높이를 뒤의 별도 flow unit에 더하고,
  분할 셀의 `fragment_owned_square_flow`는 그림을 그 높이 단위의 위치에 놓은 뒤 다시 전진한다.
- 원문 dump에서 p8/p9와 p10은 그림의 저장 사각형 옆을 실제 텍스트 LINE_SEG가 사용한다.
  p18~p25는 p17 중첩 표와 p26 가시 문단 사이의 빈 문단 run이며, p18~p24의 실제
  `column_start`는 41249HU이다. 이 run을 일반 줄높이로 계속 세면 첫 물리 쪽에서
  약 166px가 추가되어 p33 이후 본문과 p63~p70 꼬리가 다음 쪽으로 밀린다.
- 이번 단계는 **Picture와 그 그림 때문에 생긴 셀 RowBreak 원장 중복**만 다룬다.
  중첩 표 자체의 높이 산식은 새로 변경하지 않는다. 원본·PDF는 이미 보존한
  `samples/issue6712/` 및 `pdf/issue6712/`를 재사용한다.

## 수정 계획과 경계

1. 같은 셀에서 그림의 저장 사각형 옆을 실제로 지나는 다음 글줄이 있는지 판정한다.
   원본 비합성 LINE_SEG, 문단 상대 상단/왼쪽 정렬, Square 및 flow-with-text를 요구한다.
   같은 cs/sw라는 이유만으로 개체의 세로 범위를 무시하지 않는다.
2. 위 증거가 있는 그림만 독립적인 세로 flow 합산에서 제외한다. 개체 시각 하단으로 셀
   높이 하한을 지키는 기존 경로는 보존한다.
3. 그림은 호스트 문단의 첫 글줄과 같은 source unit이 소유하게 하여 페이지마다 반복되거나
   높이를 제거한 뒤 통째로 사라지는 것을 막는다. 저장된 signed offset도 보존한다.
4. 합성 문서로 옆 글줄 전진/그림 중복/분할 소유를 재현하고, 옆 글줄 없음·full-width 줄·
   synthetic seg·TopAndBottom은 제외 반례로 확인한다.
5. 관련 기존 계약과 실물 첫 제목/그림을 확인한 뒤 분석보고를 완성하고 일반 커밋한다.

전체 회귀·native/WASM/workspace lint와 최종 visual sweep 통과 전에는 PR 또는 #6712 종료를
진행하지 않는다. 로그·중간 SVG/PNG/JSON은 임시 경로에만 남기고 커밋하지 않는다.

## 분석보고

### 원인 확정

- 기존 focused 실행은 합성 계약 6개는 통과했지만 실제 한국어·중국어 HWP가 각각
  3쪽으로 남아 8개 중 6개 통과였다. 단순히 빈 Picture anchor 줄의 높이와 continuation
  패딩만 제거해서는 p63~p70의 footer만 남는 3쪽 상태가 해소되지 않았다.
- 기준 PDF의 1쪽에는 p33~p40의 예방 안내까지 포함되고 2쪽에는 p63~p70의 출처·서명
  영역이 포함된다. 수정본 tree에서 1쪽 뒤에는 p33~p62만, 2쪽에는 footer만 남았으므로
  p18~p25 빈 문단 run이 첫 쪽의 셀 예산을 과다 점유하는 것이 남은 공통 원인이었다.
- 이 빈 run은 `text/control`이 비어 있고 실제 LINE_SEG가 하나씩만 있으며, run 앞은
  중첩 표이고 뒤는 가시 문단이다. 동시에 같은 셀에서 저장 Square 그림과 인접
  LINE_SEG가 확인된다. 이 네 가지 구조 증거가 없는 빈 문단은 축약 대상이 아니다.

### 구현

- `height_measurer.rs`에 저장 Square 그림과 같은 세로 band의 좌우 LINE_SEG를 판정하는
  공통 helper를 추가했다. `flow_with_text`, `Square`, 문단·그림 상대 좌표, 비합성
  LINE_SEG와 그림 경계의 실제 교차를 모두 요구한다. 이 helper는 셀 높이와 paint 경로가
  공유한다.
- `table_layout.rs`에서 셀 문단 compose에 저장 wrap anchor를 전달하고, 그림의
  `CellUnit` owner를 실제 source line에만 붙였다. 인접 저장 줄이 그림 높이를 이미
  소유한 경우 별도 Square flow height와 renderer cursor 전진을 중복하지 않는다.
- 저장 HWP5 RowBreak 셀에서만, 같은 셀의 저장 Square 흐름·중첩 표 직후·다음 가시 문단
  전·후·비합성 좁은 LINE_SEG를 모두 만족하는 연속 빈 문단 run을 0 높이 spacer로
  기록한다. 일반 빈 줄, 합성 LINE_SEG, 명시적 문단 나눔, 텍스트가 섞인 문단은 보존한다.
  continuation에서는 같은 row padding을 다시 예약하지 않는다.
- `table_partial.rs`와 `typeset.rs`의 RowBreak 경로도 동일한 helper 결과를 사용해
  저장 그림의 셀 행 높이와 분할/페인트 계산이 서로 다른 기준을 쓰지 않게 했다.
- 테스트에는 한국어·중국어 실제 HWP oracle을 추가하고, 기존 합성 테스트의 좌우 조각
  동시 소유·행 컷·합성 반례를 유지했다.

### 기준 자료

| 자료 | 경로 | 확인 결과 |
| --- | --- | --- |
| 한국어 기준 PDF | `pdf/issue6712/한국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방-2020.pdf` | `Creator: Hwp 2022 0.0.0.0`, 2쪽, A4, SHA-256 `ff0ab5e0cc70c4104d9dae960be01f736c0bf7a6d9a394212efb860dda1bdfd8` |
| 중국어 기준 PDF | `pdf/issue6712/중국어_2026년 8호 가정통신문_여름철 영유아 감염병 예방-2024.pdf` | `Creator: Hwp 2024 0.0.0.0`, 2쪽, A4, SHA-256 `aa85b871b5d8049af5bd8240fa210678a48ebe4b258fe6497a528de234cb6f97` |

### 단계 검증

- 최종 focused 명령:

  ```bash
  cargo nextest run --cargo-profile release-test \
    --target-dir target/pr-review \
    --test regression_suite_014 issue_6712_cell_visual_row_units \
    --test-threads 12 --no-fail-fast
  ```

- 결과: **8개 실행, 8개 통과, 0개 실패, 163개 skip, exit 0**. 실제 한국어·중국어
  HWP 모두 2쪽 oracle을 통과했고, 좌우 가로 조각을 같은 쪽에 유지하는 기존 계약과
  합성 세그먼트 반례도 함께 통과했다.
- `cargo fmt --all -- --check`와 `git diff --check`도 통과했다. nextest가 출력한
  `.config/nextest.toml`의 `profile.ci-duration-observation.junit.report-skipped`
  unknown-key warning은 기존 설정 경고이며 테스트 실패가 아니다.
- 전체 회귀, native/WASM/workspace lint, 최종 visual sweep은 이 단계에서 수행하지
  않았다. 이 단계의 결과는 저장 Square 그림 셀 분할 focused 계약의 통과이며, #6712
  전체 시각 동치 또는 PR 제출 준비 완료를 의미하지 않는다.
- 실행 로그와 임시 SVG/PNG/JSON은 커밋하지 않는다. 기준 PDF와 저장소에 필요한
  fixture·계약 테스트만 유지한다.

### 다음 단계 경계

- 이 문서와 현재 코드·계약 테스트를 일반 커밋으로 고정한다. 다음 단계에서 이 커밋을
  amend하지 않고 새 분석 문서를 만든다.
- 남은 검증은 전체 회귀, native/WASM/workspace lint, 최종 visual sweep이다. 실물 PDF의
  페이지 수가 focused oracle과 일치했지만, 전체 회귀 전에는 #6712를 `Closes` 대상으로
  삼지 않는다.
