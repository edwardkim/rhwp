# Stage 3 보고서 — Task #1068: 파서 정규화 시도 → 반려 + 재설계

- 브랜치: `local/task1068` (시도 후 revert, 소스 클린)

## 시도 — 파서 정규화 (Stage 2 설계)

`parse_table` 반환 직전 `if treat_as_char { attr |= 0x01 }` → HWPX TAC 표를 TAC 경로로 라우팅.

## 결과 — 타깃 해소 but 광범위 회귀 (반려)

- ✅ 제안요청서.hwpx: para 567 **839px → 해소**(잔여 ≤27px), cargo test 1324 + 골든 전부 통과.
- ❌ 251 샘플 LAYOUT_OVERFLOW **1626 → 1670 (+44)**, HWPX **6파일 악화**:
  - hwp3-sample11-hwpx **0 → 167px**(작은 표인데 신규 대형 overflow), tac-img-02 6→21(max34),
    hwp3-sample16-hwp5 15→27, aift 4→7, mel-001 3→10, 3-09'22 154→155.
  - (개선 3: 해외직접투자/hwpx-h-02 2→1.)

## 분석 — 두 경로 모두 HWPX TAC 표 갭

- 악화 양상 비일관: hwp3-sample11-hwpx 는 **작은 표**(≤49mm)인데 TAC 경로에서 167px overflow,
  tac-img-02 는 page-larger(254mm>본문252mm). 즉 "모든 HWPX TAC 표 → TAC 경로"는 **block 경로가
  잘 처리하던 파일들을 broadly 악화**.
- 제안요청서만 이득: block 경로가 **near-full-page 단일 TAC 표(843px)** 를 페이지에 cram(used
  1797px) → overflow. 반면 다른 HWPX TAC 표는 block 경로에서 정상.
- → 결함은 "라우팅" 이 아니라 **block 경로의 near-full-page TAC 표 cram**(이월 미수행). TAC 경로도
  소형 HWPX TAC 표(hwp3-sample11)에서 별도 갭 보유.

## 재설계 방향 (재승인 필요)

라우팅 전면 변경(반려) 대신 **block 경로에서 TAC 표 이월(carry) 조건 추가**:
- typeset_block_table 가 treat_as_char 표를 받고, **현재 페이지엔 안 들어가나 새 페이지엔 들어감**
  (measured_h ≤ base_available) 이면 cram 대신 `advance_column_or_new_page` 후 배치.
- page-larger(measured_h > base_available)면 현행 split 유지(tac-img-02 등 불변).
- 소형 TAC 표(현 페이지에 들어감)도 불변(hwp3-sample11 등 불변).

→ 제안요청서(843<941, 현 페이지 잔여 부족 시 이월)만 정정, 타 파일 불변 기대. 라우팅·TAC 경로
미변경이라 회귀면 최소.

## 다음
재설계(block 경로 TAC carry-if-fits) 승인 후 Stage 3 재구현 → smoke(제안요청서 + 6파일 무회귀)
→ 회귀 검증. (Stage 2 설계는 폐기.)
