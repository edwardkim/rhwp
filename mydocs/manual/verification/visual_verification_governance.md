---
kind: canonical
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-07-16
---

# PR 시각 검증 거버넌스 (OVL-step)

> 근거: `mydocs/feedback/ovl-step.md` (2026-07-04 작업지시자 피드백).
> 적용 대상: 일반 PR review, collaborator-mediated review, 여러 PR 체리픽 누적 검토.

## 원칙

1. **선택적 적용 — 기계적 전수 금지.** 시각 검증은 모든 샘플 PR 에 일률 수행하는 절차가
   아니다. **PR 의 수정 목적과 검증해야 할 사용자-visible 동작**에 맞춰 수행 여부와 도구를
   선택한다.
2. **자동 도구는 보조, 판정은 사람.** sweep/OVR/게이트류는 후보 검출·범위 축소·무회귀
   증명용이다. **최종 시각 판정 권위는 작업지시자(한컴 2020/2022 편집기·PDF 정답지)** 이며
   어떤 도구 통과도 이를 대체하지 않는다 (자기검증 ≠ 한컴 호환).
3. **렌더링 결과 확인이 필요한 PR 은 [visual_sweep_guide.md](visual_sweep_guide.md) 를
   기본 진입점**으로 사용한다.

## 도구 매핑 — 무엇을 확인할 때 무엇을 쓰나

| 확인 대상 | 도구 | 산출물 |
|---|---|---|
| 페이지 수·overlay 차이 위치·잉크 일치율·drift/overflow/겹침 **후보 자동 검출** | `scripts/task1274_visual_sweep.py` ([가이드](visual_sweep_guide.md)) | 페이지별 후보 목록, raster overlay |
| **개체(표·그림) geometry 무회귀** (baseline 대비 이동/리사이즈) | `tools/object_visual_regression.py --no-hwp` ([매뉴얼](object_visual_regression.md)) — Linux 가능, 한컴 불필요 | `objects.tsv`, 회귀 건수(종료코드) |
| 개체 단위 rhwp↔한글 대조 (한컴 환경) | 동일 도구 full 모드 | `gallery.html` side-by-side 크롭 |
| 라운드트립 시각 기하 회귀 | `rhwp render-diff` | PASS/OVER/STRUCT 판정 |
| HWPX→HWP 변환 페이지네이션 정합 | `tools/roundtrip_fidelity_harness.py` | SAME/PI_MOVED/PAGE_DELTA |
| 직렬화 구조 보존 | `hwpx-roundtrip`/`hwp5-roundtrip` + baseline 테스트 | 하드실패 종료코드 |
| **최종 시각 판정 자료** | before/after(+정답지) 3-way + **OVL 정합 패널** | `mydocs/pr/assets/` PNG |

## PR 유형별 적용 (선택 기준 예시)

| PR 유형 | 시각 검증 |
|---|---|
| 렌더링 코어(layout/typeset/renderer) 수정 | **필수** — OVR 무회귀 + 발동 샘플 before/after/OVL + 작업지시자 판정 |
| 직렬화(serializer) 수정 | roundtrip 게이트 + (시각 영향 시) render-diff. 판정은 페이지·기하 변화 있을 때만 |
| 도구/문서/CI 전용 (src 무변경) | 시각 검증 불필요 — 도구 자체 스모크로 대체 |
| studio/확장 UI (렌더 엔진 무관) | 시각 판정 불필요 — 기능 스모크·e2e 로 대체 |
| golden SVG/baseline 갱신 포함 | 갱신 사유가 fix 별로 분리·설명돼야 하며 해당 페이지 판정 필수 |

## OVL(overlay) 정합 패널 규약

정답지(한컴 PDF 렌더)와 판정본(rhwp 렌더)을 같은 크기로 겹친 합성 이미지:

- **채널 규약**: R=오라클 gray, G=B=rhwp gray → **검정=일치 잉크, 빨강=rhwp 만,
  청록=오라클만, 흰색=배경**.
- 판정 대상 geometry(표 구조·행 높이·배치)가 프린지 없이 검정으로 겹치는지를 본다.
- 글자 주변 미세 프린지는 폰트 메트릭 차이(오픈소스 폴백 vs 한컴 폰트)로 인한 일반 현상 —
  캡션에 명시해 오해를 방지한다.

## 산출물·게시 관례

- 판정 PNG 는 `mydocs/pr/assets/pr{번호}_{주제}_review_p{페이지}[_3way|_ovl].png` 로 커밋.
- PR 코멘트에는 **커밋 SHA 고정 raw URL** 로 임베드(브랜치 URL 금지 — devel 진전 시 깨짐).
- 렌더링 PR 머지 코멘트 표준 구성: 정량 요약(테스트/게이트) + OVR 무회귀 표 +
  before/after/OVL 시각 자료 + 판정 결과.

## 관련 문서

- [visual_sweep_guide.md](visual_sweep_guide.md) — sweep 도구 사용법(진입점)
- [object_visual_regression.md](object_visual_regression.md) — OVR 하니스
- [roundtrip_fidelity_harness.md](roundtrip_fidelity_harness.md) — 변환 페이지네이션 정합
- `mydocs/feedback/ovl-step.md` — 본 거버넌스의 원 피드백
