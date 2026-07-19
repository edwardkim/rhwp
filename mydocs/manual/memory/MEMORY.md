---
kind: memory
status: historical
canonical: mydocs/manual/codex/docs_and_git_workflow.md
last_verified: 2026-07-17
---

# Claude memory dump 색인

이 디렉터리는 과거 사용자 피드백과 프로젝트 메모리의 출처를 보존한다. 항목에 적힌 개인 경로, 브랜치,
도구 상태와 작업 순서는 현재 절차가 아니며, 현행 정책과 충돌하면
[`docs_and_git_workflow.md`](../codex/docs_and_git_workflow.md),
[`pr_review_workflow.md`](../pr_review_workflow.md) 및 작업별 canonical 문서를 따른다.

## user
- [작업지시자 정체성 + 역할](user_role_identity.md) — edwardkim, rhwp 메인테이너, Windows + WSL2 + macOS(ios/devel)
- [기술 스택 + 협업 분배](user_tech_stack.md) — 본인 보유(Rust/WASM) vs Claude 위임(문서/cherry-pick) vs 직접 결정(시각 판정)
- [작업 스타일](user_work_style.md) — 하이퍼-워터폴, 시각 판정 게이트, 광범위 sweep 정량화, 외부 PR 옵션 분류

## feedback — 워크플로우/프로세스
- [작업 시간 제한 금지](feedback_no_time_limits.md) — 클로드가 임의로 작업 종료 제안 금지
- [타스크 프로세스 반드시 준수](feedback_process_must_follow.md) — 이슈→브랜치→할일→계획서→구현 순서 절대 생략 금지
- [Hyper-Waterfall 워크플로우 필수 준수](hyper_waterfall_workflow.md) — 수행계획서→구현계획서→단계별 보고, 승인 없이 코딩 금지
- [이슈 클로즈는 작업지시자 승인 필수](feedback_no_close_without_approval.md) — 미해결 상태 임의 클로즈 금지
- [이슈 close 시 devel 머지 검증 필수](feedback_close_issue_verify_merged.md) — close 전 git branch --contains 검증
- [이슈 착수 시 즉시 assignee 지정 필수](feedback_assign_issue_before_work.md) — 진짜 일차 방어선
- [라운드형 이슈는 부모의 서브 이슈로 등록](feedback_subissue_linking.md) — 생성 직후 sub_issue 연결(numeric id), 실행 umbrella 귀속
- [이슈 작업 전 열린 PR 확인 필수](feedback_check_open_prs_first.md) — gh pr list로 외부 기여 확인, 이차 방어선
- [타스크 번호는 GitHub Issues로 채번](feedback_task_numbering.md) — gh issue create로 자동 채번
- [마일스톤 표기 규칙](feedback_milestone_notation.md) — v1.0.0→M100, v0.5.x→M05x
- [보고서는 타스크 브랜치에서 커밋](feedback_commit_reports_in_branch.md) — merge 전 git status 필수
- [오늘할일 문서 갱신 필수](feedback_update_daily_orders.md) — 세션 종료 전 커밋

## feedback — 문서/명명 규칙
- [작업 문서 네이밍 규칙](feedback_working_doc_naming.md) — task_m100_{번호}_stage{단계}.md 패턴 필수
- [최종 보고서 위치 규칙](feedback_report_location.md) — 최종은 report/, 단계별은 working/
- [한국어 단어 선택 + 자기검열](feedback_kr_word_choices.md) — "산수" 금지, 비교/최상급/공공기관 오인 회피
- [단어 선택 — "산수" 대신 "계산"](feedback_word_choice_calculation.md) — 기술 문서에서 "산수" 금지
- [기계적 어휘 회피](feedback_machine_vocabulary.md) — "본질/정합 영역" 반복 금지, 자연스러운 한국어 산문
- ["영역" placeholder filler 금지](feedback_no_yeongyeok_filler.md) — 빈자리 메우기에 "영역" 금지, 실제 공간/범위 의미일 때만
- [외부 공개 문서 자기검열 체크리스트](feedback_external_docs_self_censor.md) — 7개 카테고리 점검

## feedback — PR/컨트리뷰터 관련
- [첫 PR 컨트리뷰터 환영](feedback_first_pr_courtesy.md) — "rhwp 첫 PR" 표현, fork base 동기화 권장
- [PR 댓글 톤 — 과도한 표현 자제](feedback_pr_comment_tone.md) — 차분하고 사실 중심
- [PR 머지/close 후 메인테이너 코멘트 필수](feedback_pr_maintainer_comment_required.md) — 처리 결과+검증 요약+감사, 이슈 코멘트와 별개
- [작은 단위 PATCH 회전 운영](feedback_small_batch_release_strategy.md) — 빠른 회전, 위험 분산
- [PR 생성은 별도 승인 후 진행](feedback_pr_requires_explicit_approval.md) — PR 준비와 GitHub PR 생성 분리
- [PR 본문 한국어 작성 필수](feedback_pr_body_korean_required.md) — 내부 타스크 PR 제목과 본문은 한국어
- [PR 전 로컬 CI급 검증 필수](feedback_pr_ci_before_pr.md) — PR 생성/최종 푸시 전 CI급 로컬 테스트
- [push 전 cargo test --tests + fmt --check 필수](feedback_push_full_test_required.md) — --lib만으론 통합 테스트 회귀 못 잡음 (PR #1020)
- [push 전 테스트는 release-test 프로필 사용](feedback_release_test_profile.md) — Linux WSL2 실측 262s→108s (~2.4배)
- [PR 브랜치 npm 검증 위생](feedback_npm_verify_hygiene.md) — npm ci 선행(stale 교정), 브랜치 이탈 후 untracked node_modules 정리, 신설 패키지 dir 은 개별 .gitignore
- [컨트리뷰터 사이클 사전 점검 의무](feedback_contributor_cycle_check.md) — gh pr list --author로 누적 PR 확인, "첫 사이클" 임의 추정 금지
- [PR supersede 체인 네 패턴](feedback_pr_supersede_chain.md) — close+통합 / 머지+supersede / 머지+회귀정정 / 상호보완 둘다머지(실측 필수). 동일 컨트리뷰터 PR 점검 필수
- [GitHub 미연결 author는 .mailmap으로 정정](feedback_mailmap_for_unlinked_authors.md) — history rewrite 금지, 비파괴 매핑

## feedback — 시각 판정/한컴 호환
- [시각 판정 권위](feedback_visual_judgment_authority.md) — 한컴 2022 정답지, Claude는 정량 측정만 보조
- [rhwp 자체 시각 해석 권위](feedback_rhwp_visual_authority.md) — IR 충실보다 시각 본질 우선 가능
- [PDF 정답지 등급](feedback_pdf_not_authoritative.md) — 한컴 2020/2022 편집기 PDF만 정답지, 뷰어/외부/2010은 미달
- [한컴 호환은 케이스별 명시 가드](feedback_hancom_compat_specific_over_general.md) — 일반화보다 구조 가드가 안전
- [자기 검증 ≠ 한컴 호환](feedback_self_verification_not_hancom.md) — 한컴2020 수동 검증 게이트 필수
- [렌더링 의미는 추정 금지 — 권위 자료로 확정](feedback_no_inference_authoritative_spec.md) — 한컴 스펙+대비 샘플+편집기 UI 교차검증 (#1156)
- [v0.7.6 회귀의 origin](feedback_v076_regression_origin.md) — 컨트리뷰터 PDF 정답지 사용 → 회귀. 시각 검증 게이트
- [시각 회귀 비중 증가](feedback_visual_regression_grows.md) — 페이지 수 비교만으로 검출 불가, 시각 판정이 핵심
- [시각 검증 선택 적용 (OVL-step)](feedback_visual_verification_selective.md) — 기계적 전수 금지, PR 목적 기준 선택, manual/verification/visual_verification_governance.md 준수

## feedback — 코드/렌더링 관련
- [렌더링 보정 하드코딩 금지](feedback_no_hardcoded_render_tuning.md) — 샘플명·페이지·임의계수 분기 금지, 근거는 문서 속성/스펙 필드만 (백업에서 복원)
- [renderer별 별도 image 함수 sweep](feedback_image_renderer_paths_separate.md) — svg/canvas/paint/json 4 backend 점검
- [결함 진단 시 layer 귀속 정확화](feedback_diagnosis_layer_attribution.md) — emission 위치 ≠ 결함 본질 위치, 시프트 출처 추적
- [정정 시 두 경로 점검 패턴](feedback_fix_scope_check_two_paths.md) — layout 정정만으로 부족, reflow/preprocessing도 동일 정정
- [폰트 추가 시 alias 동기화 필수](feedback_font_alias_sync.md) — style_resolver + font_metrics_data 2계층 등록
- [트러블슈팅 폴더 사전 검색 의무](feedback_search_troubleshootings_first.md) — 직렬화/한컴 호환 작업 전 전수 검색

## feedback — 릴리즈/배포/CI
- [릴리즈 전 main 동기화 점검 필수](feedback_release_sync_check.md) — git pull --ff-only origin main
- [릴리즈 작업 시 매뉴얼 정독 필수](feedback_release_manual_required.md) — 부분 검색 금지, 체크리스트 1:1 대조
- [AMO 제출 4대 함정](feedback_amo_submission_gotchas.md) — Firefox 확장 제출 전 체크리스트
- [CI 진행중 수치 보고 금지](feedback_no_metrics_from_inprogress_ci.md) — in_progress run의 step 시간으로 성능 보고 금지 (#1192)

## project
- [거버넌스 부트로더화](project_governance_bootloader.md) — 2026-07-17 이후 CLAUDE.md=38줄 부트로더, 절차는 canonical 문서 단일 기록
- [0.8.0 분기점 — HWPX 저장 마일스톤](project_v080_hwpx_save_milestone.md) — 다음 릴리즈 0.7→0.8 MINOR, studio pkg 재빌드 필수
- [#1582 리팩토링 umbrella](project_1582_refactor_umbrella.md) — 0.8/v1.0 구조 정리 기준 이슈, 1단계 SourceProvenance+LayoutCompatibilityProfile, freeze 선행
- [리팩토링 거버넌스 2원칙](project_refactor_governance.md) — SOLID + 복잡도, 계획·평가·PR 검토의 기준 축
- [브랜치 정책 + iOS 분기](project_branch_policy.md) — main/devel/local-devel + ios/devel(맥북 전용)
- [외부 컨트리뷰터 명단](project_external_contributors.md) — 누적 32명(2026-07-04), 첫 PR/재기여 식별, merged 집계만으론 판단 금지
- [collaborator 2명 — jangster77·postmelee](project_pr_merge_collaborator.md) — 외부 PR 머지 분업, Taegyu Lee=postmelee 동일인, assignee는 3계정만 가능
- [LFS 쿼터 초과 — pdf-large M 오진 금지](project_lfs_quota_full.md) — 10GB 초과로 LFS 객체 미수신, 건드리지 말 것
- [확장 vite publicDir:false + CSP 인라인 금지](project_extension_publicdir_false.md) — public/ 자산 build.mjs 개별 copy 필수, 인라인 script 차단
- [알한글 iOS 프로젝트](project_alhangeul_ios.md) — iPad HWP 학습 도구, 맥북 전용
- [안드로이드 IME 미구현](project_android_ime_pending.md) — 기기 미보유
- [rhwp 정체성 — DTP 엔진 + 워드프로세서](project_dtp_identity.md) — 아래아한글 = QuarkXPress 대체 의도
- [수식 컨트롤은 항상 TAC](project_equation_always_tac.md) — paragraph_layout 인라인 배치 핵심 경로
- [한컴 LINE_SEG 자동 재계산](project_hancom_lineseg_behavior.md) — LINE_SEG 비어있어도 한컴이 재계산
- [HWPX→HWP 어댑터의 한계](project_hwpx_to_hwp_adapter_limit.md) — 다음 시도는 "완전 변환기" 필요
- [HWPX serializer fidelity 한계](project_hwpx_serializer_limits.md) — run 평탄화·셀/글상자 컨트롤 소실·합성 lineseg, 후속 이슈 대상 (#1315)
- [output 폴더 서브폴더 구조](project_output_folder_structure.md) — re/svg/debug 용도별 분리
- [HWPX switch/case와 줄간격 유형](hwpx_switch_case.md) — HwpUnitChar case=글자에따라, default=고정값

## reference
- [작업지시자 정답지 한컴 환경](reference_authoritative_hancom.md) — Windows 한컴 편집기 1차 + 맥/리눅스는 한글 2020/2022 PDF
- [로컬 폰트 경로](reference_font_path.md) — TTF 폰트 프로젝트 외부 분리 (Linux/macOS 경로별)
- [hwp2hwpx Java 라이브러리](reference_hwp2hwpx_library.md) — HWP↔HWPX 변환 매핑 권위 자료
- [Discord 커뮤니티](reference_discord.md) — Rust Discord 소개 (2026-04-04)
