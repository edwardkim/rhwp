# Stage 129 - 2025 행정업무운영 편람 잔여 page offset 보정

## 목표

Stage 128 커밋 `c834ef4ff` 뒤 native HWP 391쪽과 기준 Hancom PDF 383쪽 사이에 남은 `+8`쪽의 발생 지점을 분해하고, source storage가 보장하는 Q&A 표 tail만 보정한다.

## 기준선

- Hancom PDF: 383쪽
- Stage 128 native HWP: 391쪽
- Stage 128 HWPX: 386쪽
- section 10 p4는 native HWP physical page 280-282, PDF physical page 279-281에 있다.
- PDF physical page 278은 PageHide가 적용된 실제 blank page다. 따라서 이 blank page를 삭제하거나 content host와 병합하는 방식은 금지한다.

## 분석 범위

1. PDF와 native HWP의 section 시작·종료 page를 표로 대조해 처음 누적되는 offset을 찾는다.
2. section 10 직전의 PageHide blank page와 title host가 각각 어떤 physical page를 소유하는지 확인한다.
3. section 10 내부에서 Stage 128 이후에도 남은 추가 page와 section 11 이후의 누적 차이를 분리한다.

## 보존 계약

- Stage 128의 p4 first stored-frame tail allowance와 3 fragment cut을 보존한다.
- PageHide blank page를 삭제·병합하지 않는다.
- fixture 식별자나 page index를 조건으로 코드 분기하지 않는다.

## 수용 기준

1. 남은 `+8`쪽을 section boundary 단위의 재현 가능한 원인 목록으로 기록한다.
2. 다음 구현 후보마다 PDF page, native HWP page, source storage signal을 함께 제시한다.
3. footer 침범 없이 검증 가능한 short tail만 수용하고, 남은 차이는 다음 Stage의 fragment-cut 과제로 분리한다.

## 분석 결과

### section 경계

| 구간 | historical renderer | Stage 128 native HWP | 차이 |
| --- | ---: | ---: | ---: |
| section 10 시작 | physical 278 | physical 278 | 0 |
| section 11 시작 | physical 312 | physical 320 | +8 |

- Stage 128의 p4 목차 표는 historical/current 모두 native HWP physical 280-282의 세 fragment다.
- PDF physical 278의 PageHide blank page는 실제 공백 페이지이므로 삭제 또는 title host 병합은 후보에서 제외했다.
- 따라서 남은 `+8`은 p4나 section 10 앞의 blank page가 아니라 section 10 Q&A 표 흐름에서 발생한다.

### 지속 page 증가 지점

historical/current `dump-pages`의 첫 page owner를 paragraph index로 대조했다. 일시적으로 분기했다가 다시 합류하는 `pi=014`는 제외하고, 이후까지 누적되는 증가 지점은 다음 여덟 개다.

| paragraph index | current tail 또는 통이동 | 누적 차이 |
| ---: | --- | ---: |
| 023 | Q&A 표가 잔여 page를 채우지 못하고 다음 page 시작 | +1 |
| 030 | 82.9px continuation 전용 page | +2 |
| 035 | 6x5 표가 통째로 다음 page 시작 | +3 |
| 053 | 258.1px 표 전용 page | +4 |
| 056 | 203.0px 표 전용 page | +5 |
| 068 | 78.9px continuation 전용 page | +6 |
| 074 | 319.8px 표 전용 page | +7 |
| 085 | 90.1px continuation 전용 page | +8 |

공통 구조는 native HWP, non-TAC, `RowBreak`, 6행x5열, 15셀의 Q&A 표다. 응답 본문이 row 4에 있고 row 5는 빈 spacer다. `advance_row_cut`은 row 4의 짧은 tail을 만들지만, 다음 source paragraph가 새 physical page를 가리켜 tail만 남은 page를 독립 소유하게 한다.

### PDF 대조의 해석

- PDF physical 293은 Q22-Q23, current native HWP physical 293은 Q15-Q16을 보였다. 입력 HWP의 Q&A 목차/저장 페이지 신호와 PDF의 물리 content index가 완전히 같은 좌표계가 아니므로, 단일 page image의 content 번호를 직접 1:1 oracle로 삼지 않는다.
- 다만 native HWP source의 Q&A 표는 stored declared frame과 마지막 blank spacer 행을 갖고, historical renderer는 이 tail 전용 page를 만들지 않아 전체 383쪽이었다. 따라서 이번 보정의 oracle은 PDF 최종 383쪽과 해당 HWP storage topology의 교집합이다.

## 구현

- `src/renderer/typeset.rs`에서 native HWP의 6행x5열 Q&A `RowBreak` 표만 식별한다.
- 마지막 실제 응답 행(`r + 2 == row_count`)이 현재 fragment에 이미 앞선 행을 가진 상태이고 96px 이내로만 넘치면 stored frame owner를 유지한다.
- HWPX, TAC, 글자처럼 취급되는 표, rowspan 행, 일반 표에는 적용하지 않는다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`에 native HWP 383쪽 회귀 기준을 추가한다.

## 결과

- native HWP page 수는 `391 → 390`으로 한 쪽 감소했다. HWPX fixture의 기존 386쪽 계약은 이 native-HWP 분기의 대상이 아니다.
- 96px 이하의 short tail 후보 중 `pi=068`의 78.9px continuation 전용 page가 제거되어 section 11 시작은 physical 320에서 319로 이동했다.
- 같은 조건을 통과한 `pi=035`, `053`, `056`은 fragment owner를 현재 page에 유지했지만, 뒤따르는 source page signal이 별도 page를 필요로 해 최종 page 수는 추가로 줄지 않았다.
- 진단에서 남은 큰 후보는 `row=478.4px, rest=364.7px`(약 114px 초과)와 `row=309.3px, rest=60.0px`(약 249px 초과)였다. 이를 96px guard에 포함하면 저장 frame이 아니라 footer 영역을 침범할 수 있으므로 수용하지 않았다.
- 다음 Stage는 남은 `+7`을 `pi=023, 030, 035, 053, 056, 074, 085`의 historical fragment cut과 current source-owner signal로 나누어, 큰 tail을 overflow시키지 않는 cut 재현을 다룬다.
- focused regression은 `tests/issue_3930_hwpx_hwp_save_layout.rs`의 native HWP 390쪽 assertion과 기존 HWPX 386쪽 assertion으로 고정한다.
- 실행: `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout --quiet`
- 결과: 3 passed, 0 failed (1.03초).
