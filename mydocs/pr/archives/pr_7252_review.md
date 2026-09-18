---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-09-18
---

# PR #7252 검토

## 최종 판정

**머지 보류** — 2026-09-18 통합 검토.

[P1] 통합 body_overflow 16 partition 중 1개 실패: issue1891_external_bindata_link.hwpx의 상한 6 < 실측 7. 동일 환경에서 최신 devel 제품과 통합본 모두 7이므로 현재 증거로 통합 코드 회귀라고 할 수 없다. 구 base 236a601da 실측을 최신 base에 그대로 적용한 값이 맞지 않는다.

최신 base·동일 환경 실측으로 해당 행의 근거를 갱신하고, 변화 원인 또는 환경 차이를 기록한 후 래칫을 다시 통과시킨다. 실패를 숨기려고 전체 상한을 넓히지 않는다.

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
| 검증 제품 코드 | `66015f64ba89618d03ce9e5ea9774a9e54860c4f` |
| 형식·주석·PDF 보존 | `30b9cca953848cb03dd16fcff6c7a548007ce623` — 실행 의미 변경 없음 |

| 적용 source commit | 로컬 commit |
| --- | --- |
| `8fec62e01449b2b280d739665f31ee3aaa63b989` | `9327d87122ed766eb8a65d7fe68875ed06cac145` |

원 head CI는 성공/skip/neutral 상태이며 통합 head CI를 대신하지 않는다. 재조회에서 원 head가 동일함을 확인했다. [CI 1](https://github.com/edwardkim/rhwp/actions/runs/35290576814) · [CI 2](https://github.com/edwardkim/rhwp/actions/runs/35290576707) · [CI 3](https://github.com/edwardkim/rhwp/actions/runs/35290576409) · [CI 4](https://github.com/edwardkim/rhwp/actions/runs/35290576703) · [CI 5](https://github.com/edwardkim/rhwp/actions/runs/35290576670)

## 범위·조판 계약 검토

관련 이슈: [#6976](https://github.com/edwardkim/rhwp/issues/6976). body_overflow baseline 85행을 기존 실측 감소분으로 낮추고 운영 문서를 갱신한다.

이 PR은 렌더 코드가 아니라 검출 상한을 변경한다. 기존 잘못된 86712 파일의 행은 복원하지 않았다. 외부 BinData 링크 fixture는 별도 입력이며 정상 교체 문서와 혼동하지 않는다.

파일명·문서 ID에 따른 제품 분기를 추가하지 않았다. 저장 정보/재조판·음성 대조·최종 paint 적용 범위의 미검증은 위 판정에 명시했다. 분할·이어받기가 범위에 없는 PR에는 해당 체크를 적용하지 않았다.

## 실행한 검증과 한계

- 통합 제품의 `body_overflow_baseline`: **16 tests run: 15 passed, 1 failed, 210 skipped**.
- Native CLI build, fresh WASM build, 수정 후 fmt: 통과. Studio TypeScript 및 renderer 단위 검사 64개 통과.
- 전체 기록: [공동 실행·검증·입력 원장](pr_7244_review_impl.md). 원 PR의 전체 회귀 통과는 작성자/CI 증거이고 이번 로컬 재실행으로 세지 않는다.
- 전체 nextest·Clippy 3종·Native Skia 전체 게이트는 통합 focused/시각 보류가 확인되어 아직 실행하지 않았다. 승인 PR도 통합 최종 head의 필수 gate 완료 전 merge-ready가 아니다.

이 PR 자체의 시각 변경은 비해당. 실패 문서는 CLI layout-anomaly의 overBottom > 2px를 두 제품에서 직접 집계했다. 7→7이며 전체 overflowCount와 혼동하지 않는다.

## 검증 입력 커밋 확인

**충족** — 파일로 사용한 입력/PDF는 `30b9cca953848cb03dd16fcff6c7a548007ce623`에서 실제 blob과 로컬 bytes를 대조했다. 경로·SHA-256은 [공동 입력 원장](pr_7244_review_impl.md#검증-입력-커밋-원장)에 있다. 코드가 메모리에서 생성·소비하는 문서는 별도 중복 fixture를 만들지 않았다. 기존 커밋된 HWP/HWPX/PDF를 재명명하지 않았다.

## 이슈·다음 단계

#6976 장기 개선 전체를 닫는 PR이 아니다. 원 PR/이슈의 원격 상태는 이번 검토로 변경하지 않았다. 충돌·실행 순서·후속 단계는 [공동 실행 기록](pr_7244_review_impl.md)을 따른다.
