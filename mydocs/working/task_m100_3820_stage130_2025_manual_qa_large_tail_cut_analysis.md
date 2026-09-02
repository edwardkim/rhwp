# Stage 130 - 2025 행정업무운영 편람 Q&A large tail cut 분석

## 목표

Stage 129 커밋 `cd2c55a71` 뒤 native HWP 390쪽과 기준 Hancom PDF 383쪽 사이에 남은 `+7`쪽을 줄인다. 이번 Stage는 96px short-tail overflow를 넓히지 않고, native HWP Q&A `RowBreak` 표의 큰 마지막 응답 행을 historical renderer와 같은 fragment cut으로 배치하는 근거를 만든다.

## 기준선

- Hancom PDF: 383쪽
- Stage 129 native HWP: 390쪽
- Stage 129 HWPX: 386쪽
- Stage 129은 6x5 Q&A 표의 96px 이하 tail만 저장 frame owner에 남겨 391쪽에서 390쪽으로 줄였다.
- `tests/issue_3930_hwpx_hwp_save_layout.rs`는 native HWP 390쪽과 HWPX 386쪽을 각각 고정한다.

## 남은 핵심 후보

Stage 129의 `RHWP_DIAG_SCAN` 진단에서 Q&A 표의 마지막 실제 응답 행(`r=4`)은 다음 큰 초과를 보였다.

| 응답 행 높이 | 현재 잔여 | 초과 | 판단 |
| ---: | ---: | ---: | --- |
| 309.3px | 60.0px | 약 249px | 현재 page에 통째 배치 금지 |
| 478.4px | 364.7px | 약 114px | 현재 page에 통째 배치 금지 |

- 이 두 경우는 Stage 129의 96px guard에 포함하면 body/footer 경계를 침범한다.
- historical renderer는 전체 표를 통이동하지 않고 이전 page의 유효 row prefix와 다음 page의 suffix를 각각 소유했다.
- source page signal이 suffix 뒤 문단을 새 page로 고정하는지, 또는 `advance_row_cut`의 end-cut이 prefix를 너무 짧게 선택하는지를 분리해야 한다.

## 분석 결과

### pi=023: 별도 large-tail cut

- historical renderer는 physical p289에서 `PartialTable pi=023`, `end_cut=[1, 1, 11]`을 만들고 p290에서 suffix와 `pi=024`를 함께 배치했다.
- Stage 129은 p289에 `pi=021`, `pi=022`만 두고 `pi=023` 전체를 p290으로 이월했다.
- 이 표의 큰 행은 약 114px의 추가 높이가 필요하므로, Stage 129의 96px stored-frame overflow에는 포함하지 않는다.
- Stage 131에서 block/row cut gate와 p290의 suffix owner를 별도로 분석한다.

### pi=035: 0.5px minimum-prefix rounding

- historical renderer는 physical p293에서 `PartialTable pi=035`, `end_cut=[1, 1, 2]`를 만들고 p294의 suffix 뒤에 `pi=036`, `pi=037`을 함께 배치했다.
- Stage 129은 p295에 앞선 표만 두고 pi=035 전체를 p296으로 이월했다.
- initial scan의 row 4는 `budget=29.8px`, `consumed=24.5px`을 계산한다. 그러나 이 값은 pi=035가 아니라 먼저 scan된 pi=012의 값이었다.
- pi=035는 scan에 진입하기 전에 `DIAG_ADVC` pre-defer로 통째 이월된다. 따라서 0.5px minimum-prefix 보정은 이 표에 효과가 없으며 폐기한다.

### 공통 pre-defer 원인

- `pi=023`: `cur_h=352.2`, `declared=402.5`, saved bottom `719.1`에서 `bottom_fits=false`가 되어 `DIAG_ADVC`가 실행된다.
- `pi=035`: `cur_h=576.1`, `declared=191.2`, saved bottom `728.2`에서 같은 `DIAG_ADVC`가 실행된다.
- `pi=053`, `056`, `074`도 같은 declared/saved-frame pre-defer로 scanner 전 이월된다.
- 이 표들은 native HWP, 빈 host, non-TAC, `RowBreak`, 6행x5열, 15셀이라는 공통 저장 topology를 갖는다. historical renderer는 같은 표에 row prefix/suffix를 만들었으므로, early-defer가 아닌 뒤의 fragment scanner가 owner를 결정해야 한다.

## 분석 순서

1. 두 큰 후보를 paragraph index와 current/historical page item sequence에 정확히 연결한다.
2. 각 후보의 row 4 cell-unit, stored LINE_SEG, `advance_row_cut` end-cut과 historical fragment cut을 대조한다.
3. prefix가 현재 page에서 최소 내용량과 footer 안전선을 동시에 만족하는지 확인한다.
4. 구현은 fragment cut 선택 또는 suffix source owner로 한정하며, generic overflow allowance를 늘리지 않는다.

## 보존 계약

- PDF physical 278의 PageHide blank page를 삭제하거나 title host와 병합하지 않는다.
- Stage 128의 1x1 목차 표 saved-frame tail과 Stage 129의 96px short-tail 보정은 유지한다.
- HWPX, TAC, 글자처럼 취급되는 표, 일반 `RowBreak` 표에는 Stage 130 분기를 적용하지 않는다.
- fixture 이름, physical page 번호, paragraph index를 조건으로 코드 분기하지 않는다.

## 수용 기준

1. 큰 tail 두 건의 paragraph/table identity와 historical/current fragment owner를 문서에 남긴다.
2. source storage와 현재 page boundary가 뒷받침하는 row prefix/suffix cut만 구현한다.
3. native HWP page count와 HWPX 386쪽 회귀 결과를 코드 변경 뒤 기록한다.

## 구현

- native HWP, non-TAC, `RowBreak`, 6행x5열, 15셀 Q&A 표의 마지막 실제 응답 행만 식별한다.
- declared/saved-frame pre-defer에서 위 Q&A topology만 제외해 기존 fragment scanner로 넘긴다.
- 표 전체 높이 또는 row budget을 늘리지 않고, scanner가 계산한 row prefix/suffix만 사용한다.
- 실패한 0.5px minimum-prefix 후보와 temporary diagnostic은 최종 코드에 남기지 않는다.

## 결과

- native HWP page 수는 `390 → 386`으로 4쪽 감소했다. HWPX는 기존 386쪽을 유지한다.
- `pi=023`, `035`, `053`, `056`, `074` 등 declared/saved-frame early-defer 대상이 기존 row-cut scanner에서 prefix/suffix를 만들게 되면서 section 10 이후의 누적 offset은 `+7`에서 `+3`으로 줄었다.
- historical renderer 대비 final `+3`은 `pi=056` 뒤 `pi=057`에서 `+2`가 되고, `pi=074` 뒤 `pi=075`에서 `+3`이 된 뒤 문서 끝까지 유지된다. `pi=085`의 일시 `+4`는 `pi=098`의 owner 합류로 다시 `+3`이 된다.
- PDF 383쪽에는 아직 도달하지 않았다. 다음 Stage는 `pi=056`과 `pi=074`의 terminal spacer/source owner를 분석하며, 이번 Stage의 pre-defer bypass를 넓히지 않는다.
- focused regression은 `tests/issue_3930_hwpx_hwp_save_layout.rs`에서 native HWP 386쪽, HWPX 386쪽으로 고정한다.
- 실행: `CARGO_TARGET_DIR=target/stage124-3820 cargo test --profile release-test --test issue_3930_hwpx_hwp_save_layout --quiet`
- 결과: 3 passed, 0 failed (0.84초).
