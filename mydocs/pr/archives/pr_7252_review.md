---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7252 검토

## 최종 판정

**메인터너 보정 후 수용 가능** — 통합 로컬 필수 검증 완료. 원 PR 직접 merge가 아닌 보정 통합 head를 수용한다.

외부 BinData 링크 fixture의 상한을 최신 devel과 같은 **7**로 정정했다. 기존 devel의 15보다 엄격한 15→7 변경이며, 원 PR이 구 base에서 측정한 6과 구분한다. 나머지 감소 행과 정상 교체 86712 행 제거는 유지한다. 제품 코드 변경은 없다.

동일 입력 SHA-256 `ce9f7275b9c84e4f032c218b9b6f94cf53c24f6e91fba30be7fd55b17acee924`에서 base `18a9fa85e`와 같은 제품 코드의 보존 바이너리 및 통합본을 비교했다. `overBottom > 2px`인 7개 노드의 페이지·경로·y·높이·bodyBottom·초과량이 모두 같다. 0-based page 5/6/26/37/38/45/68, 초과량 2.147/2.147/231.907/228.920/109.360/550.227/3.573px다. 최신 base에서 이미 존재하는 결과이며 이번 통합의 렌더링 증가를 허용한 변경이 아니다. 구 base의 여섯 건과 달라진 개별 과거 코드 원인은 확정하지 않았다.

보정 후 `body_overflow_baseline` **16/16 PASS**, exit 0. 실행 제품은 기존 통합 제품이며 별도 visual 변경은 없다. 통합 전체가 준비되었다는 판정은 아니다.

## 최종 통합 검증

보정 제품 코드 `88f2f00da8412c769f34ef6bc3b72bc13402557b`. contributor 원 head는 아래 provenance에 별도 기록했다.
[공동 최종 검증](pr_7244_review_impl.md#최종-통합-검증)의 전체 nextest, Native Skia 3종,
fmt·Clippy 3종·workspace build·정책 검사, Studio 타입·단위·production build를 통과했다.
원 PR head CI를 재사용한 통과 주장과 구분한다. 해당 입력은 최종 Native/fresh WASM으로
다시 Visual Sweep했고 compare·standalone overlay·review와 남은 차이를 확인했다.
렌더 변경이 없는 진단·scaffold ID·래칫 자체에는 별도 시각 통과를 주장하지 않는다.

작업지시자의 PR 생성·CI 모니터링·merge·후속 처리 승인을 받았다. 통합 원격 최종 head의
CI·mergeability 확인은 아직 남아 있으며 완료 전 merge하지 않는다.

## Metadata·체리픽 provenance

| 항목 | 값 |
| --- | --- |
| 원 PR | [#7252](https://github.com/edwardkim/rhwp/pull/7252) — 시험: 본문 바닥 넘김 래칫을 실측으로 조인다 — 원장 1,386 → 1,016 (#6976) |
| 작성자·reviewer | planet6897 / jangster77 (검토 전 지정) |
| 원 base·head | `devel` / `8fec62e01449b2b280d739665f31ee3aaa63b989` |
| 규모 | 4 files, +291 / -91 |
| 조회 당시 mergeability | `CONFLICTING` / `DIRTY` — 참고 snapshot |
| 통합 base | `18a9fa85e955c220e5eb4d0143dc918a4de6be73` |
| 로컬 branch | `codex/planet-review-20260918` |
| 최초 검토 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `8fec62e01449b2b280d739665f31ee3aaa63b989` | `9327d87122ed766eb8a65d7fe68875ed06cac145` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35290576814) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35290576707) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35290576409) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35290576703) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35290576670)

## 범위·조판 계약 검토

관련 이슈: [#6976](https://github.com/edwardkim/rhwp/issues/6976). body_overflow baseline 85행을 기존 실측 감소분으로 낮추고 운영 문서를 갱신한다.

이 PR은 렌더 코드가 아니라 검출 상한을 변경한다. 기존 잘못된 86712 파일의 행은 복원하지 않았다. 외부 BinData 링크 fixture는 별도 입력이며 정상 교체 문서와 혼동하지 않는다.

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 최초 검토 검증과 한계

- 최초 통합 제품: **15 PASS / 1 FAIL**. 최신 base 근거로 해당 행만 보정한 뒤 **16 PASS / 0 FAIL / 210 skipped** (32.042초).
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 최초 검토 당시에는 전체 게이트가 미실행이었다. 아래 과거 기록을 최종 상태로 해석하지 않으며, 최신 결과는 최종 통합 검증 절과 공동 원장을 따른다.

이 PR 자체의 시각 변경은 비해당. 실패 문서는 CLI layout-anomaly의 overBottom > 2px를 두 제품에서 직접 집계했다. 7→7이며 전체 overflowCount와 혼동하지 않는다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 이슈·다음 단계

#6976 장기 개선 전체를 닫는 PR이 아니다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
