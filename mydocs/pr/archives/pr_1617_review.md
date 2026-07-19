# PR #1617 처리 보고서 — footer 독립-쪽 규칙 역공학(규칙 부재 확정) + #1612 메트릭 정정 통합

- PR: https://github.com/edwardkim/rhwp/pull/1617
- 제목: `Task #1616 (docs): footer 독립-쪽 배치 규칙 역공학 — 규칙 부재 확정`
- 작성자: planet6897 (collaborator)
- 연결: Closes #1616, #1615(=Task #1612) 위 스택
- base ← head: `devel` ← `planet6897:pr/devel-1616-squash`
- 처리일: 2026-06-28

## 1. 처리 결정

**cherry-pick 통합 후 PR close.** -1쪽 갭 시리즈의 마무리 — razor-thin footer 8건+다페이지
4건의 독립-쪽 배치 규칙을 역공학한 결과 **"규칙 부재" 확정**(한글 line-level 페이지-채움
재현 외 경로 없음, 통제셋 실질 상한 78.3%)을 문서화한다.

**중요 — "문서 전용"이 아니다:** PR 본문은 docs로 표기하나, squash 스택의 신규 커밋
(`e3d3b750`)에는 **#1612(=#1615, CLOSED·미머지) 소스 `src/document_core/queries/rendering.rs`
+77줄이 함께 포함**된다. #1615가 별도 머지 없이 close되어, 이 진단-메트릭 정정이 본 PR로 처음
devel 에 들어온다. → 빌드/테스트 검증을 수행했다(아래 4).

## 2. 충돌

신규 커밋은 단일(`e3d3b750`). 충돌은 `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` add/add 1건.
devel 고유 2줄("대형 tac 표 과소측정 36398709 등")은 PR판에서 **상세 보고서 참조로 교체 +
별도 섹션에서 '메트릭 아티팩트로 정정'**하는 더 정확한 서술로 대체됨 → PR판 채택이 의미상
상위·정정판. 소스(rendering/typeset/parser/test)는 충돌 없이 적용.

## 3. 통합 내용 (devel 위 cherry-pick 1커밋, 작성자 보존)

| 파일 | 내용 |
|---|---|
| `src/document_core/queries/rendering.rs` (+77) | **[#1612]** `compute_hwp_used_height` 에 `base_top`(페이지 시작 vpos) 차감 추가. vpos 누적값을 per-page used 와 비교하던 메트릭 아티팩트(다페이지 diff ~수백px 누적 증가→"표 과소측정" 오판) 정정. **dump-pages 출력 진단 함수에서만 호출 → 페이지수 불변, 저위험.** 단위 테스트 1건 동반 |
| `src/renderer/typeset.rs`, `src/parser/hwpx/*` | #1611·#1608 스택 잔재 — 이미 devel 반영분과 동일, no-op |
| `mydocs/tech/investigations/issue-1600/render_minus1_page_gap.md` | 역공학 결과(규칙 부재) + 잔여 12건 전수 최종 판정 + 알려진 한계(78.3%) |
| 다수 `mydocs/` 문서 | #1600/1608/1611/1612/1616 계획·단계·최종 보고서 (스택 누적) |

## 4. 검증 (로컬)

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze) | 전부 pass (21m43s) |
| 충돌 시뮬레이션 | 문서 1건(상위·정정판 해소) |
| 신규 `task1612_hwp_used_height_is_per_page_not_cumulative` | 통과 |
| `issue_1611`/`issue_1608`/`hwpx_roundtrip_baseline`/`visual_roundtrip_baseline` | 4/1/3 passed |
| 전체 `cargo test --tests` | **FAILED 0건** (160 결과 / 누적 2637 passed) |
| fmt --check | clean |
| clippy (rendering.rs/typeset.rs) | PR 변경 무경고 |

## 5. 시각 판정 주의

통제셋 60→72(+12, 78.3%), 잔여 12건 = 한글 내부 line-level 알고리즘 재현 필요(대규모/고위험)
판정은 컨트리뷰터 한글 오라클(로컬 root) 측정·역공학이다. PR 의 **결정적 논거(양방향성:
-1쪽 12건 외 +초과 8건 공존 → 단일 systematic 보정 net 개선 불가)**는 메트릭 분석으로
타당하나, 최종 페이지 정합 권위는 작업지시자 환경(`feedback_self_verification_not_hancom`).
로컬은 baseline 회귀 없음 + 진단 함수 무영향만 확인.

## 6. 의의 — -1쪽 갭 시리즈 종결

#1608(요인 A, +6)·#1611(요인 B, +6)·#1612(메트릭 정정)·#1616(규칙 부재·한계 확정)으로
-1쪽 갭 시리즈를 **알려진 한계로 종결**. 추가 투자는 한글 레이아웃 엔진 재현 영역(별도 대규모
과제)으로 분리.

## 7. 산출물

- 본 처리 보고서: `mydocs/pr/pr_1617_review.md`
