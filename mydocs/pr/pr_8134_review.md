# PR #8134 — 저장 줄 없는 글상자의 세로 정렬

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel에서 원 증상과 통합 후보의 실제 Native/fresh WASM 출력을 확인한 뒤 판정을 갱신한다. 현재 상태는 기여 코드의 거부 판정이 아니다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8134; davindev, CONTRIBUTOR, Open, non-draft, base devel.
- 원 head: `77d5673d533ee3cc43666ef824200b74207ad8a3` (`kidsnote/rhwp:fix/no-ls-textbox-valign`).
- 기준 devel: `2d535febe00a61585b6770cb95860fee912bbe54`; 검토 브랜치 `review/davindev-20261030`.
- 관련 이슈: #8122. 메인테이너 `edwardkim`의 reviewer 지정을 API로 확인했다(2026-10-30).
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch. #8175는 rework_and_exceptions도 적용한다.
- 사용자 승인: 세 PR 검토. #8175의 SVG·Canvas2D 충돌은 최신 기울임꼴·글꼴 임베딩을 보존하고 clipping·대체 목록을 통합하는 메인테이너 보정을 별도로 승인했다. 원격 게시·push·merge는 아직 수행하지 않았다.

## 선행 이력과 검증 계획

#8134가 #8147의 조상이다. 이전 메인테이너 지적은 앞 간격을 측정에서 별도로 합산해 실제 배치와 어긋난 문제였고, 기여자가 a8a86c106a에서 실제 배치의 줄 끝을 소비하도록 재작업했다. 최신 head를 대상으로 원 사례와 앞 간격·빈 문단·위/가운데/아래 정렬 경계를 확인한다.

- 같은 입력/PDF·compat 2022·96dpi·실제 글꼴 환경으로 최신 base와 후보의 전쪽 TSV를 산출한다. 90% 최소값·2%p 초과 하락·쪽수·내용 소유를 확인하고 대표 PNG를 직접 판독한다.
- 원 PR의 기존 GitHub CI는 과거 원 head의 참고 증거다. 메인테이너 보정 통합 후보는 새 코드의 focused·필수 lint·회귀·최적화 WASM 검증을 수행한다.
- 임시 로그·원격 metadata: `output/pr-review/davindev-20261030/`. 수용에 사용하는 원본·PDF·최종 TSV·대표 PNG는 정식 경로에 보존한다.
