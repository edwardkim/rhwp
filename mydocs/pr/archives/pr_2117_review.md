# PR #2117 Review

## 메타

- PR: https://github.com/edwardkim/rhwp/pull/2117
- 작성자: `planet6897`
- base: `devel`
- 제목: `Issue #2097: 실문서 권위 검증 보강 — 1730000 원본 + 한글2022 PDF(1쪽 정합) + fixture oracle 의도 명시`
- 문서 작성 시점 참고값: `MERGEABLE` / `BEHIND`
- reviewer assign: `jangster77` 완료

## 변경 범위

- `samples/task2097/1730000_selection_report.hwp` 추가
- `pdf/task2097/1730000_selection_report-2022.pdf` 추가
- `pdf/task2097/none_table_declared_fits-2022.pdf` 추가
- `samples/task2097/README.md` 갱신
- `tests/issue_2097_1730000_real_doc_pin.rs` 추가

이 PR은 #2097 synthetic fixture의 한글 재조판 oracle 불일치를 명시하고, 실문서 기준으로 한글 2022 정합을 검증하는 자료를 보강한다.

## 검증 결과

- conflict 제외 재확인: #2117은 GitHub 기준 `MERGEABLE/BEHIND`, 로컬 `devel` 위 최종 delta cherry-pick 성공.
- PDF 확인:
  - `pdf/task2097/1730000_selection_report-2022.pdf`: `Pages: 1`, A4, Hancom PDF
  - `pdf/task2097/none_table_declared_fits-2022.pdf`: `Pages: 2`, A4, Hancom PDF
- synthetic fixture PDF 텍스트 추출:
  - 1쪽: `AFTER TABLE`
  - 2쪽: `BIG ROW`, `MID ROW`, `TAIL ROW EXPANDING`
  - README의 "한글 2022에서도 p1=AFTER TABLE, p2=표" 설명과 일치.
- 샘플 SHA-256:
  - `samples/task2097/1730000_selection_report.hwp`
  - `70e279871a663bf25bb76853e09d8dd408074c581cf28a81e5ee043874cc2328`
- 로컬 테스트:
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2097_1730000_real_doc_pin` 통과
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2097_none_table_declared_fits` 통과
- MCP 교차 확인:
  - `hwp2020Convert` MCP bridge `convert_local_document` 로 `1730000_selection_report.hwp`를 PDF 변환.
  - 결과: `status=success`, `run_status=0`, `validation=ok`, 출력 PDF 1쪽 A4.
- 누적 검증:
  - #2114/#2115/#2116/#2117 체리픽 누적 브랜치에서 추가 핀 테스트 4개 모두 통과
  - `git diff --check upstream/devel..HEAD` 통과

## 판단

PR 설명의 핵심 주장, 즉 synthetic fixture는 한글 2022 재조판 기준 p1/p2 배치가 rhwp fixture 기대와 다르고, 실문서 `1730000`은 한글 2022/rhwp 모두 1쪽이라는 주장이 실제 PDF/MCP/테스트와 일치한다. README도 synthetic fixture를 한글 정답지가 아니라 rhwp None 표 선언높이 신뢰 시맨틱 핀으로 분리해 설명한다.

Blocking finding 없음. merge 후보로 판단한다.
