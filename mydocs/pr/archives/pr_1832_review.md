# PR #1832 리뷰 — baseline 대조 도구 PUA 수식 글리프 오탐 수정

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1832 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1829` |
| 작성 시점 참고 head | `3a0bd9b496826c6ebffd7481e113405cfd94d080` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `tools/compare_line_baselines.py` 의 baseline pair key 에서 한컴 PUA 수식 글리프 영향을 제거한다.
- #1828 위에 쌓인 후속 PR 이며, 같은 커밋 `69afc43631b7` 을 포함하고 있었다.

## 로컬 검증

- #1828 공통 커밋 `69afc43631b7` 은 이미 적용되어 #1832 체리픽 중 empty 로 skip 했다.
- 실제 추가 커밋: `3a0bd9b49682` -> `d2ac20797`
- 충돌: 없음
- focused 검증: `python3 -m py_compile tools/compare_line_baselines.py` 통과.
- 누적 검증: release-test integration, Clippy 통과.

## 판단

비교 도구의 오탐 키를 줄이는 변경이며, #1828 merge 이후 적용되는 것이 자연스럽다. 코드 렌더 경로가 아니라
검증 도구 변경이므로 별도 visual sweep 은 수행하지 않았다.

## 결론

merge 후보. merge 순서는 #1828 다음 #1832.
