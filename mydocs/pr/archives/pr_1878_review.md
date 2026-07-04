# PR #1878 처리 보고서 — 분할 co-anchored 빈-host float 예산 이중차감 교정 (#1860)

- PR: https://github.com/edwardkim/rhwp/pull/1878
- 제목: `Task #1860: 분할 co-anchored 빈-host float 예산 이중차감 교정 + 노드-자식 포섭 불변`
- 작성자: planet6897 (82건 머지, 페이지네이션 시리즈)
- 연결: Closes #1860 (PR #1855 리뷰에서 표면화)
- base ← head: `devel` ← `pr/devel-1860` (base = 현 devel `71e6d8ec`, 착시 없음)
- 처리일: 2026-07-04

## 1. 처리 결정 — 시각 판정 통과 → admin merge

**admin merge (작업지시자 시각 판정 통과, 2026-07-04).** 분할(RowBreak) 표의 valign=Center
라벨 세로 위치 어긋남(−23~+42pt)의 근본 원인을 예산 이중차감으로 규명·교정. CLEAN + CI 전부
pass + 로컬 전체 테스트/게이트 green + p44/45 3자 대조(before/after/한글 2024 정답지)에서
PR 주장과 정확히 일치 확인 후 판정 통과.

## 2. 변경 범위 (6 files +488/-0, 순수 추가)

| 파일 | 내용 |
|---|---|
| `src/renderer/typeset.rs` (+50) | `DeferredTableControl.para_start_height`(참 para_start 보존) + `typeset_block_table`에 **예산 전용** `budget_para_start_height` 분리. `is_empty_host_column_float`(비-continuation + 비-TAC + v_off>0 + 빈 host) 4조건 가드만 예산 기준을 current_height→참 para_start |
| `src/renderer/layout/table_partial.rs` (+23) | 노드-자식 포섭 불변 — 렌더 후 표 노드 bbox를 자손 최하단까지 **확장만**(축소 없음), 분할 조각 셀 내 as-char 텍스트박스 clip 해소 |
| `mydocs/` 4건 | 계획·구현·계측·최종 보고서 |

## 3. 근본 원인 평가 — 정밀

- valign 로직이 아니라 **fragment 분할 예산**이 문제: 지연 co-anchored 경로가 참 para_start(0)를
  잃고 current_height(선행 tac 캡션 65.5px 반영)를 쓰면서, v_off와 캡션을 **이중차감** →
  page_avail 795.2(참값 860.8) → RowBreak 컷이 소스 hard_break(37)보다 3줄 조기(34).
- 단일 결함이 반대 방향 두 증상(p44 라벨 −24.6pt / p45 +40.8pt)을 모두 설명 — 진단 일관성 높음.
- **렌더 위치는 불변 유지**(예산만 분리) — 위치까지 바꾸면 대형 지연 표(admrul_1065)가 무한
  재배치 루프(계측 확인). 회귀 위험을 계측으로 통제한 설계.

## 4. 검증 (로컬, Linux WSL2)

| 항목 | 결과 |
|---|---|
| GitHub CI | 전부 pass (Build&Test/CodeQL/Analyze/Canvas) |
| 충돌 / base | 0건 / base=현 devel |
| `issue_rowbreak_chart_overlap` | **20/20** (pi=28 회귀 해소 포함, PR 주장 일치) |
| `svg_snapshot` | 8/8 (golden 무변동) |
| 전체 `cargo test --tests` | **FAILED 0** (185 결과 / 2809 passed) |
| fmt / clippy(변경 2파일) | clean / 무경고 |
| 컨트리뷰터 자체 빅코퍼스 | big_hwpx/big_hwp 각 2,500 render-diff 지표 불변(PAGE 0), page ±1 3건 전부 PASS |

## 5. 시각 대조 (로컬, 권위: pdf/issue1853_caption_precedes_body_split-2024.pdf 한글 2024)

p44 fragment 끝 비교 — PR 주장과 정확히 일치:

| | p44 fragment 끝 | 라벨 Δ |
|---|---|---|
| before(devel) | "17. 그 밖에…" 중간 컷 (3줄 짧음) | −24.6pt |
| after(PR) | "③ 정부는… 업무를 수" (2줄 회복, end_cut 34→36) | **−7.8pt** |
| 정답지(한글 2024) | "…지원할 수 있" (hard_break 37) | 기준 |

잔여 1줄(−7.8/+7.2pt)은 내용셀 per-line 측정 미세 과대(#1759/#1760/#1763 계열) — 본 PR 범위
외로 명시, 타당. 자료: `output/poc/task1860/{before,after}_p044/045.png`, `pdf_p-44/45.png`.

## 6. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1878_review.md`
