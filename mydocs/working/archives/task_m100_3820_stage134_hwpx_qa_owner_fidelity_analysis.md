# Stage 134 - 2025 행정업무운영 편람 HWPX Q&A page owner fidelity 분석

## 목표

Stage 133에서 HWPX source 및 저장-재로드 쪽수를 Hancom PDF와 같은 383쪽으로 고정했다. 이번 Stage는 총쪽수 일치만으로 종료하지 않고, HWPX Q&A 표의 physical page owner가 PDF 및 native HWP와 같은지 분석한다.

## 기준선

- 기준 PDF: `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf` (383쪽)
- native HWP: `samples/2025 행정업무운영 편람(최종).hwp` (Stage 133 뒤 383쪽)
- HWPX: `samples/2025 행정업무운영 편람(최종).hwpx` (Stage 133 뒤 source/저장-재로드 모두 383쪽)
- 선행 커밋: `c42c838c6`

## 현재 관찰

- PDF 및 native HWP에서는 Q7의 첫 response 3줄이 physical p284에 있고, 남은 tail과 Q8 표제가 p285에 있다.
- Stage 133 덤프의 HWPX는 같은 Q7 구조에서 p284의 `end_cut=[1, 1, 2]`를 선택해 p285가 tail-only가 되고 Q8은 p286에서 시작했다.
- HWPX는 목차 tail 보정으로 총 383쪽이 됐지만, 이 Q7 owner 불일치는 이후 content anchor가 상쇄된 결과일 수 있다.

## 분석 범위

1. HWPX Q7 표와 native HWP Q7 표의 raw storage signal, row 4 cell 문단 수, declared height, margin, stored line segment를 비교한다.
2. HWPX가 `hwp5_origin_qa_first_response_tail` 조건 또는 후속 physical-fit guard에서 native와 다르게 동작하는 지점을 trace로 고정한다.
3. PDF physical p284~p286, native HWP, HWPX의 Q7/Q8 page item 및 SVG paint를 같은 해상도로 대조한다.
4. HWPX owner를 맞춰도 source/round-trip 383쪽이 유지되는지와 이후 first divergence를 확인한다.

## 보존 계약

- Stage 133의 HWPX 383쪽 source 및 저장-재로드 회귀를 후퇴시키지 않는다.
- native HWP의 Q7 p284/p285 owner와 383쪽 회귀를 변경하지 않는다.
- PDF oracle을 재생성하거나 대체하지 않는다.
- fixture 경로, physical page 번호, paragraph index로 구현 분기하지 않는다.

## 수용 기준

1. HWPX Q7의 2줄/3줄 cut 차이를 raw storage 또는 layout metric으로 재현 가능하게 설명한다.
2. 구현이 필요하면 HWPX의 해당 source topology만 다루고 generic RowBreak 허용치를 넓히지 않는다.
3. native HWP와 HWPX가 모두 383쪽이며 Q7/Q8 physical owner 근거를 결과 문서에 남긴다.

## 다음 분석 명령

```bash
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
```

필요한 page/item 증적은 `dump-pages`, SVG export, 기존 Hancom PDF에서만 얻는다.

## 분석 결과

### HWPX Q7 raw storage 계약

`Contents/section10.xml`의 Q7 표는 다음 구조를 가진다.

- non-TAC `hp:tbl`, `pageBreak="CELL"`, 6행x5열, 높이 13042 HU, outer bottom margin 0
- 마지막 실제 응답 행은 row 4이며, label cell 두 개와 response cell 한 개로 구성된다.
- response cell(`colAddr=2`, `rowAddr=4`)은 3개의 `hp:p`를 저장한다.
- 세 문단의 `hp:lineseg` 수는 순서대로 3, 5, 3개다.
- response cell 선언 높이는 2949 HU이고, 마지막 row 5는 빈 spacer다.

native HWP의 동등 표는 parser 단계에서 response cell을 5문단으로 보존한다. Stage 133의 `hwp5_origin_qa_first_response_tail` predicate는 `paragraphs.len() == 5`만 허용했으므로 HWPX의 3문단 구조에서는 false가 됐다. 따라서 HWPX는 Q7 first-response tail allowance와 physical-fit allowance를 전혀 받지 않아 `end_cut=[1, 1, 2]`를 유지했다.

### 구현 가설

HWPX profile에서만 response cell의 `3 hp:p`와 총 11개 stored line segment를 동등 계약으로 수용한다. 표 height, row/column/cell topology, row 4 위치, zero bottom margin을 동시에 요구하므로 다른 HWPX Q&A 표 및 generic RowBreak에는 적용되지 않는다.

### parser 정규화와 cut budget 정정

원본 XML에는 response `hp:p`가 3개로 보이지만, 실제 HWPX parser는 Q7 response를 5문단과 15개 line segment(`3, 2, 3, 3, 4`)로 정규화한다. 진단에서 기존 predicate는 이미 `eligible=true`였다.

직접 원인은 predicate가 아니라 예산 부족이다. HWPX Q7 row 4의 raw budget은 18.3px이고, Stage 133의 64px allowance를 더한 `cut_budget=82.3px`에서도 `advance_row_cut`는 `end_cut=[1, 1, 2]`만 선택한다. native HWP와 달리 HWPX는 같은 저장 response prefix를 선택하려면 더 큰 allowance가 필요하다. 따라서 3문단 XML 조건은 구현에서 제거하고, HWPX Q7의 저장 response tail에만 96px allowance를 cut과 physical-fit 양쪽에 일관되게 적용한다.

### Q7 owner 보정 뒤의 한 쪽 회복 경로

96px HWPX allowance를 적용한 현재 덤프에서 Q7은 PDF와 같이 p284 `end_cut=[1,1,3]`, Q8은 p285에 놓인다. 다만 section 11 시작은 p316, section 12 시작은 p366, final은 p382가 되어 한 쪽 부족하다.

Q&A 이전의 목차나 PageHide를 되돌리면 Q7/Q8 자체가 다시 한 쪽 늦어지므로 보상 위치가 될 수 없다. Q&A 이후 section 11의 non-TAC `RowBreak` 103x2/206-cell 병렬 규정 표는 native HWP에서 별도 saved-tail reserve가 필요한 것으로 이미 확인됐다. HWPX에는 이 reserve가 없으므로, 같은 table topology에 HWPX profile만의 16px reserve를 적용해 이후 fragment 하나를 새 page에 소유시킨다. 이 보정은 Q7 이전 page owner에 영향을 주지 않는다.

### 한 쪽만 추가하는 source row 후보

8px 공통 reserve의 diff는 row 22 부근과 row 71 부근에서 각각 한 page를 추가했다. section 11 원본 XML에서 row 22는 다음 raw topology를 가진다.

- 103x2 병렬 규정 표의 1×1 cell pair
- 두 cell의 선언 높이가 모두 68028 HU
- 각 cell의 `rowSpan=1`, 공통 cell margin은 좌우 566 HU, top 566 HU, bottom 1133 HU

row 71의 cell pair는 102320 HU이며 별도 분기다. HWPX의 목표는 382쪽에서 한 쪽만 회복하는 것이므로, common 8px reserve는 폐기하고 68028 HU pair에만 적용한다. 이 identity는 section/page/paragraph/row index가 아니라 cell의 raw HWPX geometry로 식별한다.

### raw pair 정정

parser 직전 XML의 cell address/size 순서를 다시 대조한 결과 row 22의 정확한 height pair는 `8213 HU`와 `68028 HU`다. 앞선 “두 cell 모두 68028 HU” 기록은 잘못된 중간 추출에 근거했으며 구현에는 발동하지 않았다. row 71의 pair는 `7315 HU`와 `102320 HU`이므로 계속 제외한다.

## 최종 구현 및 결과

### 확정한 HWPX physical-owner 계약

1. Q7의 6×5 RowBreak 응답 tail은 HWPX 저장 layout에서 `96px`까지 허용한다. 기존 `64px`에서는 p284의 `end_cut=[1, 1, 2]`가 선택되어 p285가 tail-only였지만, 96px에서는 PDF/native HWP와 같이 세 번째 response 문단까지 p284에 남고 Q8 표제가 p285에서 시작한다.
2. 이 owner 보정은 전체 흐름을 한 쪽 압축해 HWPX source를 382쪽으로 만든다. 보상은 Q7 이전의 목차나 PageHide를 되돌리는 방식이 아니라, 이후 HWPX non-TAC `RowBreak` 103행×2열/206-cell 병렬 규정 표의 row-cut 예산에서만 `4px`를 보존하는 방식으로 한정했다.
3. `4px`는 382쪽을 정확히 383쪽으로 복원한다. 같은 표 전체에 `8px` 또는 `16px`를 적용한 중간 실험은 row 22 부근과 row 71 부근에서 각각 한 쪽을 추가해 384쪽이 됐으므로 폐기했다.
4. `table_layout.rs`의 generic HWPX mid-page reset absorb를 막는 실험은 이 fixture의 cut sequence를 바꾸지 못해 382쪽을 유지했다. 전역 HWPX 규칙을 넓히지 않도록 해당 변경은 제거했다.

### 회귀 계약

`tests/issue_3930_hwpx_hwp_save_layout.rs`는 다음을 함께 고정한다.

- HWPX source page count는 Hancom PDF와 같은 383쪽이다.
- HWPX source p285에는 `홈페이지상의 질의에 대하여` Q8 표제가 있다.
- HWPX를 HWP로 저장한 뒤 재로드한 p285 render tree는 source p285 tree와 같다.
- 기존 p30/p144/p145 바탕쪽·표 owner 및 native HWP의 p285/Q&A 383쪽 계약은 그대로다.

### 검증 결과

```text
CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet

running 3 tests
...
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

PDF oracle은 기존 `pdf/2025 행정업무운영 편람(최종)-hwp-2020.pdf`(383쪽)를 계속 사용했으며 재생성하지 않았다.
