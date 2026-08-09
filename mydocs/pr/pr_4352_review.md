---
kind: pr_review
status: active
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-10
---

# PR #4352 검토 — Kitesurf 상태 수명과 W1 지속성 경계

## 라우팅

base route: `maintainer_general.md`

modifiers: `intake_and_review.md`, `local_validation.md`,
`multi_pr_update_branch.md`, `review_only_fast_pass.md`

loaded documents: `pr_review_workflow.md`, `pr_review/README.md`,
`pr_review/maintainer_general.md`, `pr_review/intake_and_review.md`,
`pr_review/local_validation.md`, `pr_review/multi_pr_update_branch.md`,
`pr_review/review_only_fast_pass.md`

## 메타데이터와 적용 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4352](https://github.com/edwardkim/rhwp/pull/4352) / @kevin9327 |
| base | `devel` |
| 원 PR head | `440b4a472dc58f0d7c1c9af0525e520c321003c3` |
| 기준 devel | `e48fe86947fbf9a44b1b98c7037150751af541ab` |
| 가시성 브랜치 | `review/kevin9327-20260810-pr4352` |
| 원 변경 규모 | 조사 문서 1파일, `+70/-0`, contributor 커밋 1개 |

원 변경은 Kitesurf 동향을 W1 `rhwp workspace` 제안에 사상한다. 메인터너 보정은
해당 조사 문서와 이 review·구현 기록만 바꾸며 source, test, workflow, fixture,
baseline에는 영향이 없다. contributor commit은 amend, rebase, squash하지 않고 원
head 뒤에 single-parent 문서 commit만 추가한다.

## 발견한 차단 결함

원 문서는 Kitesurf를 V8 isolate에 "상주"하는 런타임으로 설명하고 자원 절감이 상주
workspace의 비용 근거인 것처럼 연결했다. 그러나
[Cloudflare 공식 기술 글](https://blog.cloudflare.com/kitesurf/)의 상태 모델은 한
task의 수명에 묶인 ephemeral 세션이며 가능한 한 stateless다. 장기 인증 상태를
유지하는 persistent browsing session은 이 모델의 대상이 아니다.

성능 서술도 교환비를 빠뜨렸다. Cloudflare의 Chrome 대비 스크린샷/HTML 추출
측정은 CPU 3.1배/3.8배, 메모리 4.7배/7.0배 절감을 보고하지만 wall time은
1.8배/1.7배 느리다. 따라서 자원 절감을 지연 단축이나 persistent workspace의
검증으로 전용하면 안 된다.

## 메인터너 보정

- Kitesurf를 task-scoped ephemeral/stateless 격리로 정정하고 task 종료 시 상태가
  폐기된다는 경계를 적었다.
- CPU·메모리 절감과 wall-time 1.7~1.8배 지연을 한 측정 문맥에 함께 기록했다.
- W1을 열린 문서·안정 ID·색인·저널을 보존하는 persistent workspace로 분리했다.
  Kitesurf와 공유하는 것은 사람용 표면 제거와 구조화된 에이전트 계약이라는 원리뿐이다.
- W1에 상태 소유권, 재시작 복구, 입력 digest 변화 시 무효화 계약과 별도 벤치마크가
  필요함을 명시했다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| Cloudflare 공식 출처 대조 | task 수명의 ephemeral/stateless 세션, persistent session 한계, CPU·메모리·wall-time 수치를 대조해 반영 |
| 대상 문구 계약 검사 | 공식 URL, `ephemeral`, `stateless`, `persistent workspace`, 3.1/3.8·4.7/7.0·1.8/1.7 수치가 모두 존재하고 기존 "상주 실행" 문구가 제거됨 |
| Markdown 상대 링크 검사 | 조사 문서와 review·구현 기록의 저장소 내부 링크 통과 |
| `python scripts/check_document_metadata.py` | 통과. 문서 522개의 front matter·canonical 관계 이상 없음 |
| `git diff --check origin/pr/4352..HEAD` | 통과 |
| Cargo·시각 검증 | 생략. `mydocs` 아래 Markdown만 변경하며 실행 코드·렌더 출력 영향 없음 |

## 리스크와 권고

- 수치는 Cloudflare가 고른 두 workload의 Chrome 대비 결과다. rhwp W1의 성능
  예측치로 해석하지 않는다.
- 최신 PR head의 required checks와 mergeability는 실제 push 뒤 다시 확인해야 한다.
- 이 로컬 보정은 remote에 push하거나 GitHub 상태를 바꾸지 않았다.

**정정된 상태 모델과 성능 교환비를 유지하는 조건으로 merge 후보에 둘 수 있다.**
