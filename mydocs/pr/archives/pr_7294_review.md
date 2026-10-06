---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-21
---

# PR #7294 검토

**검토 승인 — trailing commit 사전승인 대기.** 통합 후보 head [`4e4ac77d92790ce2f6ee69c965411b27c946552e`](https://github.com/edwardkim/rhwp/commit/4e4ac77d92790ce2f6ee69c965411b27c946552e)의 [Build & Test CI](https://github.com/edwardkim/rhwp/actions/runs/35570500455)와 [CI Impact Policy](https://github.com/edwardkim/rhwp/actions/runs/35571860540)가 성공했다. 이 기록과 증적은 아직 commit·push하지 않았다.

- 원 PR: [#7294](https://github.com/edwardkim/rhwp/pull/7294), `planet6897`, head `08ecd99c11359c143ba1b654bc330f8f094f8ef8`; 통합 적용 `e8e6b43c9`.
- 범위: #7226·#4068 판정에 사용한 한/글 PDF 네 건을 저장소 `pdf/`에 보존한다. 기존 검증 입력을 이름만 바꿔 중복 추가하지 않는다.

## 검증

추가 PDF의 Git blob과 경로를 확인했고, #7298·#7296의 실제 Visual Sweep에서 해당 정본을 사용했다. PDF 파일은 검증 기준 자료이며 renderer 동작을 바꾸지 않으므로 독립 Visual Sweep 변경은 비해당이다. PDF 버전/파일명만으로 한컴 엔진을 추정하지 않고 각 비교 문서의 `rhwp info --json` 저장 메타데이터와 `pdfinfo` Creator를 분리해 기록한다.
