---
kind: snapshot
status: active
canonical: mydocs/pr/archives/pr_7152_review.md
last_verified: 2026-09-15
---

# PR #7152 — 메인터너 보정 및 기록 단계

## 승인 범위

사용자는 로컬 보정·검증·전후 스크린샷과 초안을 확인한 뒤, 보정 code push → 새 CI 확인 → 리뷰 문서 push → 최종 head 확인 → 코멘트를 포함한 Approve 게시를 승인했다. merge와 이슈 close는 이번 범위에 없다.

[collaborator 직접 보정 경로](../../manual/pr_review/collaborator_external_pr.md#931-contributor-pr-head-직접-보정)를 사용했다. 기존 preview worktree `/private/tmp/rhwp-review-7152`에서 원 PR head로 시작한 `codex/pr7152-review-20260915` 한 branch에 code와 기록을 순서대로 추가한다. 사용자 변경이 있는 기본 checkout의 branch·파일은 유지한다.

## 커밋과 단계

| 단계 | SHA / 산출물 | 결과 |
| --- | --- | --- |
| contributor 원 변경 | `9787d549237b4c678605c1ade7ba116ee2515baf` — feat(i18n): 메뉴·툴바 영어 표시 (2/4) | 보존. 원 작성자 rubidus-api, rewrite 없음 |
| source head 확인 | API, `git ls-remote`, 로컬 시작 SHA | 동일 확인; maintainerCanModify=true |
| 보정 범위 | 영어 부모 폭, 선택 상자 격자 수용, 영어 828/982px 전환, controller query | 완료. Format Painter의 기존 세로 정렬은 제외 |
| code·회귀 검사 | `e917cffb9f70dfc73451a32fb4c1bf3ef9cffc8c` — fix(studio): 영어 서식 도구 모음의 필드 폭과 반응형 경계 보정 | contributor commit 바로 위의 별도 commit; source branch push·새 CI 완료 |
| 로컬 검증 | tsc, npm test, production build, responsive 전체 E2E, 원 PR 음성 대조 | 완료. [review 결과](pr_7152_review.md#4-완료-검증과-실행-증거) |
| 리뷰 기록·초안 | review, 이 impl, comment draft, assets | 별도 docs commit으로 보존. 이 문서를 포함하므로 자신의 commit SHA를 본문에 미리 쓰지 않음 |
| code push·새 CI | `e917cffb9` | 완료. CI·CodeQL·Render Diff·Adapter·Proptest 성공. 정확한 run은 review와 JSON에 기록 |
| 최종 리뷰·증적 push | 이 후행 문서 묶음 | code CI 완료 후 준비. push 전 merge tree·공백·링크·기존 오늘할일 보존 검사 |
| GitHub Approve review | 사용자 승인 완료 | 최종 문서 head 검사 뒤 핵심 이미지 5장과 본문을 게시. 실제 결과는 GitHub review에 기록 |
| merge·후속 이슈 | 미실행 | 이번 승인 범위 밖 |

## 실행·증거 주의점

- 초기 browser 캡처에서 viewport 전환 직후 이미지 크기가 이전 프레임에 남는 현상을 확인했다. 실제 `innerWidth` 측정과 최종 JPEG 해상도를 대조하고 잘못된 중간 캡처는 최종 증거에서 제외했다.
- 새 E2E를 원본 CSS/controller에 적용한 음성 대조 후, 보정 파일을 복원하고 전체 responsive suite를 통과했다. 원본 재현을 위해 source를 잠시 복원한 상태는 최종 commit에 남지 않았다.
- 코드 검증 후 CSS 설명 주석만 정리했다. 테스트·실행 코드·CSS 선언 변경은 없었다.
- 저장소의 PNG 관례 대신 browser 도구가 직접 반환한 JPEG를 그대로 보존했다. UI 증거이며 한컴 PDF 비교/OVL 대상은 아니다. 원본 래스터 변경 없이 HTML에서 도구 모음 영역을 보여 주고, 그 비교 페이지를 다시 캡처했다.
- 최초 preview용 `rhwp-studio/public/pr7152-boundary.html`과 `public/pr7152-evidence/`는 로컬 표시용 untracked 파일이다. 소스·docs commit에 넣지 않는다. 커밋 대상 비교 HTML과 원본은 `mydocs/pr/assets/`에 있다.

## 후행 기록 반영과 정리

코드 push 전 원격 SHA·권한·LFS 비대상과 dry-run을 확인했고 원본 `9787d5492`에서 `e917cffb9`로 fast-forward했다. 새 code CI가 끝난 뒤 같은 로컬 branch의 review-only 기록을 확정했다. 원 head의 녹색 CI로 보정 검증을 대체하지 않았다.

문서 push 전 최신 devel/source ref를 고정하고 `git merge-tree --write-tree`와 실제 merge tree의 Markdown 링크·공백·기존 오늘할일 보존을 확인한다. source에 없는 오늘할일 파일을 새로 만들면 최신 devel과 add/add 충돌이 발생하므로 선택 항목인 오늘할일 갱신은 생략했다. 이를 위해 다른 PR 내용을 source로 복사하거나 devel을 merge하지 않았다.

최종 문서 head의 aggregate 검사가 통과하면 코멘트 본문에 asset commit SHA 고정 raw URL을 넣고 UTF-8 JSON 파일을 `gh api`에 전달해 해당 `commit_id`에 APPROVE 리뷰를 게시한다. 게시 뒤 API로 head·리뷰 상태·한국어 본문·이미지 링크를 다시 읽는다. 별도 일반 코멘트를 중복 게시하지 않는다.

merge와 이슈 close는 수행하지 않는다. contributor branch·사용자 기본 checkout·공유 Cargo cache는 보존하고, 사용자가 확인 중인 preview 서버·worktree도 유지한다.

## 코멘트 초안 표현 보완

사용자 요청에 따라 비교 페이지 전체를 캡처한 이미지를 코멘트에서 제거했다. 제목·원인·보정·경계 표·검증 결과는 Markdown으로 작성했고, 원본 도구 모음의 핵심 영역만 담은 JPEG 5장을 각각 배치했다. 정렬 패널은 열린 상태를 명시했으며, 이미지 안에 설명용 접기 버튼이나 링크를 넣지 않았다. 원본 전체 JPEG와 상세 비교 페이지는 기존 리뷰 증거로 보존했다. 영역 좌표·출처·해시는 `pr7152_evidence.json`에 추가했다. 제품 코드·검증 결과·원격 상태는 변경하지 않았다.

## 적용 경로

기본 경로는 collaborator_external_pr 9.3.1 직접 보정이며, intake_and_review·local_validation·review_only_fast_pass를 함께 적용했다. 현재 source는 `e917cffb9`이며 이후 허용된 기록만 이어 붙인다. 정책상 별도 docs PR이나 기여자에게 기록 제출 요청을 만들지 않는다.
