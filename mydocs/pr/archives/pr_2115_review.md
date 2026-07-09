# PR #2115 Review

## 메타

- PR: https://github.com/edwardkim/rhwp/pull/2115
- 작성자: `planet6897`
- base: `devel`
- 제목: `Issue #2006: 1790387 검증용 파일 보강 — 원본 HWPX + 한글2022 권위 PDF(LFS) + 141쪽 핀`
- 문서 작성 시점 참고값: `MERGEABLE` / `BEHIND`
- reviewer assign: `jangster77` 완료

## 변경 범위

- `samples/issue2006/1790387_prep_final_report.hwpx` 추가
- `pdf-large/issue2006/1790387_prep_final_report-2022.pdf` 추가
- `samples/issue2006/README.md` 추가
- `tests/issue_2006_1790387_prep_pagination_pin.rs` 추가

이 PR은 코드 변경 없이 #2006 / PR #2082의 핵심 검증 대상 원본과 한글 2022 기준 PDF, 현재 도달값 핀 테스트를 보강한다.

## 검증 결과

- conflict 제외 재확인: #2115는 GitHub 기준 `MERGEABLE/BEHIND`, 로컬 `devel` 위 최종 delta cherry-pick 성공.
- LFS 확인:
  - `.gitattributes`가 `pdf-large/**/*.pdf filter=lfs diff=lfs merge=lfs -text`를 지정한다.
  - pointer oid: `226cd9b10e41394da09d96ce09eaa50f1b6c919952cecc9af87f2f18d6ce22d7`
  - pointer size: `50228784`
  - `git lfs install --local && git lfs pull --include='pdf-large/issue2006/1790387_prep_final_report-2022.pdf'` 후 실물 PDF 확인.
- PDF 확인: `pdfinfo pdf-large/issue2006/1790387_prep_final_report-2022.pdf`
  - Creator: `Hwp 2022 12.0.0.4547`
  - Producer: `Hancom PDF 1.3.0.550`
  - Pages: `146`
  - Page size: `A4`
  - File size: `50228784 bytes`
- 샘플 SHA-256:
  - `samples/issue2006/1790387_prep_final_report.hwpx`
  - `c68baed24096386f9041930d24d39409b61ac99463bf04dfd242440dfdeb739f`
- 로컬 테스트:
  - `CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_2006_1790387_prep_pagination_pin` 통과
- 누적 검증:
  - #2114/#2115/#2116/#2117 체리픽 누적 브랜치에서 추가 핀 테스트 4개 모두 통과
  - `git diff --check upstream/devel..HEAD` 통과

## 판단

PR 설명의 핵심 주장, 즉 한글 2022 기준 PDF 146쪽과 rhwp 현재 도달값 141쪽 핀이 실제 LFS PDF/테스트와 일치한다. 50MB 이상 PDF를 `pdf-large/` LFS 영역에 둔 것도 저장소 정책과 맞다.

Blocking finding 없음. merge 후보로 판단한다.
