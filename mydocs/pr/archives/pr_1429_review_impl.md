# PR #1429 리뷰 처리 계획

## 목적

외부 기여자 PR #1429를 Collaborator 절차에 맞춰 검토하고, 리뷰 문서와 오늘할일을 archive 경로 기준으로 PR head에 반영한 뒤 GitHub Actions 재확인 후 merge한다.

## 처리 단계

1. PR 메타데이터 확인
   - PR #1429 URL, base/head, draft 여부, mergeable 상태 확인 완료.
   - base는 `edwardkim/rhwp:devel`, head는 `seo-rii/rhwp:render-p26`이다.
   - `maintainerCanModify=true`이므로 리뷰 문서 커밋은 PR head에 직접 push 가능하다.

2. 변경 범위 검토
   - 변경 파일은 `docs/text-ir-v2.md`, `src/paint/text_shape.rs`, `src/paint/text_v2.rs`, `src/renderer/layer_renderer.rs` 네 곳이다.
   - Text IR v2 authority pending gate와 fallback 유지 정책이 중심이다.
   - 관련 이슈는 `Refs #536`이며 자동 close 대상은 아니다.

3. 로컬 targeted 검증
   - `paint::text_v2` 필터: 11 passed
   - `renderer::layer_renderer` 필터: 27 passed
   - `font_resolution_without_shaping_proof_never_emits_public_glyph_runs`: 1 passed

4. 리뷰 문서/오늘할일 커밋
   - `mydocs/pr/archives/pr_1429_review.md`
   - `mydocs/pr/archives/pr_1429_review_impl.md`
   - `mydocs/orders/20260618.md`
   - 문서 전용 변경이므로 `git diff --check`로 공백 오류와 변경 범위를 확인한다.

5. 원격 push
   - PR head가 외부 fork이므로 `maintainerCanModify` 권한을 사용해 `seo-rii/rhwp:render-p26`에 직접 push한다.
   - push 후 PR diff에 archive 리뷰 문서와 오늘할일이 포함됐는지 확인한다.

6. GitHub Actions 재확인
   - 문서 커밋 push 후 required check가 재실행되면 완료를 기다린다.
   - 모든 required check가 통과하면 merge한다.

7. 후속 처리
   - `Refs #536`이므로 이슈 #536은 자동 close하지 않는다.
   - merge 후 `upstream/devel`을 fetch하고 `local/devel`을 동기화한다.
   - 필요 시 임시 로컬 브랜치 `local/pr1429`를 정리한다.

## 주의 사항

- 리뷰 문서는 active `mydocs/pr/` 경로를 거치지 않고 archive 경로에 바로 작성한다.
- 문서 커밋 push 후 CI 통과 여부만 추가하려고 새 문서 커밋을 다시 push하지 않는다.
- GitHub PR/issue 코멘트는 초안을 작업지시자에게 보여주고 승인받은 뒤 등록한다.
