# Stage 2 보고서 — Task #1068: 근본 원인 확정 + 수정 설계

- 브랜치: `local/task1068` (진단 후 전량 revert, 소스 무변경)

## 근본 원인 — 확정 (HWPX TAC 표 attr 비트 미설정 → 오라우팅)

진단 (`DIAG_TAC`, para 567):
```
pi=567 ft.is_tac=false attr&1=0 treat_as_char=true rows=14 measured_h=843.1 base_avail=941.1
```

- 표 디스패치(`typeset.rs:2332`): `if ft.is_tac { typeset_tac_table } else { typeset_block_table }`.
- `format_table`(2091): `is_tac = table.attr & 0x01 != 0`.
- **HWPX 파서**(`section.rs:1104`)는 `common.treat_as_char` 만 설정, **`attr & 0x01`(HWP5식 TAC
  비트) 미설정** → HWPX TAC 표는 `is_tac=false` → **block 경로 오라우팅**.
- block 경로는 inline TAC 배치/페이지 fit 을 적용하지 않아 표(843px<941)가 페이지 하단에 얹혀
  본문 밖 839px 렌더 (used=1797px).

`attr & 0x01` 의존 사이트가 typeset 에 **9곳**(2091/2252/2429/2452/2486/2620/2624/2656/3760) →
format_table 한 곳만 고치면 나머지 미정합.

## 수정 설계 — 파서 정규화 (포괄적·단일점)

`parse_table`(`src/parser/hwpx/section.rs:1014`) 반환 직전:
```rust
if table.common.treat_as_char { table.attr |= 0x01; }
```
→ HWPX TAC 표의 `attr` 비트0 을 treat_as_char 와 일치(HWP5 정합)시켜, 모든 `attr & 0x01`
렌더 사이트가 일관 동작. (HWP5 파서는 이미 비트0 설정 — 본 수정은 HWPX 한정.)

대안(미채택): format_table is_tac 만 `|| treat_as_char` — 나머지 8개 사이트 미정합으로 불완전.

## 페이퍼 검증 (모순 점검)

| 케이스 | attr&1(전) | 수정 후 | 라우팅 |
|------|------|------|------|
| HWPX TAC 표(treat_as_char=true) | 0 | **1** | block→**tac** (정정) |
| HWPX 블록 표(treat_as_char=false) | 0 | 0 | block (불변) |
| HWP5 TAC 표 | 1 | 1 | tac (불변, 파서 다름) |
| HWP5 블록 표 | 0 | 0 | block (불변) |

→ treat_as_char=true 인 HWPX 표만 비트 추가. 다른 케이스 불변. 모순 없음.

- typeset_tac_table 경로: table_height=fmt.height_for_fit, fit 체크 후 `advance_column_or_new_page`
  → 843px 표가 페이지(941)에 fit → 본문 내 배치. (para 567: pi566 17px + 843 = 860 < 941)

## 리스크 / 검증 항목

- **직렬화 round-trip**: HWPX 직렬화는 treatAsChar 속성 사용(attr 비트 아님)이라 영향 없을 것으로
  보이나, HWPX→HWP 직렬화에서 attr 비트0 기록 확인(HWP5에선 TAC=비트0 이라 정합) — Stage 3 점검.
- **공개 테스트 픽스처**: 비공개 문서 대신, 트래킹된 HWPX 중 treat_as_char 표 보유 샘플
  (`표-텍스트.hwpx` 등) 로 동일 라우팅·overflow 재현 확인 → 테스트. 없으면 최소 HWPX 합성.
- 비회귀: tac-*/table-* 골든, 전 251 LAYOUT_OVERFLOW 합계.

## 다음 (Stage 3)
파서 정규화 1줄 + 공개 픽스처 확정 → 빌드·smoke(제안요청서 839px 해소) → 회귀 검증.
