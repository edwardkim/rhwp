# PR #1905 리뷰 진행 기록

## Stage 1. 메타 확인

완료.

- reviewer `jangster77` 지정.
- 검토 기준 PR head `391550d9e1e14db240c92c65c982a1e06f57897b` 확인.
- base `devel`, draft 아님.
- 문서 작성 시점 mergeable `MERGEABLE` / `CLEAN`.
- 변경 파일 12개, +466/-29.

## Stage 2. 변경 내용 검토

완료.

- PR 본문의 4개 원인 축과 코드 변경을 대조했다.
- `$con` 빈 컨테이너는 `ShapeComponentAttr.ctrl_id` 를 근거로 판별하므로 샘플명/페이지 번호 하드코딩이 아니다.
- HWP3 크기 기준 bit 기록은 기존 `parse_common_obj_attr` 의 디코드 매핑과 대응한다.
- rendering matrix fallback 변경은 raw/explicit/rotation 경로를 유지하고 fallback translation 만 identity 로 바꾼다.
- null tab marker 필터는 직렬화기가 내보내는 "데이터 없음" marker 만 제외하고, HWPX 실 tab 확장 보존 테스트는 유지된다.

## Stage 3. 충돌 검증

완료.

```bash
git merge-tree $(git merge-base HEAD upstream/devel) HEAD upstream/devel
```

출력 없음. `upstream/devel` 기준 내용 충돌 없음으로 판단했다.

## Stage 4. 로컬 검증

완료.

검증 전 `/Users/tsjang/rhwp/target` 하위 항목을 삭제했다.

```bash
git diff --check upstream/devel...HEAD
env CARGO_INCREMENTAL=0 cargo test --test issue_1892
env CARGO_INCREMENTAL=0 cargo test --test issue_1244_tab_extended_fallback
env CARGO_INCREMENTAL=0 cargo run --bin rhwp -- render-diff samples/issue1892_hwp3_drawing_group_roundtrip.hwp --via hwp
env CARGO_INCREMENTAL=0 cargo run --bin rhwp -- render-diff samples/issue1892_hwp3_tab_roundtrip.hwp --via hwp
env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
cargo fmt --check
env CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

모두 통과.

## Stage 5. 리뷰 결론

완료.

- 코드/검증 기준 merge 후보.
- 문서 품질 이슈 1건: `mydocs/report/task_m100_1892_report.md` 의 `<!-- CARGO_TEST_RESULT -->` placeholder 를 maintainer 권한으로 보정했다.

## Stage 6. Maintainer 보정

완료.

```bash
git commit -m "docs: #1892 보고서 검증 결과 보정"
git push https://github.com/planet6897/rhwp.git HEAD:pr/devel-1892
```

- 보정 커밋: `f4dee14aa854e514232ea480161c52963305020a`
- PR head 갱신 확인: `f4dee14aa854e514232ea480161c52963305020a`
- GitHub Actions 는 새 head 기준 preflight 성공, heavy job skip, `Build & Test` 성공.

## Stage 7. Merge

완료.

```bash
gh pr merge 1905 --repo edwardkim/rhwp --merge --admin
```

- merge commit: `23aea574acca7d1f20cfeb27e4ab4d06df8ad317`
- merged at: 2026-07-04T14:20:46Z
- #1892 상태: GitHub auto-close 로 `CLOSED`

## Stage 8. 후속 문서 PR

진행 중.

- 옵션 2 경로로 원 코드 PR merge 후 review 문서만 별도 docs-only PR 로 반영한다.
- review 문서는 active 경로가 아니라 `mydocs/pr/archives/` 경로로 준비한다.
