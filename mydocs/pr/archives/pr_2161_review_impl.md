# PR #2161 검토 실행 기록

## 대상

- PR: `edwardkim/rhwp#2161`
- 제목: `planet6897 열린 PR 8건 체리픽 및 검토 기록`
- branch: `codex/planet6897-cherrypick-review-20260710`
- base: `devel`

## 커밋 구성

```text
443642354 Issue #2136: near-top 저장 리셋 상한 2000→2500HU — sb=2500HU 리셋 배제 과적 해소
bb9f9734a Issue #2137: TopAndBottom float 앵커 saved-bounds 신뢰 — 소형 글상자 여백 스필
eed791673 Issue #2098/#2138: 불확실 앵커 footer fit 에 62px 마진 - 10k r12 회귀 60건 대응 (warm PDF 권위 재보정)
6c21f7f30 docs: hwpdocs 12차 10k 검증 (전 PR 적용, MATCH 92.3%) + 오라클 하니스 강화 3건
9a32c90d4 Issue #2145: 개요번호 ^n/^N 레벨 경로 자동코드 구현 — 리터럴 '^N' 출력 수정
f91234b62 Task #2146: NO_LS 라벨 셀 행높이 선언 신뢰 - 사선/Fixed-모순 셀 (21761835 r0 +26.9px→0)
f4519e25f Task #2150: 21761835 잔여 팽창 분해 - 한글 NO_LS fresh 줄높이 공식 확정 + 상쇄 커플링 증명
86a7ffaa5 Issue #2151: HWP3 그림 pgy=0 페이지 시작 후 거짓 쪽 경계 — prev_last_pgy 리셋
9dc07ea9f test: DocumentCore Send 회귀 테스트 추가
c061ee3c7 docs: planet6897 PR 검토 기록 추가
```

## 주요 명령

```bash
git fetch upstream devel --prune
git switch -c codex/planet6897-cherrypick-review-20260710 upstream/devel
gh pr edit 2141 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2142 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2143 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2144 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2147 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2149 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2155 --repo edwardkim/rhwp --add-reviewer jangster77
gh pr edit 2157 --repo edwardkim/rhwp --add-reviewer jangster77
git cherry-pick -x <원 PR head commit 8건>
```

검증:

```bash
git diff --check upstream/devel...HEAD
cargo fmt --check
python3 -m py_compile tools/verify_pi_page_vs_hangul.py tools/hangul_row_heights2.py tools/make_ls_ladder.py tools/probe_ls_ladder.py
CARGO_INCREMENTAL=0 cargo test --profile release-test --tests
CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings
```

## PR 생성

```bash
git push upstream codex/planet6897-cherrypick-review-20260710
gh pr create --repo edwardkim/rhwp --base devel --head codex/planet6897-cherrypick-review-20260710
```

생성된 PR: https://github.com/edwardkim/rhwp/pull/2161

## 후속

1. PR #2161 최신 CI 완료 대기.
2. CI 통과 후 작업지시자 승인 확인.
3. merge 후 원 PR 8건 close/comment 후속 처리.
