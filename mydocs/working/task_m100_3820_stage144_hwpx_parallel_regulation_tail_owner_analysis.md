# Stage 144: HWPX 병렬 규정 표 후반 owner 누적 분석

## 목적

2025 행정업무운영 편람 HWPX의 103×2 병렬 규정 표에서 PDF p364~p367과 다른 후반
fragment owner를 분석하고, p365~p367의 조문·부칙 owner를 보정한다. Stage 143에서
확정한 p314 제3조 tail 보정과 383쪽 계약을 유지한다.

## 고정 계약

- 한컴 PDF와 HWPX 렌더는 383쪽이다.
- p310 blank, p314의 제2장 시작, p374→p375 전환은 유지한다.
- HWPX와 native HWP의 같은 셀 line 수·height는 일치한다. 후반 차이는 Chrome 대체
  글꼴 metric 문제가 아니라 stored RowBreak fragment owner 문제로 취급한다.
- Stage 143의 r5 보정은 이 Stage의 기준이며 되돌리지 않는다.

## 분석 절차

1. PDF와 현재 HWPX render tree p353~p367의 visible 조문 owner를 나란히 추출했다.
2. native HWP와 HWPX의 같은 셀 line 수·height는 일치했다. 차이는 Chrome 대체 글꼴
   metric이 아니라 stored RowBreak fragment 예산과 complete-row owner 판정이다.
3. r71의 고정 `200px` reserve는 p355의 제51조 시작을 늦추며 후반 누적을 만들었다.
   r71은 저장 frame tail 뒤 다음 행을 시작시켜야 한다.
4. p365에서 r97(제69조)은 complete-row fit 경로여서 cut reserve가 적용되지 않았다.
   r93~r96이 현재 fragment를 소유한 경우 r97을 다음 fragment로 넘기는 owner break가
   필요했다.
5. r99는 p366의 212.6px 잔여에서 일반 reserve를 적용하면 첫 줄만 선택되어
   `MIN_TOP_KEEP_PX`에 못 미쳤다. reserve 없이 제47·48조 10 line을 소비하면,
   `vpos=0` reset 뒤 시행규칙 부칙은 p367에 남는다.

## 구현

- 병렬 규정 표의 일반 cut reserve를 `200px`로 두고, r71과 r99는 stored frame의
  실제 owner 단위를 위해 `0px` reserve를 사용한다.
- r97이 다른 행 뒤에서 완전 적합으로 판정되는 경우에만 page owner break를 적용한다.
  새 fragment의 첫 행이면 조건이 거짓이므로 반복 page break는 생기지 않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`에 PDF p365~p367의 제65~70조,
  시행규칙 제47·48조, 부칙 owner를 HWPX source 회귀로 고정했다.

## 결과

- 최종 HWPX render tree는 383쪽이다.
- p365는 제65~68조만, p366은 제69·70조 및 시행규칙 제47·48조, p367은 양쪽 부칙을
  소유한다.
- p310 blank, p314 제2장 시작 및 p373→p374 전환은 유지했다.
- focused 회귀는 아래 명령으로 실행한다.

  ```bash
  CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
    --test issue_3930_hwpx_hwp_save_layout
  ```

  결과: `3 passed; 0 failed`.

## 잔여 범위

- p362~p364의 제61조~제63조의5 중간 owner는 PDF와 아직 완전히 일치하지 않는다.
  특히 p364는 PDF처럼 제63조의3 tail에서 시작하는 대신 아직 조문 표제부터 보인다.
  다음 Stage에서 r84~r92의 complete-row와 partial-row 경계를 별도로 분석한다.

## 상태

분석과 구현을 완료했고 focused 회귀 결과를 반영한 뒤 코드·테스트·문서를 함께
커밋한다. 새 릴리스 준비 중이므로 merge, push, PR 생성 또는 원격 변경은 금지한다.
