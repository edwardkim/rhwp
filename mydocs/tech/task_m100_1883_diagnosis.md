# Task #1883 1단계 — 현황 재진단 (마지막 리팩토링 이후, 거버넌스 2축 정량)

- 이슈: #1883 (umbrella #1582) / 측정일: 2026-07-04 / 기준 커밋: devel `4edaa23d`
- 거버넌스: SOLID + 복잡도 — 복잡도 임계값은 **코드 품질 대시보드**
  (`scripts/metrics.sh` → `output/dashboard.html`, [manual/dashboard.md](../manual/dashboard.md)) 기준:
  **파일 1,200줄 상한 / CC 목표 ≤15·경고 >25 / clippy 0 / 테스트 실패 0**.

## 1. 복잡도 축 — 4차 리뷰(2026-03-23) 대비 재축적

### 1.1 코드베이스 규모 (×2.7 성장)

| 지표 | 4차 리뷰 | 현재 | 변화 |
|---|---|---|---|
| src 총 라인 | 133,107 | **356,162** | ×2.7 |
| .rs 파일 수 | 317 | 442 | +125 |

측정: `find src -name "*.rs" -exec wc -l {} +` (재현 가능).

### 1.2 파일 핫스팟 (대시보드 상한 1,200줄 기준)

| 파일 | 4차 리뷰 | 현재 | 변화 |
|---|---|---|---|
| `renderer/typeset.rs` | (4차 표 미등재) | **15,221** | 최대 로직 파일로 급성장 |
| `document_core/commands/object_ops.rs` | 3,365 | 9,845 | ×2.9 |
| `renderer/layout.rs` | 2,659 | 8,466 | ×3.2 |
| `parser/hwpx/section.rs` | 2,530 | 7,310 | ×2.9 |
| `wasm_api.rs` | 3,742 | 6,798 | ×1.8 (2차 리팩토링 이전 24,000 대비는 여전히 −72%) |
| `renderer/layout/table_layout.rs` | 1,904 | 6,765 | ×3.6 |
| `renderer/layout/paragraph_layout.rs` | 2,355 | 5,997 | ×2.5 |

(자동 생성 데이터 파일 `font_metrics_data.rs` 45,951 / `johab_map.rs` / `pua_oldhangul.rs` 는
대시보드 관례대로 제외.)

### 1.3 함수 핫스팟 (fn 선언 간격 근사 스캔, 재현 스크립트 본문 명기)

| 줄수 | 함수 | 비고 |
|---|---|---|
| **7,059** | `typeset.rs::typeset_section_with_variant` | **4차 경고(827줄)의 8.5배** — 최우선 해체 후보 |
| 3,771 | `paragraph_layout.rs::layout_composed_paragraph` | |
| 2,267 | `hwp3/mod.rs::parse_paragraph_list` | |
| 1,615 | `table_layout.rs::layout_table_cells` | |
| 1,612 | `table_partial.rs::layout_partial_table` | |
| 1,428 | `layout.rs::build_single_column` | |
| 1,261 | `typeset.rs::typeset_block_table` | |
| 1,140 | `height_cursor.rs::vpos_adjust` | |
| 838 | `pagination/engine.rs::paginate_with_measured_opts` | 3차에서 1,456→120 해체했던 함수의 후신이 재성장 |
| 505 | `layout.rs::layout_column_item` | 4차 경고 827 → **505 (③차 정리 효과 확인)** |

측정: fn 선언 라인 간격 근사(python re 스캔). 정밀 CC 는 대시보드 3단계(clippy
cognitive_complexity) 수치를 §3 에 병기.

### 1.4 해석

- ③차 정리가 지목했던 지점(`layout_column_item`)은 개선 유지 — **정리 자체는 효과가 있다.**
- 그러나 페이지네이션 fidelity 집중 구간(4~7월)에 **typeset.rs 단일 함수가 7천 줄로 성장** —
  4차 리뷰가 "점진적 개선으로 충분"이라 본 규모를 초과했고, #1582 감사의 "layout 경로에
  소스-포맷·개체 처리 detail 혼재" 지적과 일치한다.
- 계획서의 복잡도 성공 기준은 대시보드 임계값(1,200줄/CC 15·25)으로 정량화한다.

## 2. SOLID 축 — #1582 감사 지점 실측 확인

| 감사 지적 | 실측 (devel `4edaa23d`) | 판정 |
|---|---|---|
| `Document` 에 canonical/보존/provenance 축 혼재 | `extra_streams`·`hwpx_aux_entries`(package 보존) + `is_hwp3_variant`(provenance) 필드 잔존 확인 (`model/document.rs:39-48`) | **일치** — 1단계 분리 대상 |
| `DocumentCore` = session aggregate | `document_core/` pub fn **277개** (문서 상태+layout+undo+validation+view 혼재) | 일치 |
| `lib.rs` public surface 과대 | pub mod/use **18개** | 일치 (호환성 결정 없이 내부 변경 곤란) |
| layout 경로 소스-포맷 detail 혼재 | `typeset_section_with_variant` 7,059줄이 그 집적점 | 일치 |

## 3. 대시보드 지표 (scripts/metrics.sh, 2026-07-04 실측 — 공식 측정)

| 대시보드 카드 | 임계값 | 측정값 | 판정 |
|---|---|---|---|
| 1,200줄 초과 .rs 파일 | 0 목표 | **70개** | 🔴 |
| Clippy 경고 | 0 | 0 | 🟢 |
| CC > 25 함수 (경고) | 0 목표 | **80개** (CC>15 목표 초과는 169개, 수집 779개 중) | 🔴 |
| 테스트 | 실패 0 | 2,820 passed / 0 failed | 🟢 |
| 커버리지 | — | null (tarpaulin 장시간 소요로 이번 측정 생략 — 스크립트 실패 허용 설계) | ⚪ |

### CC 상위 (clippy cognitive_complexity — §1.3 함수 근사 스캔과 상호 검증됨)

| CC | 위치 | (§1.3 대응 함수) |
|---|---|---|
| **288** | `paragraph_layout.rs:1458` | `layout_composed_paragraph` (3,771줄) |
| **282** | `typeset.rs:2007` | `typeset_section_with_variant` (7,059줄) |
| 234 | `hwp3/mod.rs:428` | `parse_paragraph_list` |
| 163 | `table_partial.rs:85` | `layout_partial_table` |
| 119 | `table_layout.rs:2292` | `layout_table_cells` |
| 116 | `main.rs:4890` | (CLI dispatch 계열) |
| 114 | `layout.rs:3516` | `build_single_column` |
| 110 | `typeset.rs:11961` | `typeset_block_table` |

두 측정(대시보드 CC vs 함수 크기 스캔)의 상위권이 일치 — 해체 우선순위의 신뢰도를 상호
검증한다. 경고 임계(25)의 **10배를 넘는 함수가 2개**(288/282).

## 4. 재현 방법

```bash
# 파일/총계
find src -name "*.rs" -exec wc -l {} + | sort -rn | head -16
# 대시보드 (공식 측정)
./scripts/metrics.sh && xdg-open output/dashboard.html
# 함수 근사 스캔: fn 선언 간격 (본 문서 1.3 — python re 스캔, 데이터 파일 3종 제외)
```

## 5. 결론 (계획서 입력)

1. 재진단 결과는 "리팩토링 실행 전 계획 수립" 판단을 정량으로 뒷받침 — 3.5개월간 ×2.7 성장,
   함수 재축적 8.5배.
2. 우선 해체 후보: `typeset_section_with_variant`(7,059) → `layout_composed_paragraph`(3,771)
   → 표 계열(1,２~1,6천 4개). 단 **#1582 1단계(SourceProvenance/LayoutCompatibilityProfile
   분리)가 선행**돼야 typeset/layout 해체 시 소스-포맷 분기가 policy 로 빠져 안전해진다.
3. 성공 기준은 대시보드 임계값으로 정량화 + 6차 리뷰(SOLID 점수 재평가)로 마감.
