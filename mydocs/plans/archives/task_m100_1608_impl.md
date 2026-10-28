# 구현계획서 — Task #1608

**제목**: `is_hwp3_origin` 오탐지 제거 — 네이티브 HWPX 부당 tolerance 차단
**마일스톤**: M100 · **이슈**: edwardkim/rhwp#1608 · **브랜치**: `local/task1608`
**확정 방향**: 수행계획서 §5 **방향 3**(tolerance 제거 + HWP3 변환본 시각 회귀 점검) — 승인됨.

## 수정 대상 (단일 지점)

`src/parser/hwpx/mod.rs:177-181, 264-268`
```rust
let hwpml_version = header::parse_hwpx_hwpml_version(&header_xml);
let is_hwp3_origin = hwpml_version.as_deref() == Some("1.4");   // ← 오탐지 근원
doc_info.hwpml_version = hwpml_version.clone();                  // (무손실 보존 — 유지)
...
if is_hwp3_origin {                                              // ← 부당 tolerance
    for section in sections.iter_mut() {
        section.section_def.page_def.pagination_bottom_tolerance =
            section.section_def.page_def.margin_bottom.min(1600);
    }
}
```
- `hwpml_version` 보존(doc_info.hwpml_version) 은 직렬화 무손실에 필요 → **유지**.
- 제거 대상은 `is_hwp3_origin` 판정 + tolerance 부여 블록.

## 단계 (3단계)

### Stage 1 — RED + HWP3 변환본 영향 측정

1. **RED 단위테스트** (`src/parser/hwpx/mod.rs` `#[cfg(test)]` 또는
   `tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs`):
   - 네이티브 HWPX(`samples/hwpx/` 내 head version 1.4 파일)를 `parse_hwpx` →
     모든 `section.section_def.page_def.pagination_bottom_tolerance == 0` 단언.
   - 현재 동작(비-0 부여) → **RED**.
2. **HWP3 변환본 영향 측정**(빌드 후, 코드 미수정 상태에서 baseline 먼저):
   - `rhwp info samples/hwp3-sample-hwpx.hwpx` = 현재 16쪽(측정 완료).
   - tolerance 제거 시(Stage 2 적용 후) 페이지수·SVG 변동 측정 → **완전 제거 vs 보수 가드** 결정.
   - 판정 기준: HWP3 변환본이 권위자료(`pdf/`·`pdf-2020/`) 대비 시각·페이지 회귀가 없으면
     **완전 제거 확정**. 회귀 발생 시 Stage 2에서 보수 가드(콘텐츠 휴리스틱 재사용) 추가.
3. Stage1 완료보고서(`_stage1.md`) — 측정 수치 + 완전제거/가드 결정 → 승인 요청.

### Stage 2 — 수정 (GREEN)

- **1차(권고)**: `is_hwp3_origin` 변수 + tolerance 블록(264-268) 제거. `hwpml_version`
  보존 라인은 유지. → Stage1 RED 테스트 GREEN.
- **조건부**: Stage1에서 HWP3 변환본 회귀 확인 시, 완전 제거 대신 진짜 변환본만 살리는
  보수 가드 적용(예: `uses_hwp3_origin_page_tolerance` 류 콘텐츠 휴리스틱을 파싱 시점에서
  재사용). 이 경우 RED 테스트는 "네이티브는 0, 변환본은 비-0" 로 조정.
- Stage2 완료보고서(`_stage2.md`) → 승인 요청.

### Stage 3 — 게이트 검증 + 최종 보고

수행계획서 §6 전 게이트:
1. `python tools/render_page_gate.py --save output/poc/task1608_after.tsv` → before(baseline)
   대비 개선−회귀 > 0 (목표 net +6). before/after tsv diff 첨부.
2. `cargo test --test hwpx_roundtrip_baseline` 회귀 0.
3. HWP3 변환본 시각 회귀: `samples/hwp3-sample*` export-svg / 페이지수 점검.
4. #1589 붕괴 회귀 0, visual_roundtrip 회귀 0.
5. `cargo test`(라이브러리) + `cargo clippy`(수정 범위) 무경고.
6. `_stage3.md` + `_report.md` + `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` 갱신(요인 A 해소 기록).

## 산출물

| 종류 | 경로 |
|------|------|
| 소스 | `src/parser/hwpx/mod.rs` |
| 테스트 | `tests/issue_1608_hwpx_native_no_hwp3_tolerance.rs` (또는 mod 내부) |
| 측정 | `output/poc/task1608_baseline.tsv`, `output/poc/task1608_after.tsv` |
| 보고 | `_stage1.md`, `_stage2.md`, `_stage3.md`, `_report.md` |
| 갱신 | `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` |

## 회귀 위험

- 전 HWPX 코퍼스에서 본문 fit 판정 +21px 제거 → 경계 문서 페이지 분할 변동. 통제셋 게이트
  (net>0)로 방어. 네이티브 2건(36382819·36395325) +회귀는 측정상 수용(net +6 우세).
- 진짜 HWP3 변환본 tolerance 상실 위험은 Stage1 측정으로 사전 차단(가드 fallback 보유).
