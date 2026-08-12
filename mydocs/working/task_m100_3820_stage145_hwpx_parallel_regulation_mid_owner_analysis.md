# Stage 145: HWPX 병렬 규정 표 중간 owner 분석

## 목적

2025 행정업무운영 편람 HWPX 103×2 병렬 규정 표에서 PDF p362~p364와 다른
제61조~제63조의5 owner를 분석한다. Stage 144가 고정한 p365~p367 owner와 383쪽
계약을 유지하면서, r84~r92의 complete-row·partial-row 경계를 PDF visible owner에
맞춘다.

## 고정 계약

- 한컴 PDF와 HWPX render tree는 383쪽이다.
- p310 blank, p314 제2장 시작, p365 제65~68조, p366 제69·70조와 시행규칙
  제47·48조, p367 양쪽 부칙 owner를 유지한다.
- HWPX와 native HWP의 동일 셀 line 수·height는 일치한다. 차이는 Chrome 대체 글꼴
  metric이 아니라 RowBreak owner 판정으로 취급한다.

## 분석 절차

1. PDF/HWPX p361~p365의 visible 조문 시작과 row unit을 나란히 추출한다.
2. r84~r92에서 complete-row fit으로 조기 소비되는 행과 partial-row로 cut되는 행을
   구분한다.
3. PDF p362의 제62조 tail·제63조, p363의 제63조의2 tail·제63조의3, p364의
   제63조의3 tail·제63조의4~제65조를 각각 독립 owner로 고정한다.
4. `r84`는 남은 높이 0px에서 시작해 로컬 cut reserve가 owner를 바꾸지 못함을
   확인한다. 따라서 p364의 조기 heading은 표 전체의 보수적 cut 누적에서 발생한 것으로
   한정한다.

## 구현

- `HWPX_PARALLEL_REGULATION_CUT_RESERVE_PX`를 200px에서 180px로 낮췄다.
- r79 전용 reserve 0px 후보도 비교했지만 p360~p363 owner를 이동시키지 못해 채택하지
  않았다. row-local 상수로 원인을 가리는 대신, 전체 표에 적용되는 보수량만 20px
  줄이는 최소 변경을 유지한다.
- Stage 144의 r5, r71, r99 reserve와 r97 complete-row owner break는 변경하지 않았다.

## 결과

- HWPX render tree는 한컴 PDF와 같이 383쪽이다.
- PDF p364는 제63조의3 heading이 아니라 `관리하여야 한다`로 시작하는 본문 tail을
  소유한다. 이전 200px reserve에서는 p364가 제63조의3 heading부터 시작했다.
- p365는 제65~68조, p366은 제69·70조와 시행규칙 제47·48조, p367은 부칙 owner를
  계속 유지한다.
- p310 blank와 p314 제2장 시작도 유지했다.

## 회귀

- `issue_3930_preserves_page_count_and_inherited_even_master_page`에 p364의 본문 tail
  owner와 제63조의3 heading 비소유를 추가했다.
- 아래 focused integration test로 HWPX 원본, HWP 저장·재로드 및 기존 page-owner
  계약을 함께 확인한다.

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout
```

## 상태

focused integration test는 3 passed, 0 failed로 통과했다. 이 문서는 코드·테스트
변경과 같은 커밋에 포함한다. 새 릴리스 준비 중이므로 merge, push, PR 생성 또는
원격 변경은 금지한다.
