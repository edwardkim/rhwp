# PR #1844 검토 보고서 — 같은 문단 float 스택 이월 규칙 (사후 회귀 발견)

- PR: https://github.com/edwardkim/rhwp/pull/1844
- 제목: `Task #1831: 같은 문단 float 스택 이월 규칙 — 다쪽 표 continuation 상수 오프셋 해소`
- 작성자: planet6897
- 연결: #1831
- base ← head: `devel` ← `pr/devel-1831` (head merge commit `ff0c7ac2`)
- 상태: **MERGED** (본 문서는 merge 후 5200건 회귀 서베이에서 발견한 사후 검토)
- 검토일: 2026-07-03 (**조사 전용 — 소스 수정 없음**)

## 1. 요약 판단 — merge 후 표 이월 회귀 3건 확인

작업지시자 지시로 pi-page vs 한글 정합 서베이를 **문서 5,200건**(≥5000)으로 재실행했다.
샘플: `output/poc/survey_pipage_pr1844/`, 도구: `tools/verify_pi_page_vs_hangul.py`,
바이너리: PR #1844 head(`ff0c7ac2`) release 빌드.

총계는 devel 대비 거의 불변(MATCH 4911 / PAGE_DELTA 154 / PI_MISMATCH 132 / ERR 2 /
PARA_COUNT 1)이나, **파일 단위로 PR #1844 기인 페이지수 변화 3건이 전부 +1쪽(회귀)** 이다.
devel 단독 바이너리와 교차 대조하여 devel 드리프트와 분리했다.

| 파일 | devel | PR #1844 | 한글(정답지) | 판정 전이 |
|---|---|---|---|---|
| 156767631 서울 금성당 무신도 | 5 | **6** | 5 | MATCH → PAGE_DELTA (직접 회귀) |
| 78842 데이터기반행정 조문별 이유서 | 52 | **53** | 52 | PI_MISMATCH → PAGE_DELTA |
| 3143097 과태료 납부 독촉 | 3 | **4** | 1 | PAGE_DELTA(Δ+2 → Δ+3 악화) |

세 케이스 모두 `wrap=자리차지`(TopAndBottom) 표의 이월 처리에서 발생하며, **단일 근본
원인**으로 수렴한다(§3). 한글 정답지 기준으로 세 건 전부 devel 이 더 정확하거나 동률이고
PR 이 더 이탈한다. 특히 156767631 은 MATCH 를 깨뜨린 직접 회귀다.

## 2. 변경 범위

31 files +1497/-46. 레이아웃 코어 변경은 다음에 집중:

| 파일 | 핵심 |
|---|---|
| `renderer/typeset.rs` | +67 — **같은 문단 float 스택 통째-이월 규칙** + 단 상단 whole-table-fit 2px 허용 |
| `renderer/height_measurer.rs` | +53/-… — 행높이 측정 정리 |
| `renderer/layout/{paragraph,table}_layout.rs` | prefill_before_deferred_table 경로 |
| `serializer/control.rs` (+tests) | 직렬화 |
| `main.rs` | +173 — 진단 서브커맨드 |
| `tools/{compare_line_baselines,patch_cell_flags}.py` | 측정·패처 |

회귀 원인은 `typeset.rs` 의 이월 규칙 블록 1곳으로 국소화된다.

## 3. 근본 원인 — `preceded_by_same_para_float` 술어의 과잉 포착

`src/renderer/typeset.rs` (약 12277줄, PR #1844 신규):

```rust
let preceded_by_same_para_float = st.current_items.iter().any(|it| match it {
    PageItem::Table { para_index, .. } | PageItem::PartialTable { para_index, .. }
        => *para_index == para_idx,
    _ => false,
});
if row_count > 1 && preceded_by_same_para_float {
    // 잔여 공간에 표 전체가 안 들어가면 → prefill 후 통째로 다음 쪽/단 이월
    let remaining_now = (table_available - st.current_height - first_frag_overhead).max(0.0);
    if total_rows_h + caption_base_overhead > remaining_now {
        self.prefill_before_deferred_table(...);
        st.advance_column_or_new_page();
    }
}
```

이 술어는 **`para_index` 만 비교하고 `ci`·`tac` 을 구분하지 않는다.** 그 결과 표 자신의
캡션/제목 상자(`tac=true` 하위 표)나 같은 문단에 앵커된 다른 상자까지 "선행 형제 float"
로 오분류하여, 원래 **분할되어야 할 본체 표를 통째로 다음 쪽으로 밀어낸다**.

### 3.1 세 회귀의 트리거 (dump-pages 직접 확인)

| 파일 | 선행 "float" 의 실체 | devel(정상) | PR #1844(회귀) |
|---|---|---|---|
| 156767631 | 본체 pi66 ci=0 앞의 **캡션표 ci=1 (tac=true, 1×1)** | p4 에서 rows 0..7 분할 시작 → 5쪽 | 캡션만 p4, 본체 통째 p5 이월 → 6쪽 |
| 78842 | 본체 pi371 ci=1 앞의 **캡션표 ci=0 (tac=true, 1×3)** | p44 에서 rows 0..2 분할 시작 → 52쪽 | 캡션만 p44, 본체 통째 p45 이월 → 53쪽 |
| 3143097 | pi2 에 매달린 **`vert=용지` 절대위치 상자 22개**(ci=14..35)가 서로를 선행 | 상자 분할·조밀 배치 → 3쪽 | 모든 다행 표 통째 이월, 분할 전무 → 4쪽 |

트리거가 세 건 모두 동일하다 — **같은 anchor 문단(`para_index`)을 공유하는 `tac=true`
캡션 상자(또는 페이지-절대 앵커 상자)가 "선행 float"로 잡혀, 본체 표가 현재 쪽에서 분할
시작하지 못하고 통째 이월되어 페이지가 +1 된다.**

이 규칙은 2448877(별표4)의 진짜 "표1 → 표2 본체 스택" 정합을 위해 설계됐으나
(`output/poc/task1831/` 실측), 술어가 의도보다 넓어 다음을 오검출한다:

1. **표 자신의 `tac=true` 캡션 하위 표** — 156767631(캡션 ci=1), 78842(캡션 ci=0).
   본체의 캡션은 "선행 형제 float"가 아니다.
2. **`vert=용지`(page-relative absolute) 앵커 상자** — 3143097. flow 스택 멤버가 아니라
   용지 고정 개체이므로 이월 그룹 대상이 아니다. 서식 양식(독촉장)에서 각 필드 상자가
   용지 절대 좌표에 배치된 구조이며, 한글은 전부 앵커 문단이 있는 1쪽에 배치한다.

## 4. outlier 참고 (PR #1844 무관, 기존 갈래)

동일 서베이의 |Δ| 최대 이탈 2건 — PR #1844 와 무관(devel 동일)하나 표 처리 계통 기록:

- **3015131 해사노동 점검표 (Δ−13, rhwp 48 vs 한글 61)**: 145행×9열 대형 RowBreak 표
  2개가 각 22~23쪽에 걸침. rhwp 가 쪽당 ~9-10행, 한글 ~6-7행 — continuation 행높이
  계통 과소로 13쪽 적음. PAGE_DELTA −1 과소군(59건)의 대형 극단.
- **3143097 (Δ+3)**: §3.1 대상. devel 부터 이미 과대(3쪽, 한글 1쪽)였고 PR 이 4쪽으로 악화.

## 5. 전체 분포 참고 (PI_MISMATCH 132 / PAGE_DELTA 154)

- **PAGE_DELTA**: ±1쪽 134건(87%). 과대 80 / 과소 74 로 거의 대칭. +1 과대의 최대 원인은
  마지막 쪽이 표 이월인 케이스(43건/57%) — PR #1844 가 손대는 바로 그 영역. −1 과소는
  마지막 쪽을 800px+ 로 꽉 채운 razor-thin 26건 + 여유 있는 실계산 과소 33건.
- **PI_MISMATCH**: 단일 문단 플립 72건(55%). 방향 rhwp 당김:밀림 = 88:35 (rhwp 가 문단을
  이전 쪽에 배치하는 편향 2.5배). 단일 pi 72건 중 표 float/tac 관련 12건은 캐럿-개체 분리
  오탐(#1757) 후보, 나머지 57건은 실제 쪽 경계 배치 차이.

## 6. 정정 방향 (제안 — 별도 이슈/브랜치/계획 절차 대상)

`preceded_by_same_para_float` 이월 그룹을 "**같은 문단의 flow 스택 본체 표**"로 한정:

1. **`tac=true` 캡션 하위 표 제외** — 본체 표의 캡션은 선행 형제 float 가 아니다.
2. **`vert=용지`(페이지-절대 앵커) 상자 제외** — 문단-상대(`vert=문단`) 앵커만 flow 스택
   멤버로 취급.

즉 술어를 `para_index 동일 AND tac=false AND vert=문단-상대` 로 좁히면 별표4(2448877)
정합을 유지하면서 세 회귀를 해소한다. **회귀 게이트로 156767631·78842·3143097 을 추가**
(특히 156767631 은 MATCH→PAGE_DELTA 직접 회귀이므로 최우선).

정정 자체는 소스 변경이므로 하이퍼-워터폴 절차(이슈 → 브랜치 → 수행계획 → 구현계획 →
단계 진행)에 따르며 작업지시자 승인 후 착수한다.

## 7. 검토 결론 (작업지시자 판단용)

| 항목 | 평가 |
|---|---|
| PR 방향성 | 진지함 — 별표4 다쪽 표 continuation 상수 오프셋 해소 목적, 실측 자산(`output/poc/task1831/`) 동반 |
| merge 후 상태 | **표 이월 회귀 3건**(전부 +1쪽), 단일 근본 원인(§3) |
| 근본 원인 | `preceded_by_same_para_float` 술어가 tac 캡션·페이지-절대 앵커까지 과잉 포착 |
| 정정 난이도 | 낮음 — 술어에 tac/anchor 가드 2개 추가, 국소 수정 |
| 회귀 게이트 | 156767631·78842·3143097 추가 필요 |
| 5197건 | devel 과 동일하거나 devel 드리프트로 개선(회귀 무관) |

이미 merge 되었으므로 롤백이 아니라 **후속 정정 PR** 로 처리하는 것이 자연스럽다. 술어
범위 축소 1건 + 회귀 게이트 3건 추가로 별표4 정합을 유지한 채 세 회귀를 닫을 수 있다.

## 8. 산출물

- 본 검토 보고서: `mydocs/pr/pr_1844_review.md`
- 서베이 데이터: `output/poc/survey_pipage_pr1844/master.tsv` (5,200행) + `driver.log`
- baseline 대조: `output/poc/survey_pipage/master.tsv` (동일 5,200건, cde8ca6e)
- 조사 스크립트: `analyze_mm.py`(분포), `classify_mm.py`(유형), `recheck.py`(페이지수
  전수 대조), `trigger.py`/`t78842.py`(트리거 검증) — 임시 스크래치패드
