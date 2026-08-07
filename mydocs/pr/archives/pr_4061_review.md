---
kind: pr_review
status: accepted-with-maintainer-correction
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-06
---

# PR #4061 검토 - #4055 차트 B1 스파이크

## 대상과 변경 경계

| 항목 | 값 |
| --- | --- |
| 원 PR / 작성자 | [#4061](https://github.com/edwardkim/rhwp/pull/4061) / @johndoekim |
| contributor 원 head | `d03425e893bf81caa78e800a4070acea97e2ba9c` |
| 검토 시작 기준 `devel` | `8d123977a8c0b3dc7d5395e76f71f510b10d5e05` |
| 가시성 검토 브랜치 | `review/johndoekim-4061-20260806` |
| 메인터너 보정 head | `97c8d9732` |
| 원격 PR head | 작성 시점에는 contributor 원 head `d03425e893...`이며, 보정 head는 아직 원격에 push하지 않았다 |

PR은 #4055 B1 본구현이 아니라 차트 표현(OOXML, 레거시 `Contents`, EMF)과 중첩 CFB 재포장의
실현성을 확인하는 프로브다. 프로덕션 `src/` 변경은 없다. contributor 원 commit은 변경하지 않았고,
`maintainerCanModify=true`를 확인한 뒤 같은 가시성 브랜치 위에 메인터너 보정 commit만 추가했다.

## 최초 검토와 보정

최초 검토에서 다음 네 항목을 발견했다.

1. Stage 4의 수동 한컴 판정 범위가 X-A/H-A로만 기록됐는데 최종 보고서는 8개 변종과 2개 대조군 전수를
   판정했다고 서술했다.
2. Stage 2가 CFB 재포장을 무손실로 기록했지만 최종 보고서는 `mini_cfb`가 루트 CLSID를 잃는다고 결론냈다.
3. H-A 누락으로 manifest가 9개 파일이라고 기록됐고, 최종 테스트 수도 현행 실행 결과와 달랐다.
4. 신규 Rust 테스트가 1,229줄이라 저장소의 신규 코드 파일 1,000줄 이하 규칙을 넘었다.

메인터너 보정 `97c8d9732`에서 HWP 2020 MCP 전수 재현을 수행해 문서·보고서·판정표를 같은 결과로
정렬했다. 프로브 공통 CFB/변종 helper는 `tests/support/issue_4055_chart_probe.rs`로 분리해 본문은
828줄, helper는 417줄이 됐다. 두 파일 모두 1,000줄 이하이며 테스트 동작은 유지했다.

## 한컴 2020 독립 재현

생성기에서 만든 HWPX 4개, HWP 4개, 대조군 2개를 HWP 2020 MCP로 PDF 변환했다. 10건 모두
`status=success`, `run_status=0`, `validation=ok`, 1페이지 A4를 반환했다. PDF는
[`pdf/issue_4055_b1_spike/`](../../../pdf/issue_4055_b1_spike/)에 보존했다.

144 DPI 첫 페이지 렌더 SHA-256은 다음 두 그룹으로 정확히 분리됐다.

```text
반영   d2effc5d35f5b0ebc5906ed89cb3faf708da6d38d7041df902bd7e569c8c9811
       X-A · X-C · X-D · H-A · H-C · H-D
미반영 6ff074f9e35ef2c67eebee4c8d9cee56e53a7ba527b3403833a567f9fabf3c67
       00-control ×2 · X-B · H-B
```

대표 이미지 [`대조군`](../assets/pr_4061_hancom2020_control.png)과
[`OOXML 변경본`](../assets/pr_4061_hancom2020_ooxml.png)을 보존했다. 대조군은 y축 최대 6의
정상 막대이고 OOXML 변경본은 y축 최대 100, 첫 막대 91.7로 렌더되어 수동 한컴 판정과 일치했다.
따라서 이 코퍼스에서는 OOXML 변경만 반영되고 레거시 `Contents` 단독 변경은 반영되지 않으며,
EMF 제거본도 정상 렌더된다는 결론을 뒷받침한다.

## 로컬 검증

| 검증 | 실행 결과 |
| --- | --- |
| focused 프로브 | `CARGO_TARGET_DIR=target/review-johndoekim-4061-20260806 CARGO_INCREMENTAL=0 cargo test --profile release-test --test issue_4055_b1_chart_edit_probe -- --nocapture`를 실행해 9 passed, 1 ignored를 확인했다. |
| 변종 생성기 | 같은 명령에 `-- --ignored --nocapture`를 실행해 1 passed를 확인했다. HWPX 4개, HWP 4개, 대조군 2개가 생성·검증됐다. |
| 포맷 | `cargo fmt --all --check`를 실행해 통과했다. |
| 정적 검사 | `CARGO_TARGET_DIR=target/review-johndoekim-4061-20260806 CARGO_INCREMENTAL=0 cargo clippy --profile release-test --test issue_4055_b1_chart_edit_probe -- -D warnings`를 실행해 통과했다. |
| 공백 오류 | `git diff --check`를 실행해 통과했다. |
| 기준선 병합 | 최신 `upstream/devel` 위 no-commit merge simulation을 실행했고 충돌 및 whitespace 오류가 없었다. simulation branch는 제거했다. |

## 수용 판단과 남은 조건

**메인터너 보정 포함 수용 권고.** 최초 네 문제는 `97c8d9732`에서 해결됐다. 전수 한컴 2020 MCP
변환, PDF/렌더 증적, 현행 manifest·테스트 수, CLSID 손실의 정확한 범위, 파일 크기 제한이 서로
일치한다.

아직 원격에는 contributor 원 head만 있으므로 다음은 완료된 검증이 아니라 merge 전 외부 조건이다.

1. 작업지시자의 push 승인 뒤 원격 PR head와 `git ls-remote` SHA를 다시 대조하고 LFS 대상 여부를 판독한다.
2. `97c8d9732`와 이 archive review·오늘 기록 commit을 contributor `task4055` branch에 순서대로 push한다.
3. 코드·테스트 변경이 있으므로 최신 보정 head의 전체 GitHub Actions와 mergeable 상태를 확인한다.
4. 작업지시자 승인 뒤 merge와 후속 정리를 수행한다.

상세 근거: [계획](../../plans/task_m100_4055.md),
[Stage 2](../../working/task_m100_4055_stage2.md),
[Stage 4](../../working/task_m100_4055_stage4.md),
[최종 보고](../../report/task_m100_4055_report.md).
