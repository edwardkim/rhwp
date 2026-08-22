---
kind: pr-review
status: local-review-complete
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-22
---

# PR #5897 검토 - strict MS-CFB MiniCFB 메타데이터

| 항목 | 확인값 |
| --- | --- |
| PR / 작성자 | [#5897](https://github.com/edwardkim/rhwp/pull/5897) / `@Shadungi` |
| 관련 issue | closes #5892, #5893 |
| source head | `6a0c04159674046f45d29ecccf0dd8ed67d0bf9c` |
| 상태 | non-draft, `CLEAN`, source CI 완료 |
| 적용 commit | `602e0be190`, `f92595e9c0` |

## 검토

- 미사용 FAT/MiniFAT slot을 `FREESECT`, directory slot을 `NOSTREAM`으로 초기화하고, directory
  BST의 red-black color를 유효하게 만든다. 출력 CFB의 strict reader 호환성만 바꾸며 renderer/UI
  출력에는 영향이 없다.
- 합성 CFB byte 계약은 미사용 slot, tree cycle, red-red, black-height를 독립적으로 검사한다.
  `mini_cfb_strict_contract` 2건을 통과했고, 통합 전체 nextest와 clippy도 통과했다.
- source CI의 archive A/B/C, Lint, CodeQL, Adapter, Proptest가 모두 완료했다. 범위 밖 frontend와
  WASM skip은 정상 경로 분류다.

## 판정

**통합 후보 수용.** 코드 검토와 결정적 byte-level 회귀 계약에서 차단 결함을 발견하지 못했다. 시각
검증은 컨테이너 직렬화 변경으로 요구하지 않는다.
