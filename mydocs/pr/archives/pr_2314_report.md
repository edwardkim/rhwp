# PR #2314 최종 보고 — 편집 vpos 재계산의 저장 리셋·문단 여백 보존 (#2299)

- PR: https://github.com/edwardkim/rhwp/pull/2314 (lpaiu-cs)
- 결정: **merge** (467ca361, 2026-07-17, BEHIND — merged tree 로컬 선검증 후 admin merge)
- 검토 기록: `pr_2314_review.md` (원 검토 + CHANGES_REQUESTED + 재검토 v2)

## 경과

1. 원 검토: 방향(저장 밴드 경계 보존) 타당 + 한컴 실기 대조(작업지시자)로
   단/쪽 유지 정답 확정. 단 treatise 7→8 을 PR 기인 회귀로 판정해
   CHANGES_REQUESTED (+ 80건 스윕 방법론 확인 요청).
2. 컨트리뷰터 자체 max-리뷰로 회귀 3계열(성장 동결/빈 문단 겹침/placeholder
   고착) 발견·보류 요청 후 보강 커밋 a1a23850 push. treatise 는 좌표 실측
   반론(정당 성장, 종전 7쪽이 위양성).
3. 재검토 v2: 반론 확증 — devel 은 간격 800HU 소실 압축으로 7쪽 유지,
   PR 은 간격 정확 보존으로 8쪽 정당 성장. **원 리뷰 판정이 devel 간격
   소실 버그를 정답 기준으로 삼은 오판**이었음.

## 검증 (merged tree)

- issue_2299 신규 회귀 테스트 8/8, `cargo test --tests` release-test 전 통과,
  fmt/clippy 0, CI 전 항목 pass
- 편집-스윕 574건(<2MB) devel 대조: 변동 70→11 (PR 기인 = treatise 1건 정당,
  공통 10건 기존 동작, **devel 가짜 변동 60건 해소**)
- devel 회귀 3건 유지: shortcut 7→7(단 밴드), biz_plan 6→6, SO-SUEOP 46→46

## 남은 축

- #2320: 로드 시 문단 중간 비-0 vpos 되감김을 쪽 경계로 미인식 (본 PR 과
  같은 저장 vpos 의미론 가족, 렌더링 경로 — 별도 처리)

## 교훈

회귀 판정의 기준선 자체가 버그일 수 있다 — "기존 동작 유지"를 정답으로
삼기 전에 기존 동작의 좌표 실측이 필요하다 (treatise 오판의 재발 방지).
