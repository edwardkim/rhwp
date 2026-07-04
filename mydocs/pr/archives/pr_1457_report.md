# PR #1457 처리 보고서 — 차트 C1a 라우팅 + 막대 누적

- PR: https://github.com/edwardkim/rhwp/pull/1457
- 제목: `Task #1453: 3D막대·3D원형·ofPie 차트 라우팅 + 막대 누적 보정 (C1a)`
- 작성자: johndoekim (collaborator, 누적 15 PR)
- 연결 이슈: #1453 (closes), 트래킹 #1431 Track C
- base ← head: `devel` ← `johndoekim:task1453-chart-3d-ofpie-stacking`
- 처리일: 2026-06-22

## 1. 처리 결정

**admin merge.** CI 전부 pass + 로컬 CI급 검증 통과 + 충돌 0건. 코드/테스트 품질 우수,
변경 범위가 `src/ooxml_chart/` 차트 모듈에 격리. 작업지시자 판단으로
`gh pr merge --merge --admin` 반영(devel 검토 문서 커밋으로 BEHIND 였으나 무충돌).

- merge commit: `dfc967af`
- PR head `b4f931b8` → `git branch -r --contains` 로 origin/devel 포함 확인.

## 2. 검토 결과

- **Part A 라우팅**: bar3DChart→Column(barDir 재사용), pie3DChart/ofPieChart→Pie.
  렌더러·enum 무변경, 2D 근사로 placeholder 제거. 3D 입체감·ofPie 보조플롯은 후속 C2.
- **Part B 누적**: c:grouping 막대 plot 한정 채택(line은 C1d 분리). 음수 클램프·percent
  0-나눗셈 가드 견고.
- **범위 격리**: ooxml_chart 3파일 + 통합 테스트만. HWP3 룰·공통 모듈 침범 없음.

## 3. 검증

| 항목 | 결과 |
|---|---|
| GitHub CI (Build&Test/CodeQL/Analyze rust·js·python) | 전부 pass |
| 충돌 시뮬레이션 (`merge-tree`) | 0건 |
| `cargo test --lib ooxml_chart` | 21 passed |
| `issue_1453` 통합 테스트 | 2 passed |
| 전체 `cargo test --profile release-test --tests` | lib 1902 passed / 0 failed |
| `cargo fmt --check`(수정 소스) / `cargo clippy --lib` | clean |

## 4. 시각 판정 주의 (후속)

머지 게이트는 자동 테스트다. PR 의 한컴 2022 PDF 기준 정합 서술은 참고값이며, 한컴 편집기
직접 시각 판정은 Windows 환경에서 별도 확인 권장(메모리 룰 `feedback_self_verification_not_hancom`).
잔여 스타일 4갭(제목·팔레트·범례·축)은 후속 C1c, line 누적은 C1d 로 분리됨(#1431 Track C).

## 5. 후속

- #1453: devel 이 default branch 가 아니라 `closes` 자동 동작 안 됨 → 수동 close(작업지시자 승인).
- #1456(차트 캔버스 rawSvg 첫 로드 공백): 본 PR 무관, 별도 이슈로 등록됨.

## 6. 산출물

- 검토 문서: `mydocs/pr/archives/pr_1457_review.md`
- 본 처리 보고서: `mydocs/pr/archives/pr_1457_report.md`
