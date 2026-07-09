# PR #2082 리뷰 — #2006 전면 TAC 이미지 스택 쪽당 1장 분할

- 작성 시각: 2026-07-09 16:30 KST
- PR: https://github.com/edwardkim/rhwp/pull/2082
- 작성자: `planet6897`
- base / head: `devel` / `fix/2006-stacked-tac-image-split`
- 문서 작성 시점 참고 head: `17be15e4dfe027e53e091526403bb756a60c8874`
- 문서 작성 시점 참고 merge state: `BEHIND`
- reviewer assign: `jangster77` 요청 완료
- 처리 경로: `codex/planet6897-prs-review-20260709` 에서 여러 PR 누적 체리픽 검토

## 변경 범위

- `src/renderer/typeset.rs`: 전면급 TAC picture-only 라인이 연속되는 경우 내부 vpos page break 경계를 확장.
- `mydocs/report/task_m100_2006_report.md`: contributor 보고서.
- 새 샘플/기준 PDF는 PR diff에 포함되지 않았다.

## 체리픽 검토

- 누적 체리픽 순서: 3/11.
- 적용 커밋: `6dc54357e` (`Issue #2006: 전면 tac 이미지 스택 쪽당-1장 분할 ...`).
- 충돌: 없음.
- 선행 PR 의존: #2073 이후 같은 통합 브랜치에서 적용했으나 파일 충돌은 없었다.

## 검증

- GitHub Actions: 원 PR head 기준 `Build & Test`, `CodeQL`, `Canvas visual diff` 등 성공 확인.
- `git diff --check upstream/devel...HEAD`: 통합 브랜치 fixup 이후 통과.
- `cargo fmt --check`: 통과.
- 현재 저장소에는 PR 본문이 언급하는 `1790387` 원 실문서와 기준 PDF가 없어 MCP 변환/visual sweep을 직접 수행할 수 없었다.
- contributor 보고서상 검증값: `1790387` 130 -> 141, 한글 146, `1430000` 403 유지.

## 판단

- 체리픽 가능 여부: 기계적 체리픽은 가능.
- 로컬 재현성 공백: `1790387` 원본 HWP/HWPX 와 기준 PDF가 없어 PR의 핵심 페이지 수 주장(130 -> 141, 한글 146)은 독립 재현하지 못했다.
- 통합 PR 처리: 머지 차단 사유로 제출하지 않고, 원 PR 보고서와 GitHub CI를 근거로 통합 후보에 포함한다.
- 후속 검토 의견: 페이지 수 변화와 시각 검증이 필요한 PR은 원본 HWP/HWPX 없이는 MCP PDF 생성과 장기 재현이 불가능하다. merge 후 원 PR에는 다음부터 `1790387` 같은 타깃 원본 HWP/HWPX 와 기준 PDF를 함께 첨부해 달라는 의견을 남긴다.

## 후속 검토 코멘트 초안

@planet6897 작업 감사합니다. 체리픽 통합과 로컬 cargo 검증은 통과했습니다.

사후 확인 의견으로 하나 남깁니다. 이 PR은 페이지 수 변화와 시각 검증이 핵심인데, PR diff와 저장소에서 `1790387` 원본 HWP/HWPX 및 기준 PDF를 찾을 수 없어 maintainer 쪽에서 MCP PDF 생성이나 독립 visual sweep을 수행하지 못했습니다.

다음에 페이지 수나 시각 검증이 필요한 PR을 올려주실 때는, 타깃 원본 HWP/HWPX 파일과 한컴 2020/2024 등에서 저장한 기준 PDF를 함께 첨부해 주세요. 기준 PDF만 없으면 maintainer 측에서 HWP 2020 MCP로 산출할 수 있지만, 원본 HWP/HWPX가 없으면 검증과 회귀 추적이 어렵습니다.
