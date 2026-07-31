---
kind: review
status: active
canonical: mydocs/pr/archives/pr_3669_review.md
last_verified: 2026-07-31
---

# PR #3669 리뷰 기록

## 라우팅

```text
base route: collaborator self-merge
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
  pr_review/collaborator_self_merge.md, pr_review/intake_and_review.md,
  pr_review/local_validation.md, pr_review/visual_fixture_evidence.md
current head: 98d61ec7f9a6e946737460cc01633748ebb7ebed (작성 시점 참고)
```

## PR metadata

| 항목 | 값 |
| --- | --- |
| PR | [#3669](https://github.com/edwardkim/rhwp/pull/3669) |
| 작성자 | `@jangster77` |
| base / head | `devel` / `task_m100_3486` |
| head | `98d61ec7f9a6e946737460cc01633748ebb7ebed` (작성 시점 참고) |
| 변경 규모 | 4 files, +321 / -26 (source head 작성 시점 참고) |
| reviewer | `@edwardkim` 요청 완료 |
| 관련 issue | [#3486](https://github.com/edwardkim/rhwp/issues/3486). PR 본문의 `Closes #3486`은 merge 뒤 close를 요청한다. |

## 변경 범위와 판정

- HWP3 조합형 문자 표에서 private 글머리표 `0x2F67`을 `▸`로 복원한다.
- 암호 HWP3 문서 계약에만 inline 개체·표 셀의 세로 흐름과 차례 페이지 번호 호스트의 후행 행간을
  좁게 적용한다. 일반 HWP3의 Shape TAC 600 HU 계약은 보존한다.
- 실제 암호 HWP3 fixture의 p1–p2 차례 행과 p3 표·inline 도형의 `vpos`, `line_height`,
  `line_spacing` 회귀를 고정한다.

파서가 공통 IR line segment를 구성하는 경로를 바꾸며 사용자 출력의 페이지 흐름·표·행간에 영향을 주므로,
Cargo 성공만으로 판단하지 않고 기준 PDF의 24쪽 visual sweep을 함께 적용했다. 기준 fixture·PDF는 이번
PR에서 추가·교체하지 않았고 기존 추적 자산을 재현 입력으로 사용했다.

## 로컬 검증

| 검증 | 결과 |
| --- | --- |
| `cargo fmt --check` | 성공 |
| `cargo test --profile release-test --test hwp3_password_fixture` | 11 passed |
| `cargo test --profile release-test --test issue_2151_hwp3_ghost_page -- --nocapture` | 2 passed (`hwp3-sample11` 151쪽, `hwp3-sample14` 11쪽) |
| `cargo test --profile release-test --tests` | 최종 exit code 0 |
| `cargo clippy --profile release-test --all-targets -- -D warnings` | 성공 |
| `git diff --check` | 성공 |

모든 Cargo 검증은 `CARGO_INCREMENTAL=0`, `CARGO_TARGET_DIR=target/issue-3486-p3-20260731`에서 순차로
실행했다. 전체 integration test는 장시간 정상 실행으로 완료까지 대기했으며, 종료 신호가 아니라 최종
exit code 0을 확인했다.

## 시각·fixture 증적

| 자산 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/HWP3-password-123456.hwp` | 원본 암호 HWP3 fixture, 24쪽 | `db743d084efc9e08e839a5b4d978b16b8676434011776e090e4cda43e57304be` |
| `pdf/HWP3-password-123456.pdf` | 한컴 2020 기준 PDF, 24쪽 | `3ced5ad95ad30331e2756b5b34509c1ac91dfe3c72013c8e14f2556ca6bd5776` |
| `mydocs/pr/assets/pr_3669_issue_3486_visual_review_p001_p006.png` | p1–6 review contact sheet | `3bc738d9caf123ccaa1a4cda7d3797ae0c161aa40bc1a09384acd8fa7ef4ee71` |
| `mydocs/pr/assets/pr_3669_issue_3486_visual_review_p007_p012.png` | p7–12 review contact sheet | `fc700736eda2b8553a09b4a799a8a7903a5fc642aabd51d13ba241e4668c0836` |
| `mydocs/pr/assets/pr_3669_issue_3486_visual_review_p013_p018.png` | p13–18 review contact sheet | `3d6528b1c85613401a301b5cc7579af19f995d2b255839df1aa14277af0d4e2f` |
| `mydocs/pr/assets/pr_3669_issue_3486_visual_review_p019_p024.png` | p19–24 review contact sheet | `1a1277f5d129609cfc1ff2d3495e02face5f5816180e61002766e88fdec47e07` |

임시 산출 경로는 `/private/tmp/rhwp-issue-3486-p3-20260731/sweep-full-post-toc-p01-06`부터
`...p19-24`까지다. 각 6쪽 batch가 SVG/render tree 24쪽 전체와 지정한 raster·overlay·review 6쪽을
완료했고, 전체 24/24쪽에서 구조 후보는 없었다.

| 페이지 | 요청/완료 | 구조 후보 | 평균 pixel match | 평균 ink proxy |
| --- | --- | --- | ---: | ---: |
| 1–6 | 6/6 | 없음 | 93.12691% | 23.11761% |
| 7–12 | 6/6 | 없음 | 93.12251% | 9.92014% |
| 13–18 | 6/6 | 없음 | 92.95761% | 11.34811% |
| 19–24 | 6/6 | 없음 | 93.70045% | 10.52678% |

사람 검토에서 p1–p2 차례 제목·행·leader·쪽 번호의 세로 기준선이 기준 PDF와 맞고, p3의 글머리표·표·본문
흐름이 유지됨을 확인했다. 나머지 쪽에서도 페이지 이탈, 표/텍스트 겹침, 문단 흐름 붕괴를 발견하지 못했다.
ink proxy는 글꼴 raster 차이를 포함하므로 단독 합격 기준으로 쓰지 않았다. 기준 PDF의
`pdftotext -bbox-layout`은 exit 6으로 실패했으므로, PDF 질문 marker 단계는 성공으로 해석하지 않고 생략했다.

영구 증적: [p1–6](../assets/pr_3669_issue_3486_visual_review_p001_p006.png),
[p7–12](../assets/pr_3669_issue_3486_visual_review_p007_p012.png),
[p13–18](../assets/pr_3669_issue_3486_visual_review_p013_p018.png),
[p19–24](../assets/pr_3669_issue_3486_visual_review_p019_p024.png).

## CI와 권고

source head `98d61ec7f9a6e946737460cc01633748ebb7ebed`의 CI preflight·lint·test archive·Native Skia,
8개 default-feature shard와 `Build & Test` aggregate, CodeQL JavaScript/Python/Rust 분석은 모두
성공했다. 이 review·asset·오늘할일 commit은 source CI 성공 뒤 별도로 추가하므로, 최종 merge 조건은
그 최신 documentation-only head의 required fast-pass 성공과 작업지시자 merge 승인이다. 조건 충족 전
권고는 **보류**다.
