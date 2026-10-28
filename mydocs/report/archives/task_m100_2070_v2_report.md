# 최종 보고서 — Task M100 #2070 v2: RowBreak 대형 표 밀도 (행미 공백 유령 줄 + aim 패딩 0 존중)

- 이슈: #2070 잔여 / 브랜치: `fix/2070-rowbreak-density` / 작성일: 2026-07-11
- 계획서: `mydocs/plans/task_m100_2070_v2.md`
- PR #2198 리뷰 후속 (maintainer 지적 항목 포함 처리)

## 원인 (계측·한글 PDF 실측 확정)

시장구조조사(RowBreak 변종 최대 인스턴스) 행 피치: rhwp 50.4px vs 한글 21.9px
(= 선언 셀높이 1650HU=22.0px 정확 일치, PDF 직독). 50.4 = Fixed(37.8) + em(12.6):

1. **행미 공백 유령 줄**: 셀 재래핑이 trailing 공백 포함 폭으로 분할 판정 →
   "100.0␣␣"가 ["100.0","␣␣"] 2줄. 한글은 행미 공백 hanging(폭 판정 제외).
2. **aim=true 패딩 0 무시**: `use_cell_padding_axis`의 `!= 0` guard(task1443
   유래·근거 기록 없음)가 pad=0 셀을 표 패딩 폴백 → 내부폭 40.1→26.5px →
   8자리 코드 2줄 분할. **한글 PDF 실측: 코드 렌더 폭 37.0px > 26.5** —
   물리적으로 폴백 불가, 0 존중이 정답 (resolve_cell_padding 주석과 일치).

## 수정

- `composer.rs`: 분할 판정 폭에서 행미 공백 제외 + 공백-단독 분할 조각 흡수
- `model/table.rs`: aim=true `!= 0` → `>= 0` (음수 결측 센티널만 폴백)
- `style_resolver.rs`: **비-Percent 줄간격(Fixed/SpaceOnly/Minimum) /2** —
  저장값이 유효의 2배 (통제 사다리 #2197 계약 + 실문서 이중 실증: 본문 ps
  Fixed 3320HU, 한글 PDF 줄 pitch 22.1px=3320/2, rhwp 종전 44.3px; 편람 한컴
  HWPX case 1560/default 3120). 여백·문단간격·탭과 동일 규약으로 정렬.
- `issue_1785` 규칙 테스트 갱신, `table-text` golden 갱신(≤1px justify 재분배)

## 검증 (한글 2022 COM per-pi 오라클 8건 + 전체 게이트)

| 문서 | 수정 전 | 수정 후 | 기준 |
|---|---|---|---|
| 시장구조조사 | 606 | **307** | 315 (잔여 −8, 과소 반전 — 별도 서브축) |
| issue2063 화성시 별표2 | 159 | 159 | 162 (잔여 −3) |
| 80168 / 80250 / sample16 / exam_kor | 157/17/64/20 | 동일 | 쪽수 정합 유지 |
| byeolpyo4 / KTX | 26/27 | 동일 | 기존 잔차 불변 |

- cargo test 전체 green (규칙·golden 갱신 포함) / fmt 0 / clippy 0
- 핀: `tests/issue_2070_rowbreak_density.rs` (시장구조조사 307 잠정·issue2063 159 잠정 — 기준 PDF 값 복귀가 목표)

## PR #2198 리뷰 지적 처리

- `tests/golden_svg/form-002/page-0.actual.svg` 제거 ✓
- 검증 자산 커밋: `samples/task2070/1130000-...시장구조조사.hwp` +
  `pdf/task2070/...-2022.pdf`(315쪽, COM HPrint 1-up) ✓ — 원문 타깃 축은 기존
  `samples/issue2063_huge_cellbreak_table.hwp` + `pdf/issue2063_...-2020.pdf`(162쪽)를
  핀 테스트로 연결 ✓
- 잔여 기록: 시장구조조사 −8(과소), issue2063 −3, 80168 p108 `line_band_drift`/
  `render_tree_frame_tail_overflow` 후보 (visual sweep), 76076/86712 잠정
  기대값은 #2195/#2197 소관
- #2110/#2136/#2137/#2138 문서 혼입: 각 이슈의 정식 산출물로 저장소 유지가
  타당하여 존치 (혼입 자체는 절차 실수로 기록)
