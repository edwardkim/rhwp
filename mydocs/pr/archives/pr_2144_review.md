# PR #2144 검토 - hwpdocs 12차 10k 검증 보고 및 oracle harness 보강

- PR: https://github.com/edwardkim/rhwp/pull/2144
- 작성자: `planet6897`
- base: `devel`
- 원 head: `bb92228c8a0fc097ad92c03300e751a7d8bd3be5`
- 체리픽 커밋: `6c21f7f30`
- 포함 README: 없음

## 결론

문서와 검증 도구 보강 PR로, renderer 동작 자체를 직접 바꾸지 않는다. 보고서는 열린 PR 누적 상태에서
10k hwpdocs PI-page oracle 결과를 정리하고, `verify_pi_page_vs_hangul.py`는 fresh HWP probe와
재시도 처리를 보강한다.

## 검토 내용

- `mydocs/report/survey_10k_r12_20260710.md`
  - 10,000건 전수, ERR 0, MATCH 92.3%
  - r11 대비 회귀 60건이 결재문서 footer 계열임을 분류
  - #2098/#2138 경계 margin 재보정 근거를 제공
- `tools/verify_pi_page_vs_hangul.py`
  - 한글 oracle 호출 안정성 보강
  - `fresh_hwp`, probe/backoff/retry 경로 추가

## 검증

- reviewer assign 완료
- 체리픽 충돌 없음
- `python3 -m py_compile tools/verify_pi_page_vs_hangul.py` pass
- 누적 브랜치 검증:
  - `git diff --check upstream/devel...HEAD` pass
  - `cargo fmt --check` pass
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --tests` pass
  - `CARGO_INCREMENTAL=0 cargo clippy --all-targets -- -D warnings` pass

## 리스크

- 보고서의 `output/poc/survey10k_r12_0709/` 원자료는 PR에 포함되어 있지 않다.
- 도구 보강은 Python compile까지만 로컬 확인했다. 실제 10k 재실행은 이번 체리픽 검토 범위 밖이다.

## 권고

누적 체리픽 PR에 포함 가능하다. #2143의 근거 문서 역할을 하므로 함께 유지하는 편이 낫다.
