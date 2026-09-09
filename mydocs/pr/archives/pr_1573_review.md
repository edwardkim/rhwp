# PR #1573 검토 보고서 — HWPX 패키지 그래프 + 렌더 fidelity 묶음 (리뷰 전용)

- PR: https://github.com/edwardkim/rhwp/pull/1573
- 제목: `fix: restore HWPX package graph and render-fidelity proof chain`
- 작성자: humdrum00001010 (9건/3 merged — Skia/렌더 fidelity 계열)
- 연결: #1570(자진 close)·#1543·#1546 의 재구성·복원
- base ← head: `devel` ← `humdrum00001010:gather/hwpx-render-fidelity`
- 상태: CONFLICTING / DIRTY
- 검토일: 2026-06-29 (**리뷰 전용 — 머지/수정 없음**)

## 1. 요약 판단 — 묶음 분해 권고

방향성과 근거(Ghidra/Frida 관찰 + 테스트 동반)는 진지하나, **현 형태로는 수용 곤란**.
44 files +4362/-414 / 18 커밋에 **8개 이상의 독립 fix 가 하나로 묶여** 있어 (a) 시각 회귀
판정 단위 분리 불가, (b) 문제 발생 시 이분 추적 불가, (c) 리뷰·롤백 단위 부재. 한컴 호환
가드는 케이스별 분리가 안전하다(`feedback_hancom_compat_specific_over_general`).

**권고: 독립 fix 단위로 PR 분해 재제출.** 각 fix 를 별도 이슈/PR 로 쪼개 시각 판정·회귀
게이트를 개별 통과시킨다. #1570 을 본인이 "older pin 기준 검증" 이유로 close 한 맥락과 동일 —
큰 묶음은 head 가 움직이면 통째로 재검증 부담이 누적된다.

## 2. 변경 범위 (merge-base=bfb5eac3 오늘, 순수 신규)

PR 주제(master-page/footnote/orphan/matrix-group)는 devel 최근 커밋에 없음 → 추월 아닌 신규.
렌더 코어 전반 대수술:

| 영역 | 규모 | 묶인 fix |
|---|---|---|
| `renderer/typeset.rs` | +804 | vpos pagination fit, master-page furniture, footnote |
| `renderer/layout/shape_layout.rs` | +431 | matrix-group double-transform, paper-origin anchor |
| `document_core/queries/rendering.rs` | +333 | (광범위) |
| `renderer/layout.rs` | +317 | orphan near-blank pages |
| `renderer/web_canvas.rs` | +246 | browser-WASM master-page 배경 회귀 |
| `serializer/hwpx/{mod,package_check}.rs` | +198/+224 | 패키지 그래프 복원(container.rdf, header/footer id 보존) |
| `composer/line_breaking.rs`, `text_measurement.rs` | +109/+85 | leading space, Unicode 화살표 폭 |

독립 주제(커밋 기준): ① HWPX 패키지 그래프 복원 ② matrix-group double-transform ③ matrix-scaled
cover text ④ chapter-divider 스트로크 억제 ⑤ orphan near-blank 페이지 ⑥ footnote 예약 +
master-page furniture 정렬 ⑦ paper-origin anchor ⑧ trailing empty paragraph(footnote) ⑨ vpos
pagination fit 제약 ⑩ browser-WASM 배경 회귀 ⑪ KoPub 폰트 alias ⑫ Unicode 화살표 폭.

## 3. 충돌 / CI

- 충돌 1건: `tests/visual_roundtrip_baseline.rs` (devel 이 그 사이 baseline 변경 → content).
- CI: PR base 기준 Build&Test/CodeQL/Analyze/Canvas 전부 pass. 단 baseline 충돌 해소 후
  재검증 필요.

## 4. 시각 판정 — 필수

golden SVG 2건 변경 → **시각 출력이 바뀐다**:
- `tests/golden_svg/issue-267/ktx-toc-page.svg`
- `tests/golden_svg/issue-617/exam-kor-page5.svg`
+ `visual_roundtrip_baseline.rs` 변경. 페이지 분할·master-page·footnote 배치가 바뀌므로
작업지시자 한컴 환경 시각 판정 필수(`feedback_self_verification_not_hancom`). PR 의 1-20
비교 이미지는 PR 본문 첨부 전용(레포 미포함)이며 컨트리뷰터 측 증거다.

## 5. Ghidra/Frida 근거 평가

- 한컴 HWPX 로더의 header/footer/master-page id·idRef·manifest href 해석, 렌더러의
  paper-coordinate object-order furniture 합성을 Ghidra 로 관찰했다고 명시. 직렬화·패키지
  그래프 변경은 이 관찰에 근거.
- **평가**: 리버스 엔지니어링 근거 자체는 가치 있으나, 메인테이너가 독립 재현 불가하므로
  단독 권위로 삼지 않는다. 최종 권위는 한컴 정답지 시각 판정 + 회귀 게이트
  (`feedback_no_inference_authoritative_spec`). 테스트(layout/pagination/svg tests, issue_937)
  동반은 긍정적 — 단, 묶음이라 어떤 테스트가 어떤 fix 를 가드하는지 1:1 추적이 어렵다.

## 6. 리뷰 결론 (작업지시자 판단용)

| 항목 | 평가 |
|---|---|
| 방향성 | 진지함 (Ghidra 근거 + 테스트 동반) |
| 수용성(현 형태) | **곤란** — 8+ fix 단일 묶음, 시각 회귀 단위 분리 불가 |
| 권고 | 독립 fix 단위 분해 재제출. 우선순위 높은 것부터(예: browser-WASM 배경 회귀, 패키지 그래프) 개별 PR + 개별 시각 판정 |
| 충돌/CI | 충돌 1(baseline), CI pass(base 기준) |
| 시각 판정 | 필수 (golden SVG 2 + baseline 변경) |

리뷰 전용이므로 머지/수정/close 없음. 컨트리뷰터에게 분해 재제출 가이드 코멘트는
작업지시자 승인 후 등록.

## 7. 컨트리뷰터 룰 준수 점검 (재검토)

다른 컨트리뷰터가 지키는 룰 대비 점검:

| 룰 | 준수 | 근거 |
|---|---|---|
| 스크린샷 바이너리 레포 미커밋 | ✅ | PNG 미커밋(0건), 비교 이미지는 PR 본문 첨부(GitHub release asset)만. 본문도 명시 |
| 저작권 폰트 미배포(OFL만) | ✅ | 폰트 바이너리 0건. "proprietary font 일 수 있어 가져오지 않음" 명시(코멘트) |
| 비공개 실문서 미커밋 | ✅ | `행정업무운영 편람.hwpx` devel 부재 — 로컬 대조에만 사용(opengov 실문서 비공개 정합) |
| 픽스처 변경 명시 | ✅ | golden SVG 2건(issue-267/617)만 변경, "page pixels were not manually edited" 명시 |
| PNG 무결성 해시 | ✅ | 비교 이미지 SHA-256 기재 |
| collaborator CI 가이드 응답 | ✅ | jangster77 2회 CI 안내(포맷/테스트) → `release-test` 결과 회신 |
| **시각 판정 권위 = 한컴 정답지** | ⚠️ **위반** | 아래 |
| **묶음 분해(케이스별 가드)** | ⚠️ **위반** | 8+ fix 단일 묶음(§1) |

### ⚠️ 핵심 — 시각 판정 권위 룰 위반 (컨트리뷰터 자기고백)

`feedback_pdf_not_authoritative` / `feedback_self_verification_not_hancom` /
`feedback_no_inference_authoritative_spec` 위반. 컨트리뷰터 본인이 코멘트에서 인정:

- **"뷰어의 코드를 읽지 않고 눈대중으로 처리했습니다"** — 일부 렌더링을 권위 자료가 아닌
  육안 추정으로 결정. 룰: 렌더링 의미는 추정 금지, 권위 자료로 확정.
- **"제 한컴 뷰어가 구형이라 ... PDF와 한컴뷰어조차 일치하지 않아"** — 구형 뷰어 출력을
  근거로 사용. 룰: 한컴 뷰어 출력은 정답지 아님(편집기 직접 출력 또는 2020/2022 PDF만).
- **"구형 맥 한글뷰어의 페이지 나눔은 기준으로 보지 않고"** — 본인도 기준 부적격 인지.
- Page 9 mismatch 는 폰트 이슈로 미해결 자인, page-break semantic 도 미해결로 남김.

즉 이 PR 의 시각 fidelity 변경 중 일부는 **정답지(한컴 2020/2022 편집기 또는 PDF) 대조가
아니라 구형 뷰어 + 육안 추정**에 근거한다. 이는 v0.7.6 회귀의 origin(`feedback_v076_regression_origin`:
컨트리뷰터 PDF/추정 기반 → 회귀)과 같은 위험 패턴이다.

### 점검 결론

- 정량/형식 룰(스크린샷·폰트·실문서·해시·CI 응답)은 **모범적으로 준수**.
- 그러나 **시각 판정 권위 룰을 본인이 위반했다고 자인** — 일부 렌더 변경이 정답지 아닌
  육안/구형뷰어 근거. 묶음 분해 룰도 미준수.
- → 분해 재제출 권고에 더해, **각 시각 fidelity fix 는 작업지시자 한컴 정답지 대조로
  재검증돼야** 수용 가능. "눈대중" 근거 변경분은 정답지 확인 전 보류.

## 8. Cherry-pick 처리 가능성 검토 (머지 불가 → 선별 cherry-pick)

머지 불가 확정(§1 묶음 + §7 눈대중 근거). 대안으로 메인테이너가 검증된 fix 만 선별
cherry-pick 하는 방식을 검토했다.

### 8.1 커밋 구조 (merge-base bfb5eac3, 18 커밋)

devel merge 커밋 2개(`0dc2204c`·`d8455e37`)가 중간에 끼어 순차 cherry-pick 불가.
fix 커밋은 단일-주제(깨끗)와 거대-혼합으로 갈린다:

| 분류 | 커밋 | cherry-pick |
|---|---|---|
| 깨끗(단일주제) | `a5bb5fa3` 패키지그래프, `70807f8c` matrix double-transform, `20038f31` chapter-divider, `c8bd5357` orphan pages, `d86231b9` paper-origin, `24bb65fd` trailing-empty-para, `122f87de` master-bg, `36484ca2` vpos-fit | 일부만 독립 적용 |
| 거대-혼합 | `fb39cdcf` "ground fidelity"(**16f**), `93866056` "refine fidelity"(**20f**), `fe52bdd4`(9f), `9971ddd0`(9f) | 다주제 뭉침 — 분해 필요 |

### 8.2 독립 cherry-pick 실측 (devel 위)

| 커밋 | 결과 |
|---|---|
| `a5bb5fa3` 패키지그래프 | ✅ 충돌 없이 적용 |
| `70807f8c` matrix double-transform | ✅ 충돌 없이 적용 |
| `20038f31` chapter-divider strokes | ✅ 충돌 없이 적용 |
| `122f87de` master-bg | ✗ 선행(`fb39cdcf` 등) 의존 — 실패 |
| `36484ca2` vpos-fit | ✗ 선행 의존 — 실패 |

→ 깨끗한 앞쪽 3개는 독립 가능하나, 뒤쪽 fix 는 거대 커밋에 의존해 단독 cherry-pick 불가.

### 8.3 결정적 제약 — 검증자산과 cherry-pick 용이성의 역상관

**깨끗한 단일-주제 커밋(`a5bb5fa3`/`70807f8c`/`20038f31`/`c8bd5357`/`d86231b9`/`24bb65fd`)은
테스트를 동반하지 않는다.** 테스트·golden 은 거대-혼합 커밋(`fb39cdcf` issue_937,
`93866056` render_p22, `fe52bdd4`/`9971ddd0`)에 몰려 있다. 즉:

- cherry-pick 쉬운 커밋 = 검증 자산 없음 (회귀 가드 부재).
- 검증 자산 있는 커밋 = 거대·혼합이라 cherry-pick 불가, 분해 필요.

### 8.4 Cherry-pick 처리 권고안

원커밋 그대로의 선별 cherry-pick 은 **부분적으로만 가능하고 위험**(검증 자산 분리). 대신:

1. **메인테이너 주도 재구성(reconstruct) cherry-pick**: 커밋을 그대로 따지 말고, fix 주제별로
   메인테이너가 **diff 를 발췌해 devel 기준 단일-주제 커밋으로 재작성** + **각각 테스트/golden
   동반 + 작업지시자 한컴 정답지 시각 판정**. 컨트리뷰터 작성자 보존은 `--author` 로.
2. **우선순위(근거 강도 순)**:
   - (상) `a5bb5fa3` 패키지 그래프 복원 — Ghidra/XML id·idRef 근거 명확, 직렬화 무손실 영역,
     시각 회귀 위험 낮음. **가장 먼저 cherry-pick 후보**. 단 hwpx_roundtrip_baseline 게이트 통과 필수.
   - (상) `122f87de` browser-WASM master-bg 회귀 — zOrder 528/IN_FRONT_OF_TEXT 근거 구체,
     "본문 픽셀 복원" 목적 명확. 단 선행 의존 분리 필요.
   - (중) `70807f8c` matrix double-transform, `20038f31` chapter-divider — 구조적 fix,
     시각 판정으로 확인 가능.
   - (보류) "눈대중" 근거 변경분(page-break/일부 fidelity), 폰트 alias, leading-space —
     정답지 대조 전 cherry-pick 보류(§7).
3. **묶음 거대 커밋(`fb39cdcf`/`93866056`)은 통째 cherry-pick 금지** — 다주제 뭉침 +
   눈대중 근거 혼재. 필요한 부분만 수동 발췌.

### 8.5 Cherry-pick 결론

**"검증된 fix 만 메인테이너가 재구성 cherry-pick"이 유일하게 안전한 경로.** 원커밋 선별
cherry-pick 은 (a) 거대 커밋 의존, (b) 검증자산 역상관 때문에 불완전하다. 권고: 패키지 그래프
(`a5bb5fa3`)부터 1개씩, 메인테이너가 devel 기준으로 재작성 + 테스트 동반 + 시각 판정 →
개별 검증 통과분만 devel 반영. "눈대중" 근거분은 정답지 확인까지 제외.

## 9. 산출물

- 본 검토 보고서: `mydocs/pr/pr_1573_review.md`
