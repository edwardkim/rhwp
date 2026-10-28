---
kind: review
status: active
canonical: mydocs/pr/archives/pr_7541_review.md
last_verified: 2026-10-02
---

# PR #7541 리뷰 — 한컴 보안 경고의 수동 PDF 생성 안내

## 최종 판정

**승인.** 문서 변경 범위의 로컬 검증을 통과했다. 작성자 self-review이며 GitHub 승인 review를 게시한 것은 아니다.
Merge 전 조건은 최신 head의 required check·mergeability 확인과 작업지시자의 별도 merge 승인이다.

## 접수 정보

- PR: [#7541](https://github.com/edwardkim/rhwp/pull/7541), 작성자 `jangster77`, base `devel`.
- 기준 base: `83bf0f3c840c9afc1de584c17b167610fd9caf3b` (`upstream/devel`).
- 검토한 매뉴얼 변경: `f6cc13af203bb7fd1aed26f9aaf45f484afca1a9`.
- 경로: collaborator self / intake_and_review / local_validation / review_only_fast_pass.
- 관련 이슈 종료 없음. reviewer assign 없음. PR 생성·push는 작업지시자의 요청으로 수행했다.
- 작성 시점 참고값: Open, MERGEABLE. CI Impact Policy는 대기 중이며 최신 기록 head에서 재확인한다.

## 변경과 검토 범위

[HWP 2024 MCP 매뉴얼](../../manual/mcp_hwp2024Convert_usage.md)에 두 오류 코드, 실패 job의 대기열 반환과
상태 기록 보존, 안전한 대화형 환경에서의 수동 PDF 생성·검증·원본 보존을 추가했다.
문서 보안 설정 하향·자동 접근 승인을 안내하지 않는지 확인했다. 서버 배포 코드는 이 PR에 포함하지 않는다.

## 검증 입력과 결과

- `git diff --check upstream/devel...HEAD`: 통과.
- 정본 `scripts/check_markdown_links.py`의 `collect_broken_links`를 해당 매뉴얼 한 파일에 적용: 오류 0건.
  CLI의 전체 tracked-file 스캔은 중단하고 파일 범위 검사로 대체했다. 새 내부 링크는 추가하지 않았다.
- 변경 경로·front matter·오류 코드·수동 생성 절차·비공개 정보 미포함을 확인했다.
- Cargo·WASM·렌더링 검증: 문서만 변경하므로 비해당.
- 조판 원칙 준수 검토: 비해당. 조판 코드·baseline·golden·fixture 변경과 시각 개선 주장이 없다.
- 검증 입력 커밋 확인: 비해당. 이 문서 PR의 수용 검증에는 HWP/HWPX/PDF 파일을 사용하지 않았다.

## 시각 증적과 남은 차이

비해당. 수동 생성 결과의 시각 일치를 주장하지 않는다. 사용자가 원본을 안전하게 열 수 없는 경우에는
정상 사본을 요청하도록 명시했다. Merge와 merge 후 contributor comment는 이번 요청 범위에 포함하지 않는다.
