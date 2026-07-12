# PR #2116 Review

## 메타

- PR: https://github.com/edwardkim/rhwp/pull/2116
- 작성자: `planet6897`
- base: `devel`
- 제목: `Issue #2093: 실문서 권위 검증 보강 — 1192000 원본 + 한글2022 PDF(16쪽 정합) + fixture oracle 의도 명시`
- 문서 작성 시점 참고값: `MERGEABLE` / `BEHIND`
- reviewer assign: `jangster77` 완료

## 변경 범위

- `samples/task2093/1192000_hydrogen_policy_research.hwp` 추가
- `pdf/task2093/1192000_hydrogen_policy_research-2022.pdf` 추가
- `pdf/task2093/saved_single_line_spacing_after-2022.pdf` 추가
- `samples/task2093/README.md` 갱신
- `tests/issue_2093_1192000_real_doc_pin.rs` 추가

이 PR은 #2093 synthetic fixture의 한글 재조판 oracle 불일치를 명시하고, 실문서 기준으로 한글 2022 정합을 검증하는 자료를 보강한다.

## 검증 결과

- conflict 제외 재확인: #2116은 GitHub 기준 `MERGEABLE/BEHIND`, 로컬 `devel` 위 최종 delta cherry-pick 성공.
- PDF 확인:
  - `pdf/task2093/1192000_hydrogen_policy_research-2022.pdf`: `Pages: 16`, A4, Hancom PDF
  - `pdf/task2093/saved_single_line_spacing_after-2022.pdf`: `Pages: 1`, A4, Hancom PDF
- synthetic fixture PDF 텍스트 추출:
  - `FILL`
  - `TAIL LINE WITH SPACING AFTER`
  - `PAGE2 HEAD`
  - 모두 1쪽에 추출되어 README의 "한글 2022에서도 1쪽" 설명과 일치.
- 샘플 SHA-256:
  - `samples/task2093/1192000_hydrogen_policy_research.hwp`
  - `34277a37bd81e69aa194ae246ef34c5f9d98d9353449089ad2fd33ceb8bfacd0`
- 로컬 테스트:
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2093_1192000_real_doc_pin` 통과
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2093_saved_single_line_spacing_after` 통과
- 누적 검증:
  - #2114/#2115/#2116/#2117 체리픽 누적 브랜치에서 추가 핀 테스트 4개 모두 통과
  - `git diff --check upstream/devel..HEAD` 통과

## 판단

PR 설명의 핵심 주장, 즉 synthetic fixture는 한글 2022 재조판 기준 1쪽이고 실문서 `1192000`은 한글 2022/rhwp 모두 16쪽이라는 주장이 실제 PDF/테스트와 일치한다. README도 synthetic fixture를 한글 정답지가 아니라 rhwp 저장 LINE_SEG 신뢰 시맨틱 핀으로 분리해 설명한다.

Blocking finding 없음. merge 후보로 판단한다.
