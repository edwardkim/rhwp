# PR #1756 리뷰 구현 메모

## Stage 1. PR 상태 확인

완료.

- PR: https://github.com/edwardkim/rhwp/pull/1756
- 관련 이슈: #1755
- 작성자: @planet6897
- `maintainerCanModify=true`
- 문서 작성 시점 참고값: `mergeable=CONFLICTING`, `mergeStateStatus=DIRTY`
- 기존 CI는 오래된 head 기준 통과 상태지만, 현재 `devel`과 conflict가 있어 최신 검증으로 재사용하지 않는다.

## Stage 2. Conflict 재현과 해소

완료.

```bash
git fetch upstream devel pull/1756/head:local/pr1756 --force
git checkout -B local/pr1756-clean-apply upstream/devel
git cherry-pick -n local/pr1756
```

충돌 파일:

- `src/renderer/typeset.rs`

해소 방식:

- `upstream/devel`에 이미 반영된 #1753 prefill 흐름을 기준으로 유지
- #1755의 `pre_emitted_host_paras` 필드와 전달 경로를 추가
- host 제목 줄 pre-emit 블록을 `prefill_before_deferred_table`의 후속 문단 prefill 앞에 유지

## Stage 3. 로컬 검증

완료.

- `rm -rf target/*`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1755_host_heading_pre_emit -- --nocapture`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_1753_deferred_table_fill_ahead -- --nocapture`
- `cargo fmt --check`
- `git diff --cached --check`
- `env CARGO_INCREMENTAL=0 cargo build`

## Stage 4. 시각 검증

완료.

```bash
python3 scripts/task1274_visual_sweep.py \
  --key pr1756-host-heading \
  --hwp samples/task1753/deferred_takeplace_fill_ahead.hwpx \
  --pdf samples/task1753/deferred_takeplace_fill_ahead-2024.pdf \
  --pages 9,11 \
  --out output/pr1756-visual
```

- 페이지 수: 21 / 21
- 자동 후보: `0/2`
- page 9 review: `output/pr1756-visual/pr1756-host-heading/review/review_009.png`
- page 11 review: `output/pr1756-visual/pr1756-host-heading/review/review_011.png`
- PR 기록 asset:
  - `mydocs/pr/assets/pr_1756_visual_review_p9.png`
  - `mydocs/pr/assets/pr_1756_visual_review_p11.png`

## Stage 5. Push/merge 준비

진행 중.

- 작업지시자 승인을 받아 local conflict-fix 후보를 커밋한다.
- contributor branch `planet6897:pr/devel-1755`에 maintainer 권한으로 push한다.
- push 후 이전 SHA run이 남으면 force-cancel하고, 최신 CI 완료 후 merge 여부를 판단한다.
- merge/close 처리 후 원시 PR #1756 코멘트에 위 asset 링크와 리뷰 결론을 남긴다.
