# Stage 141: HWPX 부록 전환 page owner 분석 및 구현

## 목적

HWPX `2025 행정업무운영 편람(최종).hwpx`의 부록 전환에서 PDF와 다른 두
물리 페이지 owner를 분리해 보정한다.

- PDF p309는 부록 표지, p310은 blank, p311은 103×2 병렬 규정 표의 첫 fragment다.
- PDF p374는 `서식의 설계 기준` 표, p375는 다음 서식부터 시작한다.
- 전체 HWPX 쪽수는 PDF와 같이 383쪽을 유지한다.

이 Stage는 규정 표의 후반 행밀도 자체를 전역 reserve로 맞추지 않는다. 그 문제는
별도 Stage에서 PDF visual owner를 기준으로 다룬다.

## 분석

### 부록 표지 뒤 blank page

HWPX section 11의 raw 문단 순서는 다음과 같다.

1. p11.0: 부록 표지 `Group` shape와 section marker
2. p11.1: 빈 일반 문단
3. p11.2: `PageBreak`와 모든 `PageHide`가 있는 빈 문단
4. p11.3: `PageBreak`와 103×2, 206-cell `RowBreak` 규정 표

기존 `hwp5_origin_redundant_pagehide_break_marker`는 p11.2를 중복 marker로
간주해 생략했다. 그러나 바로 앞이 HWPX 부록 표지 `Group`, 바로 뒤가 103×2 규정
표인 이 topology에서 p11.2는 PDF p310 blank page의 owner다.

### p374 column break

section 12에는 다음 연속 구조가 있다.

- p12.16: 11×2, 22-cell, 29491×36625 `RowBreak` 표 (`서식의 설계 기준`)
- p12.17: 빈 `ColumnBreak`, `vertical_pos == 28030`
- p12.18: `PageBreak`와 6×3, 10-cell, 29254×35894 `RowBreak` 표

p12.17은 다음 p12.18의 `PageBreak`와 겹쳐 빈 물리 쪽을 만든다. 이 특정 stored
topology의 trailing column break만 억제해야 PDF p374 표와 p375 다음 서식의 owner를
동시에 보존한다.

## 구현

`src/renderer/typeset.rs`에서 다음 두 HWPX stored-layout 계약을 추가했다.

- `hwp5_origin_redundant_pagehide_break_marker`가 부록 표지 `Group` 뒤의 103×2
  규정 표 전환 marker를 생략하지 않는다.
- `hwpx_appendix_design_table_trailing_column_break`가 위 p12.16~p12.18의 빈
  column break만 suppress한다.

일반 PageHide, 일반 column break, HWP 입력, 다른 표 topology에는 적용하지 않는다.

## 결과

최종 HWPX render-tree 결과는 다음과 같다.

- 전체: 383쪽
- p309: 부록 표지
- p310: text node 0개인 blank page
- p311: 병렬 규정 표 첫 fragment
- p374: `서식의 설계 기준` 표
- p375: 다음 서식 시작

focused 회귀도 통과했다.

```bash
CARGO_TARGET_DIR=target/stage124-3820 \
  cargo test --profile release-test \
  --test issue_3930_hwpx_hwp_save_layout --quiet
# 3 passed; 0 failed
```

## 남은 범위

103×2 규정 표는 57 fragment와 383쪽 수를 유지하지만, PDF p364~p367과 비교하면
HWPX의 후반 행 owner 및 글자폭/행높이 누적이 아직 다르다. 전역 160px reserve는
조각 수만 맞추고 r5/r12를 조기 분할해 후반 본문을 밀 수 있으므로 이 Stage의
해법으로 확대하지 않는다. 다음 Stage에서 PDF visual owner와 row-level
measurement를 분리해 분석한다.

새 릴리스 준비 중이므로 이 Stage에서는 merge, push, PR 생성 또는 원격 변경을 하지
않는다.
