# PR #1837 리뷰 — #1809 v2 RowBreak top pad/flow extra 소스 무관화

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1837 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1809-v2` |
| 작성 시점 참고 head | `df6cca3a3feed268b5cd1fe892f3ad64c721b326` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- #1809 v2 로 `RowBreak` top pad 와 컷 이월 flow extra 를 source-agnostic 하게 적용한다.
- `admrul_0556`, `admrul_0072` 잔여 케이스 해소가 핵심이다.
- #1825 의 부분 수정 이후 이어서 merge 되어야 하는 후속 PR 이다.

## 로컬 검증

- 체리픽 커밋: `df6cca3a3fee` -> `97d234910`
- 충돌: 없음
- 누적 검증: focused Rust test, release-test integration, Clippy 통과.

## 판단

#1825 가 남긴 #1809 잔여 대표 케이스를 보완하는 PR 이며, table layout 적용 범위가 문서화된 목적과 일치한다.
로컬 누적 검증에서 regression 은 확인되지 않았다.

## 결론

merge 후보. #1825 와 #1837 을 모두 merge 한 뒤 #1809 close 가능 여부를 최종 확인한다.
