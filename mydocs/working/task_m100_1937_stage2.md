# #1937 Stage 2 — 정확 지점 특정 + 수정 설계

- 브랜치: `local/task1937` / 샘플: `소상공인 중간보고서(2).hwp` (rhwp 231쪽 vs 한글 50쪽)
- 방법: 내장 진단(`RHWP_TABLE_DRIFT`)·임시 계측(측정 후 원복, 소스 무변경 유지)

## 1. 결론 — 행높이 문제 아님. **연속 페이지가 시작 페이지의 각주 예약을 재사용**하는 분할 버그

Stage 1 의 "행높이 과다" 가설을 실측으로 **기각**하고, 진짜 원인을 확정했다.

- 231쪽 폭주의 **180쪽이 단 하나의 표(pi=306, 122행×6열)** 에서 발생. 나머지 문서는
  한글과 쪽수 정합(누적 delta 0 → pi=306 에서 +180 급증).
- 행높이는 정상: `cut_row_h`≈`mt.row_heights`≈33~113px, 합 5733px ≈ 6.5쪽분(정상).
- 문제는 **분할 루프의 페이지 가용높이**:

| | 시작 페이지 available | 연속 페이지 page_avail | 페이지당 행 | 결과 |
|---|---|---|---|---|
| pi=99 (정상) | 895.8px | **895.8px** | ~18행 | 8 fragment (정상) |
| pi=306 (버그) | **75.8px** | **75.8px (재사용)** | ~1행 | **187 fragment** |

## 2. 근본 원인 (코드 경로)

1. `typeset_block_table`(typeset.rs:12306) 에서 표 가용높이를 **한 번** 계산:
   `available = base_available − total_footnote − fn_margin − zone` (12359).
2. `total_footnote = projected_footnote_height(cnt=0 →) current_footnote_height`
   (typeset.rs:1504-1505). pi=306 표 자체 각주 0개지만, **표가 시작하는 페이지에
   본문 각주가 이미 ~820px 쌓여 있어** `current_footnote_height=820` → available=75.8px.
3. `table_available = available − tol = 75.8` (12812) 을 분할 while 루프가
   **모든 연속 페이지에 재사용**: `page_avail = if is_continuation { table_available }`
   (typeset.rs:13192-13193).
4. 연속 페이지는 각주 없는 신선 full-page(base 895.8)인데 시작 페이지의 75.8 을
   물려받아 `avail_for_rows≈44.6px` → 페이지당 ~1행 → 122행이 188쪽으로 폭주.

**검증**: pi=306 각주 렌더 0개(render-tree 전 페이지 Footnote 노드 없음) — 즉 예약된
820px 는 연속 페이지에서 **실사용되지 않는 유령 예약**. pi=99 는 시작 페이지 각주 0 →
table_available=895.8 → 정상.

## 3. 수정 설계 (Stage 3 대상)

**핵심**: 연속 페이지의 `page_avail` 은 시작 페이지의 `table_available`(=시작 페이지
각주 예약 반영) 이 아니라 **신선 페이지 기준 `base_available`** 이어야 한다.

- **1차(저위험, #1937 계열 정조준)**: 연속 페이지에서 표 자체 각주가 없을 때
  (`table_footnote_count == 0`) `page_avail = base_available`(반복 머리행 overhead 는
  기존대로 차감). 각주 보유 표는 기존 동작 유지 → 회귀 위험 0.
    ```
    let page_avail = if is_continuation {
        if table_footnote_count == 0 { base_available } else { table_available }
    } else { ... };
    ```
- **일반형(후속 검토)**: 연속 페이지 각주 예약을 그 페이지에 실제 렌더되는 각주로
  한정(현재는 시작 페이지 current_footnote_height 를 전 연속 페이지에 전가). 표 셀
  각주가 연속 페이지에 걸치는 케이스까지 정합. 다만 각주 분포 로직 변경은 광역 →
  본 이슈에선 1차 저위험안으로 한정 제안.

## 4. 검증 계획 (Stage 3)

- pi=306: 188쪽 → ~7쪽(한글 ~9쪽 수렴), 전체 231 → ~55쪽 확인.
- 회귀 게이트: `cargo test --lib`(로컬) + CI 풀스위트. 표 분할 관련
  `pagination`·`typeset` 테스트, `#1658 byeolpyo1/4` 양 게이트, 3·4차 서베이 표본
  (PAGE_DELTA 과소/과대 양방향) — 특히 각주 보유 분할 표 무회귀.
- +88/+84(공급망/GCC) 문서도 동일 각주-예약-전가 여부 확인(있으면 함께 해소).

## 5. 리스크

- 각주 보유 분할 표에서 연속 페이지 각주가 실제 걸치면 base_available 로 과대 →
  겹침 위험. → **`table_footnote_count == 0` 가드로 회피**(각주 없는 데이터 표만 대상).
- typeset_block_table 는 광역 경로 → `cargo test --lib` + CI 필수.

→ Stage 3(구현 + 게이트) 진행 승인 요청. 1차 저위험안(각주 없는 표 연속 페이지
base_available)으로 구현하겠습니다.
