# PR #1009 처리 보고서 — Task #1007: HWP5 변환본 페이지 강제 나눔 한컴 정합

- 처리일: 2026-05-19
- 컨트리뷰터: [@jangster77](https://github.com/jangster77) (Taesup Jang)
- 결정: **옵션 B (수정 요청 / 보류)** — 작업지시자 승인
- 머지: **하지 않음** (cherry-pick 롤백)
- 연결 이슈: #1007 (OPEN 유지 — close 안 함)

## 1. 결정 사유 — PR base 부정합으로 인한 페이지 수 회귀

PR #1009 본문은 "sample16-hwp5 **67 → 64** 한컴 정합"을 주장하나, 이 "67" 시작점은 **PR 이 분기된 #1005 머지 이전 base** 기준이다. **현재 devel 은 #999(67→64) + #1005 머지로 이미 sample16-hwp5 = 64 페이지** (한컴 정답지 정합 달성됨). PR #1009 의 cross-paragraph vpos reset 페이지 split 로직을 이미 64 인 현 devel 위에 적층하면 **64 → 65 로 over-split (1 페이지 회귀)**.

PR 이 가정한 base 상태(67)와 실제 머지 대상 devel 상태(64)의 불일치 — 누적 적층 부정합.

## 2. 검증 결과 (cherry-pick `034ee72e`, 롤백됨)

| 항목 | 결과 |
|------|------|
| cherry-pick 충돌 | model/document.rs + parser/mod.rs (#1005 variant 인프라 중복) → HEAD 채택 (코드 동일, 주석만 차이) |
| `cargo test --release --lib` | 1307 passed / 0 failed ✅ |
| `cargo clippy -D` / `cargo fmt --check` | 통과 / exit 0 ✅ |
| sweep variant=false (sample16-hwp3, sample10/14, exam_kor, aift, biz_plan) | 전부 **diff=0** ✅ 무회귀 |
| **sweep 타깃 sample16-hwp5** | **BEFORE(devel #1005) 64 → AFTER(#1009) 65** ⚠️ **+1 페이지 회귀** |

자기 검증(test/clippy/fmt) 및 variant=false 일반 문서 무회귀는 양호하나, **타깃 sample16-hwp5 가 현 devel 기준 64→65 회귀**.

## 3. 추가 발견 — PR 본문 기술과 실제 코드 불일치

PR 본문 표는 `composer.rs CHARS_PER_LINE 45 → 50 (variant CharShape spacing -12% 보정)` 을 명시하나, **본질 커밋 `71054c51` 및 PR head 브랜치 어디에도 composer.rs 변경 없음** (45 유지, #999 값 그대로). PR 본문 문서와 실제 변경 셋 불일치 — 재작업 시 정정 필요.

## 4. 수정 요청 내용 (컨트리뷰터 전달)

1. **base 재정렬**: 최신 devel(#999 + #1005 머지 = sample16-hwp5 이미 64) 기준으로 rebase. "67 → 64" 가정 폐기.
2. **재진단**: #999(spacing_before=0 + CHARS_PER_LINE 45) + #1005(변환본 4중 AND 가드 + ParaShape /4) 적용 후 sample16-hwp5 가 이미 64 인 상태에서, #1009 의 vpos reset 페이지 split 로직이 추가로 필요한지 / over-split 을 일으키는지 재평가. 필요 시 #999/#1005 와 중복되지 않는 순수 증분만.
3. **PR 본문 정정**: composer.rs 45→50 기술이 실제 변경과 불일치 — 제거 또는 실제 반영.
4. **검증 기준 명확화**: BEFORE 를 최신 devel 로 고정한 sweep (현 devel 64 → 목표 64 유지, over-split 없음) + 작업지시자 시각 판정.

## 5. 처리 절차

- cherry-pick 브랜치 `pr1009-cherry` 롤백 (local/devel = origin/devel = `f35a0a59`, #1009 미반영)
- PR #1009 **OPEN 유지** (close 안 함 — 수정 후 재제출 대기)
- 이슈 #1007 **OPEN 유지** (close 안 함 — 미해결)
- 검토/보고서 `mydocs/pr/archives/pr_1009_{review,report}.md` 보관 (재검토 시 참조)
- 산출물 `output/poc/pr1009/{before,after}/` (회귀 입증 자료, git 미추적)

## 6. 메모리 룰 정합

- `feedback_contributor_cycle_check` — @jangster77 #997~#1011 연속. #1009 는 base 부정합으로 보류 (시리즈 누적 적층 시 base 재정렬 필수 교훈)
- `feedback_pr_supersede_chain` — #999/#1005 가 이미 타깃(67→64) 달성 → #1009 가 동일 목표를 부정합 base 기준으로 재시도 → over-split. 누적 PR 시리즈는 직전 머지 반영 후 재평가 필요
- `feedback_visual_judgment_authority` / `feedback_self_verification_not_hancom` — 페이지 수 64→65 는 시각 판정 이전 명백 회귀. sweep 정량으로 base 부정합 조기 발견 (시각 판정 단계 진입 전 차단)
- `feedback_hancom_compat_specific_over_general` — vpos reset 로직 자체는 variant 가드 한정으로 일반 문서 무회귀 (설계는 건전). 문제는 base 상태 가정 불일치
- `project_output_folder_structure` — sweep 산출물 output/poc/pr1009 배치

## 7. 결론

PR #1009 의 vpos reset 페이지 split 설계 자체는 variant 가드 한정으로 일반 문서 무회귀(sweep 입증)이나, **PR base 가정(sample16-hwp5=67)이 현 devel(#999/#1005 머지 후 = 64)과 불일치하여 타깃이 64→65 over-split 회귀**. **옵션 B — 컨트리뷰터에게 최신 devel 기준 rebase + 재진단 요청, PR/이슈 OPEN 유지**.
