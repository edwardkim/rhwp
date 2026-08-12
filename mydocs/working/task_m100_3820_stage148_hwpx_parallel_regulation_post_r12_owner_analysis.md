# Stage 148: HWPX 병렬 규정 표 r12 이후 owner 분석

## 목적

Stage 147에서 PDF p316의 제8조(r12) prefix owner를 복원한 뒤, p317 이후의 병렬
규정 표 owner가 PDF와 어떤 차이를 보이는지 다시 측정한다. 앞선 r12 전용 규칙의
범위를 넓히지 않고, PDF p317~p320과 HWPX의 첫 visible 조문·양쪽 셀 continuation을
비교해 다음 최초 발산 경계를 확정한다.

## 고정 계약

- HWPX와 한컴 PDF는 모두 383쪽이다.
- p310 blank, p314 제2장 시작, p316 제8조 prefix, p361 제61조, p362 제63조,
  p363 제63조의2·3, p364 제63조의3 tail, p365~p367 후반 조문·부칙 owner를
  유지한다.
- r12의 첫 fragment 외 continuation에는 전역 180px reserve를 유지한다. 모든 r12
  continuation reserve를 해제한 381쪽 후보는 재도입하지 않는다.

## 분석 절차

1. PDF/HWPX p317~p320의 왼쪽·오른쪽 첫 visible line과 r12 continuation 범위를
   나란히 대조한다.
2. 최초 owner 차이가 r12 continuation cut, 다음 r13~r15 complete-row 소비, 또는
   저장된 blank frame 중 어느 것인지 scanner와 fragment geometry로 분해한다.
3. 일반 103×2 표로 확장되지 않는 raw 구조·fragment 조건이 확인될 때만 구현한다.
4. 구현, 결과, source page-tree 회귀, focused integration test를 같은 커밋에 포함한다.

## 분석 결과

Stage 147 기준선은 383쪽과 p314, p316, p361~p367의 후반 owner를 보존했지만,
PDF p321~p323과 비교하면 r12의 오른쪽 셀이 두 쪽 더 오래 남았다.

- PDF p317~p320은 왼쪽 열이 비고, r12 오른쪽의 제3조~제6조가 이어진다. 따라서
  다음 왼쪽 행을 독립적으로 조기 소비하는 일반화는 원본과 다르다.
- 기준선의 r12 continuation은 `start_cut=[17, 18]` 뒤 오른쪽 cut이
  `32`, `46`, `60`, `73`, `87`로 14줄씩 증가했다. PDF p321은 제9조/제7조가
  시작하지만 기준선은 왼쪽이 비고 오른쪽 제5조 tail이 남아 있었다.
- r12 두 셀의 저장 lineSeg는 각각 17줄, 97줄이며 마지막 폭도 셀 폭을 넘지 않았다.
  stale line 재조판 후보는 실제 overflow가 없어 배제했다.

## 후보 측정

| 후보 | 전체 쪽수 | 판정 |
| --- | ---: | --- |
| r12 첫 fragment만 0px (Stage 147) | 383 | p321~p323이 두 쪽 늦어 유지 |
| r12 모든 continuation 0px | 381 | p321~p323은 맞지만 후반 owner가 앞섬 |
| r12 0px + r71 전체 180px | 383 | p361이 제58조로 과도하게 이월 |
| r12 0px + r71 continuation 180px | 382 | 한 쪽 부족 |
| r12 0px + r71 continuation 240px | 382 | 한 쪽 부족 |
| r12 0px + r71 continuation 360px | 385 | 세 쪽 과보정 |
| r12 0px + r71 continuation 300px | 383 | r17 제13조가 p322로 조기 소비되어 배제 |
| r12 0px + r17 owner break + r71 continuation 180px | 382 | 후반 owner가 한 쪽 앞서 배제 |
| r12 0px + r17 owner break + r71 continuation 300px | 383 | 채택 |

`r71`의 첫 fragment는 reserve를 바꿔도 쪽수를 만들지 않았고 continuation만
저장 frame 경계의 계단식 분할을 만들었다. r12 압축 뒤 r17 제13조는 PDF p323의
첫 owner여야 하므로 완전히 적합하더라도 p322 하단에는 넣지 않는다. 따라서 r71의
첫 fragment 0px 계약은 유지하고 continuation에만 300px reserve를 적용한다.

## 구현

- `src/renderer/typeset.rs`
  - 103x2 HWPX 병렬 규정 표의 r12는 첫 조각뿐 아니라 모든 continuation에 0px
    reserve를 적용한다.
  - r17 제13조가 새 fragment에서 시작하도록 owner break를 적용한다.
  - r71은 첫 조각 0px을 유지하고 continuation에만 300px reserve를 적용한다.
  - 두 규칙 모두 기존 `hwpx_parallel_regulation_table` fingerprint 안에서만
    작동하므로 일반 RowBreak 표에는 적용되지 않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`
  - PDF p321의 제9조, p322의 제11조, p323의 제13조 owner와 HWP 저장 왕복 tree를
    추가 고정한다.

## 결과

- `export-render-tree`로 HWPX 383쪽을 확인했다.
- p314 제2장 시작, p316 제8조 prefix, p321 제9조, p322 제11조, p323 제13조,
  p361 제61조, p362 제63조, p363 제63조의2·3, p364 제63조의3 tail,
  p365 제65조, p366 제69·70조 및 규칙 제47·48조, p367 부칙 owner를 확인했다.
- `visual_sweep.py --pages 321-324`에서 p322~p324는 flag 없이 통과했다. p321은
  footer frame의 16px tail과 bottom drift를 보수적으로 표시했지만 line-band 평균
  drift는 2.6px이고 조문 owner·line order 손실은 없었다. 이 footer-frame
  false-positive 가능성은 후속 시각 정밀화 대상으로 남긴다.
- 다음 focused integration test를 실행해 3개 테스트가 모두 통과했다.

  ```bash
  CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
    --test issue_3930_hwpx_hwp_save_layout
  ```

## 상태

구현·시각 sweep·focused integration test를 완료했다. 다음 Stage는 이 커밋 뒤에
시작한다. 새 릴리스 준비 중이므로 merge, push, PR 생성 또는 원격 변경은 금지한다.
