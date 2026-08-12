---
kind: pr-review
status: local-ci-complete-visual-fidelity-pending
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-11
---

# PR #4520 리뷰 - 앵커 줄 계상·인라인 표 흐름 보정

| 항목 | 검토 기록 |
| --- | --- |
| 원 PR | [#4520](https://github.com/edwardkim/rhwp/pull/4520) · @planet6897 |
| base / 원 head | `devel` / `d6c5ac1cdb092547c088a50b0cffce5cef3101af` |
| 규모 | 13 files, 렌더러·HWP3 lineage·재현 fixture·회귀 test |
| 작성 시점 상태 | OPEN, `MERGEABLE`, `CLEAN` (merge 직전 재확인 필요) |

## 범위와 메인터너 보정

글앞/글자처럼 표와 빈 Shape 앵커 문단이 저장 line-seg의 흐름을 예약하도록 보정하고, 재래핑된
인라인 표가 저장 line-height를 줄마다 중복 상속하지 않게 한다. HWP3 출처 표식도 lineage에 연결한다.

기존 앵커 줄 계상 보정 뒤 원 PR에 `b0f82f0`이 추가됐다. 이 변경은 HWP5 네이티브 문서에서만,
절대배치 중첩표가 셀 높이를 과대 계상하는 경우 저장 vpos 사다리 끝점으로 캡한다. HWPX 계산 lineseg와
쪽을 넘는 거대 중첩표에는 적용하지 않아 기존 흐름형 표의 페이지 수 회귀를 막는다.

이후 최신 `d6c5ac1`은 호스트 문단이 셀의 마지막 문단인 변형까지 포착하도록 절대배치 판별 근거를
뒤 문단 간격에서 직전 저장 줄 끝과 호스트 vpos 사이의 예약 간격으로 교체한다. 표 높이의 85% 이상이
호스트 위에 예약되어 있고, 표가 저장 vpos 사다리 안에 있을 때만 캡을 적용한다. 첫 문단 호스트와
쪽을 넘는 거대 중첩표는 계속 제외한다. 누적 중 생긴 서식 차이는 메인터너 commit `f92d15e02`,
`7565f6820`으로 정규화했으며 기능 의미는 바꾸지 않는다.

## 검증과 시각 증적

- 최신 `d6c5ac1`을 적용한 뒤 `issue_4490_4491_anchor_flow`의 2건과
  `issue_4515_table_overlap_diag` 1건이 통과했다. 전자는 #4490 마지막 문단과 #4491 부동
  앵커의 저장 줄 예약을, 후자는 현재 누적 상태의 표 겹침 진단을 확인한다.
- 최신 누적 head에서 `cargo nextest run --cargo-profile release-test --target-dir target/pr-review
  --tests --test-threads 12 --no-fail-fast`를 완료했다. **5,707 passed, 정책 skip 35**, nextest
  summary 435.267초다. 이전 `b0f82f0` 기준 5,703건 결과는 최신 head 검증으로 사용하지 않는다.
- HWP 2020 MCP 기준 PDF와 rhwp SVG는 #4490 2/2쪽, #4491 38/38쪽으로 페이지 수가 일치했다.
  페이지 경계 owner 후보는 두 fixture 모두 0건이다.
- #4490 p2와 #4491 p9에서 표/도식 뒤 본문이 기준과 같은 쪽에 남고 겹치지 않음을 확인했다.
  #4491 p6의 table/footer 구조 후보는 기준 PDF와 직접 비교해 시각적 겹침이 없는 후보로 분류했다.
- 최신 누적 head는 #4566의 47쪽 `LAYOUT_TABLE_OVERLAP` 통합 회귀도 통과했다. 이는 #4520과
  함께 적용했을 때 최상위 표 겹침을 관측하는 경로가 render tree 실측과 일치함을 확인한다.

기준 PDF와 asset, SHA-256 및 수치는 누적 이행 기록에 보존한다. `d6c5ac1` 직전 누적 head에서 비교한
#4490 p2 / #4491 p9 / p26 / p36의 PDF pixel diff는 각각 `12.60%`, `12.97%`, `25.95%`, `19.33%`다.
특히 p26은 줄바꿈·수직 배치가 기준 PDF와 달라 이 수치를 단순 글꼴 raster 차이로 취급할 수 없다.
`d6c5ac1`은 기능 회귀와 전체 nextest를 통과했지만, 위 대표 쪽의 PDF 래스터 재비교는 아직 수행하지
않았다. 앵커 흐름의 기능 회귀와 전체 시각 fidelity는 별도 판정이다.

**최종 권고: #4520의 최신 기능 회귀와 누적 로컬 CI는 통과했다. 다만 이 PR을 한컴 PDF 전면 fidelity
수용 근거로 사용하지 않으며, 래스터 재비교와 [#3820](https://github.com/edwardkim/rhwp/issues/3820)의
후속 fidelity 과제 판단 뒤에 수용 여부를 결정한다.**
