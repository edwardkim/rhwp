# PR #1743 리뷰 — #1692 SO-SUEOP HWP3/HWPX 렌더링 정합 보정

- PR: #1743 `task 1692: SO-SUEOP HWP3/HWPX 렌더링 정합 보정`
- 작성자: @jangster77
- 기준 브랜치: `devel`
- PR head: `8e2f89ae9c721de18881551ed114efc6eea2e6d0` (문서 작성 시점 참고값)
- 규모: 41 files, +3329/-307
- 관련 이슈: #1689, #1692, #1693, #1694, #1695, #1696, #1697, #1698, #1699
- 검토 중 상태: `MERGEABLE`, `CLEAN` (GitHub required checks 통과)
- 최종 처리: admin merge 완료
- merge commit: `f50aa4ef7a011817d8ae0ae0e41b817d42f4b030`

## 변경 요약

초기 PR 범위는 HWP3 글자색 보존이었으나, SO-SUEOP 샘플의 HWP/HWPX/PDF 페이지 단위 시각 비교 과정에서
동일 문서군의 구조적 렌더링 차이를 함께 보정하는 범위로 확장되었다.

핵심 변경:

- HWP3 글자색 인덱스를 `CharShape.text_color`로 보존
- HWP3 `Outline:` 컨트롤과 본문 개요번호/해답 번호를 IR로 복원
- HWP3 line box 폭 계산에 ParaShape 좌우 여백과 indent를 반영
- 페이지 하단 vpos reset 직전 빈 문단이 불필요한 새 페이지를 만들지 않도록 보정
- HWP3/HWPX 미주 내부 vpos reset 흐름을 정규화
- 미주 separator line은 폭과 선 종류가 모두 있는 경우에만 렌더링
- HWP3 미주 번호 표지의 문자 모양을 본문 첫 글자 스타일과 맞춤
- HWP3 HMapsi OLE preview fallback, 외부 이미지 경로, 머리말 밑줄을 복원
- HWP3 관계도 박스를 1x1 표 구조로 매핑하고 선문자/원문자 표시를 복원
- SO-SUEOP HWP/HWPX/PDF 기준 샘플과 회귀 테스트 추가

## 로컬 검증

아래 검증은 PR head `8e2f89ae9c721de18881551ed114efc6eea2e6d0` 기준으로 통과했다.

- `cargo build`
- `cargo test issue_1692 --test issue_1692 -- --nocapture`
- `cargo test issue_1293_clean_visual_sweep_targets_keep_page_counts_and_shape_profiles --test issue_1139_inline_picture_duplicate -- --nocapture`
- `cargo clippy --all-targets -- -D warnings`
- `env CARGO_INCREMENTAL=0 cargo test --all-targets`
- `env CARGO_INCREMENTAL=0 cargo test --profile release-test --tests`
- `git diff --check`

`release-test` 프로필의 통합 테스트는 macOS 로컬에서 약 3분 19초 동안 실행됐고, visual roundtrip baseline 포함
통과했다.

## 시각 검증

`mydocs/manual/verification/visual_sweep_guide.md` 기준으로 SO-SUEOP 샘플을 페이지 단위로 재확인했다.

- 기준 PDF: `pdf/SO-SUEOP-2024.pdf`, 46쪽
- HWP3 샘플: `samples/SO-SUEOP.hwp`, 46쪽
- HWPX 샘플: `samples/SO-SUEOP.hwpx`, 46쪽
- HWP3 sweep: 46/46쪽 렌더 가능, p24 order 후보 1건은 별도 관찰 항목
- HWPX sweep: 46/46쪽 렌더 가능, flag 0
- p22: 관계도 박스, 원문자, 선문자, 미주 표지 위치 보정 확인
- p43~p46: 미주 범위가 PDF 기준 흐름과 맞도록 확인
  - p43: 1~58
  - p44: 59~129
  - p45: 130~191
  - p46: 192~223
- p45 하단 footer overlap 재발 없음

## 이슈 매핑

PR #1743의 현재 head는 #1692 단일 현상을 넘어 #1689 하위 SO-SUEOP 렌더링 이슈 여러 건을 함께 다룬다.

- #1692: HWP3 글색상 손실 — 해결 후보
- #1693: HWP3 개요번호/Outline 필드 미처리 — 해결 후보
- #1694: HWP3 페이지 수 증가 및 하단 overflow — 해결 후보
- #1695: HWP3 LINE_SEG vpos reset/rewind 처리 문제 — 상당 부분 반영, close 여부는 최종 판단 필요
- #1696: HWP3 도형/선/글상자 렌더링 문제 — 해결 후보
- #1697: HWP3 다단 정보 누락 — 해결 후보
- #1698: HWP3 미주 해답 페이지 레이아웃/정렬 문제 — 해결 후보
- #1699: SO-SUEOP HWP3 폰트 fallback 검증 — 별도 open 유지

이 매핑은 PR comment로도 남겼다.

- https://github.com/edwardkim/rhwp/pull/1743#issuecomment-4864611550

## GitHub Actions

문서 작성 시점의 PR head `8e2f89ae9c721de18881551ed114efc6eea2e6d0` 기준 상태:

- CI preflight: success
- CodeQL preflight: success
- Render Diff preflight: success
- WASM Build: skipped
- Analyze (javascript-typescript): success
- Analyze (python): success
- Analyze (rust): success
- Canvas visual diff: success
- CodeQL: success
- Build & Test: success

최신 head 기준 required checks 통과 후 merge 했다.

## 리뷰 결과

Blocking finding 없음.

SO-SUEOP HWP3/HWPX/PDF 기준 페이지 수와 주요 시각 차이를 재검증했고, 하드코딩으로 맞추던 임시 접근은 제거한
뒤 일반화된 HWP3/HWPX 흐름 보정으로 정리했다. 다만 PR 제목과 본문은 초기 글색상 중심으로 시작했기 때문에,
최종 merge 전후에는 현재 범위에 맞춰 후속 문서와 close 대상 이슈를 명확히 남겨야 한다.

## merge 후 확인

- PR #1743 admin merge 완료: `f50aa4ef7a011817d8ae0ae0e41b817d42f4b030`
- merge 시각: 2026-07-02 19:32 KST
- `devel`이 default branch가 아니어서 `Closes #1692` auto-close는 동작하지 않았다.
- #1692, #1693, #1694, #1696, #1697, #1698은 수동 close 완료.
- #1689, #1695, #1699는 open 유지.
- #1695는 #1743에서 페이지 수와 미주 흐름 관련 증상이 상당 부분 줄었지만, 본문 6/23/28쪽 및 미주 42/44쪽의
  원본 LINE_SEG vpos reset/rewind 힌트 해석 규칙까지 검증한 것은 아니므로 open 유지.
- #1699는 구조적 레이아웃 보정 뒤에도 폰트 fallback 조건별 줄 높이/폭 비교가 별도로 필요하므로 open 유지.

## 후속 처리 계획

- #1692 close comment: https://github.com/edwardkim/rhwp/issues/1692#issuecomment-4864806076
- #1693 close comment: https://github.com/edwardkim/rhwp/issues/1693#issuecomment-4864806208
- #1694 close comment: https://github.com/edwardkim/rhwp/issues/1694#issuecomment-4864806357
- #1696 close comment: https://github.com/edwardkim/rhwp/issues/1696#issuecomment-4864806486
- #1697 close comment: https://github.com/edwardkim/rhwp/issues/1697#issuecomment-4864806638
- #1698 close comment: https://github.com/edwardkim/rhwp/issues/1698#issuecomment-4864806850
- #1695는 LINE_SEG vpos reset/rewind 일반 규칙 검증 후 별도 close 판단.
- #1699는 font fallback 후속으로 open 유지.
- #1689 parent 이슈는 하위 이슈 처리 상태 확인 후 별도 판단
- PR 감사/후속 코멘트와 오늘할일은 문서-only PR로 반영
