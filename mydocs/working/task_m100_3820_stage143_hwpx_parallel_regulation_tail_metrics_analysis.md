# Stage 143: HWPX 병렬 규정 표 fragment metrics 및 제3조 tail 보정

## 목적

HWPX 103×2 병렬 규정 표의 p314 및 p364~p367을 PDF와 대조해, 남아 있는 행 owner
차이가 Chrome 대체 글꼴의 glyph width/line height 차이인지, stored RowBreak fragment
owner 차이인지를 분리하고 첫 불일치를 보정한다.

## 현재 계약

- PDF와 HWPX는 383쪽이다.
- Stage 142의 r5 고정 여유값 `56px`은 p314의 제3조 tail을 완전히 제거하지 못했다.
- PDF p364~p367은 각각 제63조의4~제65조 시작, 제65조 tail~제68조, 제69조~제70조와
  시행규칙 제47·48조 시작, 양쪽 부칙을 소유한다.
- 현재 HWPX p364~p367은 r87~r102를 더 이른 행 단위로 소유해 PDF와 다르다.
- p310 blank 및 p374→p375 전환과 전체 383쪽을 후퇴시키지 않는다.

## 분석 절차

1. PDF와 HWPX SVG p364~p367, render tree의 r87~r102 셀 line 수와 높이를 추출했다.
2. native HWP의 같은 행 line 수와 셀 높이가 HWPX와 일치함을 확인했다. 따라서 이
   불일치는 Chrome 대체 글꼴의 폭이나 line height를 보정해서 해결할 문제가 아니다.
3. 첫 owner 불일치인 r5를 별도로 계측했다. 전역 `56px`은 `7/20/20/9`, 전역
   `-160px`은 `2/30/24/0` fragment 분포를 만들었다.
4. r5 세 번째 continuation(`row_start_cut[0] >= 27`)에만 `-160px`을 적용하면
   PDF에 가까운 `7/20/29/0`이 된다. 첫 두 fragment의 제3조 초반 owner를 유지하면서
   p314의 마지막 9줄을 p313에 회수한다.

## 구현

- `src/renderer/typeset.rs`는 103×2 HWPX 병렬 규정 표의 r5에서 첫 cell cut이 27 이상인
  세 번째 continuation만 별도 reserve를 사용한다.
- 일반 HWPX 표, HWP5-origin 표, r5의 첫 두 fragment 및 r71의 기존 보정에는 영향을 주지
  않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`는 PDF p314가 `공문서 관리`를 소유하고
  `정책실명제` tail을 소유하지 않음을 HWPX 원본과 HWP 저장 roundtrip에 함께 고정한다.

## 결과

- 최종 HWPX render tree는 383쪽이다.
- p314는 `제2장 공문서 관리 등 행정업무의 처리` 및 제4조로 시작하며, r5의
  `정책실명제` tail은 포함하지 않는다.
- p310은 비어 있고 p373·p374에는 모두 내용이 있어, 기존 appendix blank 및 p374→p375
  전환 계약을 유지한다.
- 다음 focused 회귀를 실행했다.

  ```bash
  CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
    --test issue_3930_hwpx_hwp_save_layout
  ```

  결과: `3 passed; 0 failed`.

## 후속 범위

- p364~p367의 r87~r102 후반 owner 차이는 font metric 문제가 아니라 별도의 stored
  RowBreak fragment 계약이다. 다음 Stage에서 r71 이후의 누적 fragment 기준으로
  독립 분석한다.

## 상태

분석, 구현, focused 회귀 및 결과 기록을 완료했다. 이 Stage는 코드와 회귀 테스트를
함께 커밋한다. 새 릴리스 준비 중이므로 merge, push, PR 생성 또는 원격 변경은 금지한다.
