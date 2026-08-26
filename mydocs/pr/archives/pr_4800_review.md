---
kind: review
status: self-review-ci-pending
pr: 4800
issue: 4739
author: edwardkim
base: devel
---

# PR #4800 검토 기록

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR | [#4800](https://github.com/edwardkim/rhwp/pull/4800) |
| 작성자 | `edwardkim` |
| 관련 이슈 | [#4739](https://github.com/edwardkim/rhwp/issues/4739) |
| head / base | `task/4739-canvas-local-font-first-layout` / `devel` |
| code candidate | `4558eb25e22986f2a1685216a92e8275628b0d73` |
| 기준 devel | `9eda49613bee1d8c63e84931cabc4dee33a455e1` |
| 변경 규모 | 22 files, +1180 / -33, 8 commits |
| 문서 작성 시점 상태 | `OPEN`, `MERGEABLE`, `BLOCKED` 참고값 — CI·CodeQL·Render Diff preflight 실행 중 |
| 검토 | collaborator 자체검토; 외부 reviewer 미지정 |

### 적용 절차

```text
base route: collaborator_self_merge.md
modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md
loaded documents: pr_review_workflow.md, pr_review/README.md,
collaborator_self_merge.md, intake_and_review.md, local_validation.md,
visual_fixture_evidence.md, docs_and_git_workflow.md,
dev_environment_guide.md, visual_verification_governance.md
current code candidate: 4558eb25e22986f2a1685216a92e8275628b0d73
```

최신 `upstream/devel` `9eda49613bee1d8c63e84931cabc4dee33a455e1` 위로 8개 작업 commit을
rebase했다. `mydocs/orders/20260815.md`의 충돌은 기존 Gym·MCP 통합 기록과 #4739
기록을 모두 보존해 해결했고, 푸시 직전 fetch에서 원격 `devel` SHA가 그대로임을
다시 확인했다. 구현과 로컬 검증이 완료된 후 draft가 아닌 Open PR로 게시했다.

## 변경 검토

저장된 local-font snapshot은 첫 Canvas2D 문서 paint 전에 준비된다. KoPub style face는
확인된 exact local full name을 체인 첫 항목으로 사용하고, `KoPub바탕체`는
monospace가 아닌 비례폭 serif로 분류한다. local-font 재감지 후에는 문서를 다시
열지 않고 backend 준비 후 현재 view를 한 번만 repaint한다.

정부상징 구형 face는 exact legacy → 확인된 ROKG successor → HWPX의 non-embedded
`한컴바탕` 대체 face → portable fallback 순으로 해소한다. Rust `DocumentInfo`가 문서
대체 쌍을 WASM·Studio에 전달하고 CanvasKit preflight는 체인의 primary face만 수집한다.
구형과 ROKG의 metric profile은 서로 합치지 않았으며, 글꼴 바이너리나 외부 검증 자산은
PR에 포함하지 않았다.

KoPub/정부상징 layout metric A/B와 전역 metric 변경은 별도 승인 대상으로
범위에서 제외했다. 이 PR은 paint와 font resolution 경계만 변경한다.

## 완료한 검증

| 검증 | 결과 |
| --- | --- |
| TypeScript focused | 초기화 순서 6/6, 문서 글꼴 상태 5/5, 글꼴 해소 9/9, local-font 14/14 통과 |
| `npx tsc --noEmit` | 통과 |
| `npm test` | 호스트 930건 중 929 통과, 1 skip, 실패 0 |
| `npm run build` | 통과; 기존 chunk-size 경고만 발생 |
| Rust focused | style resolver 29/29, renderer chain·CanvasKit primary family·`DocumentInfo` 접합면 각 1/1 통과 |
| `cargo fmt --all -- --check` | 통과 |
| native WASM | `wasm-pack build --target web --out-dir pkg --no-opt` 통과 |
| Docker WASM | WSL에 `docker` 명령이 없어 미수행; native 결과를 표준 Docker 최적화 통과로 간주하지 않음 |
| Markdown / diff | 변경 Markdown 9개 상대 링크 이상 없음, `git diff --check` 통과 |

Windows Chrome 151 CDP에서 HWP/HWPX 각 383쪽을 Canvas2D와 CanvasKit으로 확인했다.
Canvas2D에서 KoPub style face와 정부상징 해소 순서가 font setter에 반영됐고,
CanvasKit에서 local Typeface 12개, load failure 0, pending 0, 준비 후 단일 repaint를
확인했다. 메인테이너의 최종 시각 판정도 통과했다. 이 현장 검증은
`mydocs/working/task_m100_4739_stage5_validation.md`에 기록했고, PDF/글꼴 대조 근거는
`mydocs/tech/investigations/issue-4739/`에 남겼다.

## 최종 조건과 권고

코드 검토와 로컬 검증에서 blocker는 발견하지 못했다. 다만 문서 작성 시점에
CI·CodeQL·Render Diff preflight가 실행 중이므로 **현재는 merge 보류**다. 이 문서와
오늘할일이 추가된 최신 PR head의 GitHub Actions, mergeability, head SHA를 다시 확인하고
메인테이너가 merge를 별도로 승인한 뒤에만 통합한다. `Closes #4739`에 따른 이슈
종료 상태는 merge 후 post-merge 절차에서 확인한다.
