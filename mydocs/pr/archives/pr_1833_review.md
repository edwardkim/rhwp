# PR #1833 리뷰 — golden SVG LF 고정

## 메타

| 항목 | 내용 |
|------|------|
| PR | https://github.com/edwardkim/rhwp/pull/1833 |
| 작성자 | @planet6897 |
| base / head | `devel` / `pr/devel-1786` |
| 작성 시점 참고 head | `3465baaefa7db46f03353bff54d766fab526a1d2` |
| 작성 시점 참고 상태 | `MERGEABLE`, GitHub Actions 통과 |
| reviewer assign | @jangster77 지정 완료 |

## 변경 범위

- `.gitattributes` 에 golden SVG LF 고정 규칙을 추가한다.
- Windows autocrlf checkout 에서 `svg_snapshot` 이 흔들리는 문제를 줄이는 저장소 메타 변경이다.

## 로컬 검증

- 체리픽 커밋: `3465baaefa7d` -> `04bd43185`
- 충돌: 없음
- focused 검증: `env CARGO_INCREMENTAL=0 cargo test --test svg_snapshot` 통과.
- 누적 검증: release-test integration, Clippy 통과.

## 판단

라인 엔딩 정책 변경이며 코드 실행 경로 변경은 없다. golden SVG snapshot 회귀 방지 목적과 `.gitattributes` 변경이
일치한다.

## 결론

merge 후보. #1786 issue close 여부를 merge 후 확인한다.
