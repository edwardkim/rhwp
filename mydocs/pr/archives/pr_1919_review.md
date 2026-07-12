# PR #1919 검토 — Task #1898: tac 인라인 그림 문단 렌더 줄 전진 +11.7px 수정 (vpos 기준점 초기화 예외)

- 작성일: 2026-07-05 / 검토자: Claude (메인테이너 대행 검토)
- PR: planet6897 → devel / MERGEABLE, 충돌 없음
- 연결 이슈: #1898 (기전 1) — 본문 `Refs #1898`
- 시간순 처리 3번째 (#1912 → #1913 → **#1919** → #1922)
- **⚠ #1912와 동일 결함의 상류 수정** — 관계 분석은 `pr_1912_review.md` §3 참조

## 1. PR 요약

#1912와 같은 증상(36388711 p9, 렌더 44.8px vs layout 33.1px vs 한컴 32.9px)을 **원인 지점에서**
수정: tac 그림의 `PageItem::Shape`(dy=0)가 vpos 기준점(page/lazy base)을 무조건 초기화
(#409/#1027 후처리)하는 것이 원인이므로, **실제 텍스트 줄에 통합된 tac 개체**(호스트
`para_has_visible_text` ∧ Picture/Shape/Equation `treat_as_char`)의 Shape 항목은 초기화에서
면제한다. 초기화가 없으면 다음 문단이 재역산 경로에 진입하지 않아 trailing-ls bridge
이중 가산이 발생하지 않는다.

- **판별 정제 과정 기록**: 1차 blanket 면제안이 issue_1116 한컴 핀 2건을 −8.5px로 깨뜨림 →
  텍스트 없는 tac-전용 문단(sample16 pi=71, LINE_SEG lh=개체 높이)은 종전 초기화 유지로 정제.
- 동반 계측 개선: `RHWP_DEBUG_PARA_TAC` 하드코딩 pi(651/652) → 콤마 목록 env 일반화
  (release 동작 무영향).

변경: `src/renderer/layout.rs` +33 / `layout/paragraph_layout.rs` ±9(계측만) /
신규 핀 `tests/issue_1898.rs` + 계획·보고 문서 3건.

## 2. 코드 검토

- 개입 지점이 #1912(하류 보정)보다 근본적 — 잘못된 무효화 자체를 막아 이후 경로 전체가
  종전 로직으로 자연 동작. 커버 개체도 Picture/Shape/Equation 3종(수식은 항상 tac인
  프로젝트 특성상 Equation 포함이 중요).
- 판별이 기존 헬퍼(`para_has_visible_text`) 재사용으로 국소적. tac-전용 문단 보존
  (issue_1116 13/13)은 반례 기반으로 정제된 것이라 신뢰 가능.
- **⚠ 실측 결과 부분 수정 (2/3)**: p9 결함 전이 3곳 중 **87→88이 44.8px로 잔존**
  (devel+#1919 단독, SVG·render-tree 양쪽 확인). 원인: 87→88의 base 리셋은 tac 그림이
  아니라 **직전 표(성과지표)**가 일으키므로 본 PR의 "tac Shape 초기화 면제"가 닿지 않는다.
  본문의 "44.8 → 33.1" 주장은 95~97에만 성립하며, **자기 핀(`issue_1898.rs`)도 95~97만
  커버해 잔존을 검출하지 못한다**. #1912(재역산 지점 가드, 리셋 원인 무관)가 이 축을
  보완 — 상세 실측표는 `pr_1912_review.md` §3.

## 3. 게이트 결과 (devel `bf5228df` + PR 테스트 머지)

| 게이트 | 결과 |
|---|---|
| GitHub CI | 전 체크 pass |
| cargo fmt --check | 통과 |
| cargo clippy --profile release-test --all-targets | 경고 0 |
| cargo test --profile release-test --tests | **2,870 통과 / 실패 0** |
| OVR baseline 5샘플 (issue1835 TAC 샘플 포함) | **5/5 개체 회귀 0건** |

## 4. 시각 판정 자료

- `output/poc/pr1912/pr1912_p9_3way.png` (BEFORE/AFTER/ORACLE) 및 OVL — #1912 검토와 공용.
- #1919 단독은 **완전 동일 결과가 아님**: `output/poc/pr1919/after_p9.png` 실측에서
  87→88 전이 44.8px 잔존 (§2 참조). 결합(devel+#1912+#1919) 상태는 `output/poc/prcombo/`
  에서 3전이 전부 33.1px 확인.

## 5. 판단 (작업지시자 승인 대기)

- 검토자 권고 (정정): **#1912와 상호 보완이므로 둘 다 머지** — 본 PR 단독은 부분 수정
  (2/3)이나, Shape/Equation 자기-리셋 예방 + 계측 개선의 고유 가치가 있고 게이트 전부
  통과. 상세는 `pr_1912_review.md` §3. 머지 코멘트에 87→88 실측(자기 핀 미커버)을 공유.
- 이슈 #1898: 기전 1 해소 후에도 기전 2(누적 드리프트, #1759 계열) 측정 기록이 남음 —
  close/재정의는 작업지시자 판단(컨트리뷰터도 본문에서 판단 요청).
