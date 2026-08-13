---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-13
---

# PR #4732 검토 - 트랙 B 파서 재귀 깊이 하드닝 백로그 정합

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#4732](https://github.com/edwardkim/rhwp/pull/4732) |
| 작성자 / 원 head | @kevin9327 / `d532237c7c` |
| 검토 적용 commit | `c49bdf0c3` |
| 메인터너 보정 | `5cc2f994b2` `docs(roadmap): R13 파서 재귀 후속 정합 보정` |
| 통합 PR | [#4736](https://github.com/edwardkim/rhwp/pull/4736) |
| base / code candidate | `devel` `e550a270f4` / `5cc2f994b2` |

## 범위와 보정

원 PR은 [이슈 #4730](https://github.com/edwardkim/rhwp/issues/4730)에서 실측한 HWPX·HWP5
파서 재귀 깊이 하드닝 네 경로를 트랙 B에 기록한다. 그러나 전역 R21은 이미 트랙 C의
경합 유실 재현에 배정돼 있어 새 R21을 트랙 B에 추가하면 번호와 집계가 충돌한다. 또한
HWPX `<hp:container>` 보정 PR [#4731](https://github.com/edwardkim/rhwp/pull/4731)은 검토 중으로,
`devel` 착지 사실로 기록할 수 없다.

통합 보정은 새 R 번호를 만들지 않고 R13 악성 코퍼스의 후속 항목에 재귀 깊이 하드닝을
편입했다. #4731은 구현 검토 중으로 정확히 표시하고, 나머지 세 재귀 경로와 HWP3/HML의
256 깊이 선례, 상한 초과 거부·정상 깊이 통과의 회귀 계약을 남겼다. R17에는 정적 분석
발견도 퍼징→코퍼스 유입 경로를 쓴다는 연결을 추가했다.

## 완료한 검증

- `git diff --check upstream/devel...HEAD`
- `python scripts\check_document_metadata.py` — 555개 문서, 이상 없음
- `python scripts\check_markdown_links.py mydocs\tech\agent_roadmap\README.md mydocs\tech\agent_roadmap\track_b_guards_security.md` — 이상 없음
- `python tools\roadmap_progress.py` — 결번·중복 0, README 집계 일치
- code candidate `5cc2f994b2`의 GitHub CI preflight·CodeQL preflight·Build & Test
  aggregate 성공; 문서 전용 fast-pass로 heavy Rust 작업은 skipped

`mydocs`만 바꿨으므로 Cargo·WASM·renderer 시각 검증은 적용 대상이 아니다.

## 판정

**self-review 수용.** 원 contributor commit은 보존하고, 최신 `devel` 위 통합 PR에 보정과
검토 기록을 분리한다. 이 문서와 오늘할일을 올린 뒤에는 최신 trailing head의 fast-pass와
mergeability를 재확인한 다음 merge한다.
