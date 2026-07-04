# 설계 문서 — #1658 페이지네이션 엔진 개선 (다세션 연구·설계)

- 마일스톤: M100 / 브랜치: local/task1658 / 착수: 2026-06-29
- 목표: rhwp TypesetEngine 페이지네이션을 OLE 한글 기준에 수렴(현 정합 ~93% → 향상), **무회귀**.
- 오라클: OLE 한글 PageCount (Windows+한컴). 통제 게이트가 모든 변경의 심판.

## 0. 핵심 발견 (6종 실패 → 안정 메커니즘 규명)

#1653 에서 incremental 수정 6종이 모두 통제 게이트 회귀(진동·tip-over). 두 엔진 코드 대조로
**안정 메커니즘**을 규명:

### Paginator(engine.rs:593-638) — 안정적인 이유
페이지에 **블록(비-TAC) 표가 있으면**, 직전 문단의 **저장 line-seg vpos**(page_vpos_base 기준)로
`current_height` 를 **스냅(max, 절대 감소 안 함)** 한다:
```
vpos_h = px(prev_seg.vertical_pos + lh + ls - page_vpos_base)
if vpos_h > current_height && vpos_h <= avail { current_height = vpos_h }
```
→ flow 누적을 한글의 **실측 vpos 위치**에 정렬. 하단 고정 표 등 flow 밖 개체의 효과가 저장 vpos 에
이미 반영돼 있으므로, 거기에 스냅하면 자연히 한글과 일치. **reserve 가 아니라 ground-truth 스냅 →
자기 앵커를 밀지 않음 → 진동 없음.**

### TypesetEngine(typeset.rs:2364-2376) — 약한 이유
`vpos_snap_current_height`(8454, HeightCursor::vpos_adjust 사용)를 **`!has_table` 텍스트 문단에만**
적용. **블록 표가 있는 페이지의 표 높이 누적 drift 를 vpos 로 교정하지 않는다.**
→ 패턴 B(페이지 하단/블록 표) over-pagination 의 구조적 원인.

## 1. 전략 (발명 금지 — 안정 메커니즘 이식)

reserve/2-pass 를 발명하려다 6번 실패. 대신 **Paginator 의 검증된 vpos-snap 을 TypesetEngine 에
이식·정합**한다:
- 블록 표가 있는 페이지에서 TypesetEngine 도 직전 문단 저장 vpos 로 current_height 를 스냅(max).
- TypesetEngine 의 기존 HeightCursor 자산을 활용하되, **표 문단 경계에서도** 스냅이 동작하도록 게이트 확장.
- 텍스트 흐름(패턴 A)은 이미 강하므로(75/92) 건드리지 않거나, 동일 vpos-snap 으로 sb/sa 드리프트도
  부수적으로 수렴하는지 측정.

## 2. 측정 기반 (오라클)

- 소형: `tests/fixtures/render_page_controlset.tsv`(92, 기존 #1600). 일치 75 / −1 12 / +1·+2 5.
- 대형(신규, 본 이슈): `tests/fixtures/render_page_oracle_1658.tsv`(~600 층화 랜덤, seed=1658).
  `tools/build_page_oracle.py` 로 한글 PageCount 수집. 92건 과적합 방지·통계 신뢰도 확보.
- 게이트: `tools/render_page_gate.py --fixture <oracle>`. **합격 = over↓ & under/일치 무회귀(신규 −1=0).**

## 2-1. Phase 0 결과 — 대형 오라클 베이스라인 (중요 재맥락화)

`render_page_oracle_1658.tsv` (랜덤 452, ERR 148 제외) 베이스라인:

| delta | 건수 | 비율 |
|------|------|------|
| 0 (일치) | **441** | **97.6%** |
| −1 (under) | 7 | 1.5% |
| +1/+2/+8 (over) | 5 | 1.1% |

- **controlset(92) 81.5% 는 의도적 난케이스 모음**, 랜덤 실제 정합률은 **97.6%**. 두 게이트 상보적.
- **분포(불일치 9~12건) 패턴 재확인**:
  - **over(+) 전부 `law 별표 .hwp`** — 대형 법령 표의 **행분할 밀도 차이**(delta 최대 +8).
    controlset 의 page-bottom 표(PC/관악)와 **다른 문제**. → 스케일 지배 over 원인.
  - **under(−1)** = 결재 .hwpx(page-bottom/블록 표) + 별표 혼합.
- 시사점: Phase 1 우선순위는 **대형 오라클 빈도**로 결정. 현재 over 지배항 = **표 행분할(법령)**,
  page-bottom 표(vpos-snap)는 controlset 가시·스케일에선 under 측. 둘 다 블록 표 vpos 정합과
  연결되므로 vpos-snap 이식이 공통 기반일 가능성.
- 한계: 오라클 ERR 148(주로 일부 .hwp open 실패) → 향후 .hwp 견고화 + 표본 확대 필요.

## 3. 단계 계획 (다세션)

- **Phase 0 (이번 세션)**: 안정 메커니즘 규명(완료) + 대형 오라클 구축 + 설계 문서. 소스 무회귀.
- **Phase 1**: 블록 표 페이지 vpos-snap 이식 (Paginator 정합). 게이트(소형+대형)로 회귀 0 확인하며
  조건 점진 확장. 목표: 패턴 B over 해소, under/일치 무회귀.
- **Phase 2**: 잔여 패턴 A(텍스트 sb/sa) 가 vpos-snap 으로 수렴하는지 측정 → 미수렴분만 별도 처리.
- **Phase 3**: 대형 오라클 전수 + lib/hwpx_roundtrip + 시각 회귀(대표 PDF) 검증, 보고.

## 4. 불변 원칙

- 모든 변경은 **양 오라클 게이트 통과(무회귀)** 후에만 채택. 회귀 시 즉시 롤백.
- 기본 엔진 교체 금지(Paginator 전역=44/92 입증). **TypesetEngine 에 안정 메커니즘 이식**.
- 단계마다 working/ 보고 + 소스 커밋, 게이트 수치 명시.

## 5. 위험

- HeightCursor 의 표 경계 스냅은 미주/다단/TAC 수식과 상호작용 → 좁은 게이트로 점진 확장 필수.
- 대형 오라클도 표본이므로, 최종은 더 큰 표본 + 시각 판정 병행.

## 6. 제보 케이스 — centered-cell over-count (kkyu8925, 2026-06-30)

GitHub #1658 코멘트(kkyu8925) 제보. "저장 line-seg vpos 권위" 방향과 직결되는 셀 세로정렬 케이스.

- **증상**: 머리행+안내문단+중첩표2개가 든 자리차지 표(큰 셀 `valign=Center`)에서, 셀 내용이
  한컴은 세로 중앙·rhwp 는 **상단** 정렬. 안내 문단 y ≈ 122pt(rhwp) vs ≈ 247pt(한컴), 약 125pt 차.
  표 전체 높이(≈846px)는 동일.
- **근원**: `src/renderer/layout/table_cell_content.rs:650-676` (devel `30931679`). `has_nested` 분기에서
  `total_content_height = max(last_seg_end(저장), calc_composed_paras_content_height(계산),
  calc_nested_controls_bottom_height(계산))` 가 `inner_height` 에 근접 → `Center` offset
  `(inner_height - total_content_height)/2 ≈ 0` → 상단 정렬.
- **양방향 충돌**: #44(셀 내부 표 세로중앙 오류, CLOSED)는 **under-count**(중첩표 높이 미반영 → 과소),
  본 케이스는 **over-count**. computed/stored 선택이 양방향으로 어긋남 → 국소 단순치환(저장 권위화)은
  #44 를 정반대로 회귀.
- **처리 방향**: 본 #1658 통합 vpos 회계(저장 vpos 절대좌표 정렬 + 중첩 컨트롤 바닥 일관 회계)에
  centered-cell 케이스를 편입해 일관 해소. 합성 fixture(중첩표 든 centered 셀, 실문서 비포함) 제보자가
  제작 제안 — render_page_gate / valign Center offset 회귀 게이트에 편입 예정.
