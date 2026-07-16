# PR #2294 검토 - planet6897 수용 PR 3건 통합 반영

- 검토일: 2026-07-15
- 작성자: `jangster77` (collaborator-mediated 통합 PR)
- 대상: [PR #2294](https://github.com/edwardkim/rhwp/pull/2294)
- 관련 원 PR: [PR #2273](https://github.com/edwardkim/rhwp/pull/2273), [PR #2275](https://github.com/edwardkim/rhwp/pull/2275), [PR #2276](https://github.com/edwardkim/rhwp/pull/2276)
- 관련 이슈: [Issue #2234](https://github.com/edwardkim/rhwp/issues/2234), [Issue #2261](https://github.com/edwardkim/rhwp/issues/2261), [Issue #2148](https://github.com/edwardkim/rhwp/issues/2148)
- 범위: 최신 `upstream/devel` 위에 contributor 원 커밋 세 개를 순서대로 체리픽했다. 원 PR별 reviewer `jangster77` 지정과 본문·대화·소스 검토를 마쳤다.

## 변경 범위와 판단

- [PR #2273](https://github.com/edwardkim/rhwp/pull/2273): 다섯 GitHub Actions workflow의
  `actions/setup-node` 참조만 v4에서 v6으로 올린다. 캐시 키, Node 매트릭스, job 구조는 불변이다.
- [PR #2275](https://github.com/edwardkim/rhwp/pull/2275): 캐럿 진입이 차단된 문서를
  `PROTECTED_SKIP`으로 TSV에 분리한다. `page_delta`는 계속 기록하지만 일반 mismatch 실패 집계에는
  넣지 않는다. 이 분류만으로 [Issue #2261](https://github.com/edwardkim/rhwp/issues/2261)을 해결로
  처리하지 않는다.
- [PR #2276](https://github.com/edwardkim/rhwp/pull/2276): renderer 동작을 바꾸지 않는 생성기 마커
  조사 도구다. HWPX 메타 추출은 정규식 기반이므로 namespace·attribute 순서 변형 대응은 후속 개선으로
  남긴다.

세 변경 모두 renderer 출력 경로, pagination, WASM, UI를 바꾸지 않아 visual sweep 대상이 아니다.

## 검증

- `actionlint -ignore 'SC2086|SC2035'`로 변경 workflow 다섯 개의 문법을 확인했다.
- `python3 -m py_compile tools/verify_pi_page_vs_hangul.py tools/hwp_generator_probe.py`를 통과했다.
- `python3 tools/hwp_generator_probe.py samples/hwpx_sample2.hwpx`는 marker `119473`을 정상 추출했다.
- `git diff --check upstream/devel...HEAD`를 통과했다.

원 PR별 상세 판단은 `pr_2273_review.md`, `pr_2275_review.md`, `pr_2276_review.md`에 보존한다.

## 최종 권고

**최신 PR head의 GitHub Actions 통과 후 squash merge 수용.** merge 뒤 [Issue #2234](https://github.com/edwardkim/rhwp/issues/2234), [Issue #2261](https://github.com/edwardkim/rhwp/issues/2261), [Issue #2148](https://github.com/edwardkim/rhwp/issues/2148)의 자동 close 여부를 확인하고, 원 PR 세 건에는 통합 PR로 반영됐음을 코멘트한 뒤 close한다.
