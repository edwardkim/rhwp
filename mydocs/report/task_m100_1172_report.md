# 최종 보고서 — Task M100-1172: rhwp-studio 문단모양 여백 2배 + 테두리/배경 결함

## 1. 이슈

#1172. rhwp-studio 서식 > 문단모양에서:
1. 왼쪽 여백 10.0pt 지정 후 다시 열면 **20.0pt** (2배)
2. **여백만 바꿔도 문단 테두리/배경이 생성됨** (적용 즉시 화면에)

## 2. 진단 (e2e + dump 교차)

작업지시자 제공 샘플(`samples/para-001.{hwp,hwpx}`, 결함 재현본 `saved/para-001-s.hwp`)과
호스트 CDP e2e (`getParaPropertiesAt` / `getPageLayerTree`) 로 두 결함의 근본을 규명.

### 결함 A — 여백 2배
- ParaShape margin/indent IR 값은 **2× 스케일** (HWP5 바이너리 원본, HWPX 도 parser `val2x` 로 통일). 즉 1pt = 200 HWPUNIT. **한컴 정답: IR 2000 = 10.0pt.**
- `build_para_properties_json` (formatting.rs) 이 dialog 표시 px 계산 시 표준 `hwpunit_to_px`(7200/inch, 1× 가정)를 그대로 적용해 2× 스케일을 ÷2 환산하지 않음 → 2배 표시.
- frontend 쓰기 `ptToRaw2x`(×2) 는 IR 2× 스케일과 정합하므로 올바름.

### 결함 B — 여백만 바꿔도 테두리/배경 생성
- 2번째 문단 `patternType = -1` (무늬 없음).
- 문단모양 "무늬 모양" select 옵션이 `'0'`~`'6'` 만 있고 **`-1` 없음** → patternType=-1 표시 시 0 으로 폴백.
- `collectMods`: `newPatType(0) !== (p.patternType ?? 0)` → `0 !== -1` = true → `mods.patternType = 0` → `mods.fillType = 'solid'` 강제 주입.
- → props_json 에 fill 키 포함 → `create_border_fill_from_json` 이 새 BorderFill 생성(pat_type -1→0) → 문단 배경/테두리 생성.
- 즉 여백 변경/확인만 해도 patternType=-1 문단에 배경이 생김.

### 부수 — HWPX winBrush faceColor="none" 오파싱
- `header.rs` winBrush 처리가 존재만으로 `FillType::Solid` → `faceColor="none"`(배경 없음)이 흰색 Solid 로 잘못 파싱.

## 3. 정정

| 파일 | 변경 |
|------|------|
| `src/document_core/commands/formatting.rs` | `build_para_properties_json` margin/indent 표시 px 를 `hwpunit_to_px(raw / 2)` 로 (2× 스케일 → 1× 환산) |
| `rhwp-studio/src/ui/para-shape-tab-builders.ts` | 무늬 '없음' 옵션 value `0` → **`-1`** (IR patternType 정합) |
| `rhwp-studio/src/ui/para-shape-dialog.ts` | collectMods patternType 비교 기본값 `?? 0` → `?? -1`, NaN 방어 |
| `src/parser/hwpx/header.rs` | winBrush `faceColor="none"` + 무늬 없음 → `FillType::None` |
| `tests/hwpx_to_hwp_adapter.rs` | task888 `border_fills_no_fill_normalized` 기대 1→0 (파서 단계 처리로 어댑터 카운트 감소, 최종 no-fill 보장 단언 유지) |

## 4. 검증

| 항목 | 결과 |
|------|------|
| `tests/issue_1172_*` (신규) | ✅ 여백 10pt + HWPX fillType=none |
| 호스트 CDP e2e | ✅ 여백 변경 시 `mods={"marginLeft":2000}` (fill/border 미포함) |
| `cargo test --tests` 전수 | ✅ 실패 없음 (task888 포함) |
| `cargo fmt --check` / `clippy --lib` | ✅ clean |
| TS 타입체크 | ✅ tsc=0 |
| **작업지시자 시각 판정** | ✅ 통과 (여백 정상 + 테두리 미생성) |

## 5. 메모리 룰 정합

- `feedback_no_inference_authoritative_spec` — 한컴 정답(10pt) + e2e 실측으로 2× 스케일 확정, alpha=0 추측 정합 시도를 멈춤
- `feedback_hancom_compat_specific_over_general` — HWP5 alpha=0 Solid 를 일괄 None 처리하지 않음(실제 흰배경 보존). 결함 본질은 dialog patternType select 매핑
- `feedback_visual_regression_grows` — dump 정량 + e2e + 작업지시자 시각 판정 병행
- `feedback_image_renderer_paths_separate` — HWPX(winBrush) / HWP5 / dialog 경로 각각 점검

## 6. scope 밖

- HWP5 첫 문단 `Solid alpha=0 흰배경`은 dialog fillType=solid 로 표시되나 실제 렌더 무영향(treeSolidFillCount=0). 결함 B(patternType) 와 무관하므로 본 이슈 정정 대상 아님. 향후 dialog 표시 정합이 필요하면 별도 이슈.
