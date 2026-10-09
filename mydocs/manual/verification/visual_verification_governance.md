---
kind: canonical
status: active
canonical: mydocs/manual/verification/visual_verification_governance.md
last_verified: 2026-10-09
---

# PR 시각 검증 거버넌스 (OVL-step)

> 근거: `mydocs/feedback/ovl-step.md` (2026-07-04 작업지시자 피드백).
> 적용 대상: 일반 PR review, collaborator-mediated review, 여러 PR 체리픽 누적 검토.

## 페이지별 TSV와 PR 재검토 필수 조건

조판·렌더링에 영향을 주는 변경은 **검증 대상 각 문서의 전체 페이지 TSV를 먼저 산출**한다.
`tolerant_content_match_percent`(2px 이웃 관용 내용 실루엣 일치율)를 페이지별로 확인하며,
**한 페이지라도 90% 미만이면 PR을 재검토한다.** 평균값이나 대표 페이지의 높은 점수로
미달 쪽을 상쇄하지 않는다. 정확히 90%는 통과한다.

`re_review_required` 상태에서는 새 PR 생성과 열린 PR의 승인·통합을 보류한다.
작성자는 원본·독립 한컴 Print PDF·overlay를 대조해 원인을 수정하고 새 head에서 다시 산출한다.
누락·측정 불가·전체 쪽수 불일치도 보류 조건이다. 모든 페이지가 90% 이상이어도
변경 영역의 수식·미세 간격을 확대 판독한 뒤 완료 여부를 판단한다.

명령·저장 위치는 [「실루엣 보조값만 빠르게 TSV 산출」](visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)을 따른다.
기여자는 전체 페이지 **Native TSV와 fresh WASM TSV 원본 모두 및 대표 PNG**를 제출한다.
검토자의 Native/fresh WASM 검증은 해당 역할의 절차를 따른다. 해결 불가능한 실제 글꼴 차이는
[정식 글꼴 예외 계약](visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의 증거와 gate 판정으로만 처리한다.

## 원칙

1. **조판 변경에는 Visual Sweep 필수.** 글꼴·줄 구성·높이·정렬·개체 배치·페이지 분할·인쇄 모양에
   영향을 주면 Native와 fresh WASM Visual Sweep 및 페이지별 TSV를 반드시 산출한다.
   renderer 파일 수정 여부가 아니라 실제 소비 경로로 판단한다. 편집 command·파서·model·serializer의
   속성/저장 정보 변경도 이 경로에 영향을 주면 적용한다. 수정 목적은 검증할 입력과 페이지를 정하는
   근거이며 실행 의무를 면제하는 근거가 아니다. 실제 조판 영향이 없는 변경만 근거를 적어 비해당으로 둔다.
2. **자동 도구는 보조, 판정은 사람.** sweep/OVR/게이트류는 후보 검출·범위 축소·무회귀
   증명용이다. **최종 시각 판정 권위는 작업지시자(확인한 한컴 편집기·동일 원문 기준 PDF)** 이며
   어떤 도구 통과도 이를 대체하지 않는다 (자기검증 ≠ 한컴 호환).
   검증 대상의 어느 한 페이지라도 2px 이웃 관용 내용 실루엣 일치율이 **90% 미만**이거나 측정 불가이면
   작성자가 자기 branch에서 PDF·overlay 원인을 재검토·수정하고 새 head로 재실행한다.
   **정확히 90%는 통과**한다. 미달·누락 상태에서는 완료 제출·승인·통합하지 않는다.
   올바른 글꼴 공급으로도 해결 불가능한 실제 글꼴 차이만 [글꼴 예외 계약](visual_sweep_guide.md#해결-불가능한-글꼴의-pr-제출-예외)의
   증거로 `font_mismatch_exception`을 받아 90% 미만이어도 PR을 제출할 수 있다. 단순 추정·미설치 글꼴,
   배치 차이·쪽수 불일치·측정 누락은 면제하지 않는다. 사용자가 승인한 메인터너 보정도
   같은 검증 기준을 충족해야 하며 작성자의 일반 제출 의무를 자동 면제하지 않는다.
3. **원인과 발동 범위가 이미 정해진 렌더링 PR**은 [visual_sweep_guide.md](visual_sweep_guide.md)를
   기본 진입점으로 사용한다. 독립 정답지와 실제 사용자-visible 실패에서 결함을 찾고 원인·범위를
   판정하는 작업은 [버그 헌팅 playbook](../bug_hunting_playbook.md)이 상위 절차이며, visual sweep은
   그 안의 후보 검출·수정 전후 무회귀 도구다.

렌더링 변경의 새 회귀 테스트 추가는 [회귀 추가 선행 조건](../pr_review/visual_fixture_evidence.md#렌더링-회귀-테스트-신규-추가의-시각-검증-선행-조건)에 따라 관련 모든 페이지·fixture·
Native/fresh WASM 출력의 최저 일치율 90% 이상을 먼저 입증한다. 미달·측정 불가이면 회귀를 추가하지
않고 실제 출력을 개선한다. 새 회귀 추가의 선행 기준은 평균값·글꼴 예외·CI 성공으로 면제하지 않으며
고정한 `upstream/devel`에서 이미 미달인 원본에 등록된 부적합한 검사·파일은 아래 절차로 이관한다.
글꼴 예외의 PR 제출 허용과 새 회귀 추가 조건은 구분한다.

기존 회귀가 실패한 경우에도 **원본 입력의 실제 출력 보정 → 독립 한컴 Print PDF와
Native/fresh WASM의 관련 모든 페이지에서 90% 이상 확인 → 필요한 검사 변경** 순서로 진행한다.
실물 배치는 절대 px 좌표·고정 px 영역으로 검사하지 않고 문단·줄·개체의 소유·순서·보존·포함·겹침
관계로 검사한다. 90% 미만·측정 불가이면 기대값 갱신을 보류하고,
[기존 기대값 재검토](../pr_review/visual_fixture_evidence.md#기존-회귀-테스트의-기대값-재검토)를 따른다.

**SHA를 고정한 `upstream/devel`에서 이미 90% 미만인 원본에 등록되어 있던 회귀 검사**가
부적합한 경우에만
[별도 이슈 이관](../pr_review/visual_fixture_evidence.md#90-미만-회귀-대상의-별도-이슈-이관)에 따라 새로운 이슈로 등록한다.
해당 회귀 검사와 HWP/HWPX/PDF를 활성 검증 경로에서 제거·이관하고 별도 이슈에서 출력 보정을
진행한다. 같은 원본의 90% 이상 및 확대 판독 확인 뒤 관계형 회귀를 복원하며, 이관 자체를
현재 PR의 시각 통과나 결함 해결로 보고하지 않는다. 같은 입력의 base/head 비교로 기존 미달을
입증하며 현재 수정으로 생긴 회귀는 구현을 고친다. base 미검증은 이관 근거가 아니다.

## 페이지별 일치율 산출과 직접 판독

기준 PDF는 [버전에 맞는 한컴 Print 출력 계약](../mcp_hwp2024Convert_usage.md#기준-pdf-인쇄-계약)을 따른다.
편집 변경은 동일하게 편집한 저장본을 Print 출력한다. 기준 출력 부족은 미검증이며 비해당이 아니다.
페이지 분할·쪽수에 영향이 있으면 전체 문서를 비교한다. 높은 점수의 페이지만 선택하거나 평균으로
미달 페이지를 상쇄하지 않는다. 다른 변경도 주장한 경로와 경계의 페이지를 빠짐없이 포함한다.

**페이지별 TSV 명령·저장 위치:** [「실루엣 보조값만 빠르게 TSV 산출」](visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)의 Native/fresh WASM 예제를 각각 실행해 검증 대상 전체 페이지를 먼저 확인한다.
Native/fresh WASM의 비교 쪽수·최저값·90% 미만/누락 쪽과 입력/빌드 출처를 기록한다.
전쪽 overlay 합성은 기본 요구가 아니다. 미달 쪽, 각주·문단 소속 같은 구조 차이 쪽과
대표 변경 경계에는 비교 PNG를 추가 생성하고 사람이 독립 PDF와 직접 판독한다.
실루엣 도구가 이진화 경계의 유사한 유색 픽셀을 대조한 경우 원값·조정 픽셀 수도 함께 검토한다.
세부 계산은 [실루엣 TSV 절차](visual_sweep_guide.md#실루엣-보조값만-빠르게-tsv-산출)를 따르며,
실제 흰 배경의 그림 누락이나 위치 차이를 숨기는 색상 보정은 허용하지 않는다.
TSV 성공 또는 `not_evaluated`는 측정 완료만 뜻한다. 평균·90% 이상 점수로 각주 수량,
쪽수·내용 누락/중복 차이를 승인하지 않으며 대표 PR 이미지 제출 의무도 유지한다.

90% 이상이어도 변경 영역·알려진 차이·사용자 지적 영역을 같은 내용·배율로 확대해 독립 Print PDF와
변경 전·후 Native/fresh WASM을 대조한다. 수식 모양·장평·기준선·첨자·화살표·장식과 관계 기호
양쪽·앞뒤 글자·보기 번호·탭·줄 사이 간격까지 확인하고 점수와 직접 판독을 별도로 기록한다.
확인된 결함은 점수 통과로 면제하지 않으며 출력 보정·재검증 후 완료·회귀 추가/변경·승인 여부를
판정한다. 모든 역할은 [90% 통과 후 확대 판독](../pr_review/visual_fixture_evidence.md#90-통과-후-수식간격의-확대-판독)을 따른다.
TSV·실행 로그·중간 JSON은 ignored `output/pr-review/<id>/`에 보존하고 Git에 커밋하지 않는다.
PR 본문에는 명령·source/build SHA·입력/PDF 해시·페이지 범위·최저값·미달/누락 쪽과 증적 출처를 기록한다.
OVR·Skia·SVG 자기 비교·구조 회귀만으로 한컴 PDF Visual Sweep을 대신하지 않는다.

## bug-hunter와 visual sweep 라우팅

두 절차는 전역 우선순위를 다투지 않는다. 시작 목적에 따라 지배 절차를 고른다.

| 시작 조건 | 지배 절차 | visual sweep의 역할 |
| --- | --- | --- |
| 기준 PDF와 rhwp가 다르다는 실제 사용자-visible 결함, 원인·수정 범위 미확정 | [bug-hunter](../bug_hunting_playbook.md) | page ranking·overlay·render tree로 재현 범위를 좁히고 수정 전후 무회귀를 남김 |
| 원인과 발동 page가 확정된 renderer/layout PR | 이 거버넌스 | 발동 페이지의 compare/OVL와 회귀 증적 생성 |
| 도구·문서·CI 전용 변경 | 변경 범위별 직접 검증 | 보통 실행하지 않음 |

따라서 sweep의 `flagged`, pixel/ink 지표는 발견의 입력일 뿐 원인 판정이나 수용 결론이 아니다.
glyph 겹침 같은 반복 차이도 bug-hunter의 정답지 provenance, source→IR→layout→paint 원인 경로와
사람 판정을 거쳐야 코드 수정 대상으로 승격한다.

한컴 PDF와 대조하는 Visual Sweep은 Native와 fresh WASM 모두 인쇄 프로필로 캡처한다.
PDF에 없는 빈 누름틀 안내문과 기타 편집 전용 표시는 출력 단계에서 제외하고, 누름틀에
입력된 실제 본문과 쪽 구성은 계속 비교한다. 세부 명령·프로필 증적은
[Visual Sweep 가이드](visual_sweep_guide.md#pdf와-같은-인쇄-프로필)를 따른다.

## 도구 매핑 — 무엇을 확인할 때 무엇을 쓰나

| 확인 대상 | 도구 | 산출물 |
|---|---|---|
| 페이지 수·overlay 차이 위치·잉크 일치율·drift/overflow/겹침 **후보 자동 검출** | `scripts/visual_sweep.py` ([가이드](visual_sweep_guide.md)) | 페이지별 후보 목록, raster overlay |
| **개체(표·그림) geometry 무회귀** (baseline 대비 이동/리사이즈) | `tools/object_visual_regression.py --no-hwp` ([매뉴얼](object_visual_regression.md)) — Linux 가능, 한컴 불필요 | `objects.tsv`, 회귀 건수(종료코드) |
| 개체 단위 rhwp↔한글 대조 (한컴 환경) | 동일 도구 full 모드 | `gallery.html` side-by-side 크롭 |
| 라운드트립 시각 기하 회귀 | `rhwp render-diff` | PASS/OVER/STRUCT 판정 |
| HWPX→HWP 변환 페이지네이션 정합 | `tools/roundtrip_fidelity_harness.py` | SAME/PI_MOVED/PAGE_DELTA |
| 직렬화 구조 보존 | `hwpx-roundtrip`/`hwp5-roundtrip` + baseline 테스트 | 하드실패 종료코드 |
| **최종 시각 판정 자료** | before/after(+정답지) 3-way + **OVL 정합 패널** | `mydocs/pr/assets/` PNG |

## PR 유형별 적용 (선택 기준 예시)

| PR 유형 | 시각 검증 |
|---|---|
| 조판·렌더링에 영향을 주는 모든 변경 | **필수** — Native/fresh WASM Visual Sweep·TSV + 대표 review/overlay 직접 판독. OVR 등은 보조 |
| parser/model/serializer/편집 command 수정 | 실제 조판 소비 경로에 영향이 있으면 위 필수 절차 적용. 구조 보존만 변경하고 조판 영향이 없다는 근거가 있으면 비해당 |
| 도구/문서/CI 전용 (src 무변경) | 시각 검증 불필요 — 도구 자체 스모크로 대체 |
| studio/확장 UI (렌더 엔진 무관) | 시각 판정 불필요 — 기능 스모크·e2e 로 대체 |
| golden SVG/baseline 갱신 포함 | 갱신 사유가 fix 별로 분리·설명돼야 하며 해당 페이지 판정 필수 |

## OVL(overlay) 정합 패널 규약

정답지(한컴 PDF 렌더)와 판정본(rhwp 렌더)을 같은 크기로 겹친 합성 이미지:

- **Visual Sweep 색상 규약**: **파랑=RHWP에만 있음, 빨강=기준 PDF에만 있음,
  주황=양쪽 잉크의 차이, 회색=일치, 흰색=배경**. standalone overlay와 review PNG에
  범례를 포함한다. 원문 글자색과 차이 소속을 구별하고, PDF에 없는 추가 개체도 파랑으로 표시한다.
- 과거 R=오라클 gray, G=B=rhwp gray 채널 합성은 RHWP만 빨강, 오라클만 청록으로
  나타났다. 이 과거 합성과 현재 Visual Sweep의 범례를 혼용하지 않는다.
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
