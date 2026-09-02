---
kind: pr-review
status: accepted-with-maintainer-fix
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-08-26
---

# PR #6080 review - 휴먼명조·HY헤드라인M 전각 낫표 반각 강제 해제 (#6060)

## 접수 메타데이터

| 항목 | 값 |
| --- | --- |
| PR | [#6080](https://github.com/edwardkim/rhwp/pull/6080) |
| 작성자 | [@kevin9327](https://github.com/kevin9327) |
| base | `devel` |
| 원 head | `7f968da50cd0f70f7b1e1d1f4b0e7223db3cdf3e` |
| 규모 | +1741 / -10, 11 files, 2 commits (증적 SVG 1,590줄 포함) |
| GitHub 상태 | non-draft, `MERGEABLE/CLEAN`, Build & Test success (작성 시점 참고값) |
| 원 PR CI | [run 32882422142](https://github.com/edwardkim/rhwp/actions/runs/32882422142/job/97922884928) |
| 통합 적용 | `0184d10cf`, `dcd320fe9` + 메인터너 보정 `1dedbfdba` |
| 판정 | **보정 후 수용 후보** |

## 관련 이슈와 변경 범위

[#6060](https://github.com/edwardkim/rhwp/issues/6060)은 `is_halfwidth_cjk_quote`의 글꼴 무관 반각
강제가 반증되는 문서군이다. 한글 2020은 휴먼명조·HY헤드라인M에서 「」를 전폭(30.0pt)으로 조판하는데
rhwp는 15.0pt로 눌렀다.

변경은 폭 측정(`text_measurement.rs`)의 반각 오버레이를 고정폭 메트릭 한정으로 좁히고, 페인트
경로 두 곳(`skia/text_replay.rs`, `web_canvas.rs`)의 판정을 새 헬퍼 `forces_halfwidth_cjk_quote`로
교체한다. 폰트명 정규화에 따옴표 trim이 추가됐다.

## 렌더 영향과 시각 검증 판정

글자 폭 측정과 글리프 페인트가 함께 바뀌므로 **직접 증적 필수** 조합이다. 원 PR은
`samples/hwp3-sample16-hwp5.hwpx`
(SHA-256 `49e3e809eb41e22b2c059383db32b0cf038787269b5c523d1ff59d1a52b4340c`) 5쪽의 before/after SVG와
PNG를 첨부했다.

## 발견한 문제와 risk — 측정과 페인트의 판정 기준 발산 (차단, 보정함)

원 PR은 같은 문자에 대해 두 경로가 **서로 다른 기준**을 쓰게 만들었다.

| 경로 | 기준 |
| --- | --- |
| 폭 측정 | `is_halfwidth_cjk_quote(c) && is_monospace_metric(metric)` — 메트릭 |
| skia 페인트 | `forces_halfwidth_cjk_quote(font_family, c)` — 폰트 이름 목록 4개 |
| web_canvas 페인트 | 동일 |

메트릭 DB를 전수 조사하니 monospace이면서 `U+300C` 폭이 `em_size`인 글꼴은 11종인데, 이름 목록
(`돋움체`/`DotumChe`/`굴림체`/`GulimChe`)이 덮는 것은 `DotumChe`·`GulimChe` 둘뿐이다. 발산은
양방향이다.

1. **이름 목록 밖 고정폭** — `BatangChe`(바탕체), `GungsuhChe`(궁서체), `D2Coding` 계열 5종은
   advance만 반각으로 줄고 글리프는 전각으로 그려져 다음 글자와 겹친다. 별칭 표에
   `"바탕체" => "BatangChe"`, `"궁서체" => "GungsuhChe"`가 있어 문서의 한글 폰트명에서 그대로
   도달한다.
2. **메트릭 DB 밖인데 이름만 닮은 글꼴** — `KoPub돋움체`는 전용 폭 표를 쓰고 메트릭 DB에 없다.
   `U+300C`에서 그 표는 어느 분기에도 걸리지 않아 `None`을 반환하므로 측정은 반각 오버레이를
   적용하지 않는데, 이름에 `돋움체`가 들어 있어 페인트만 0.5×로 눌렀다.

devel에서는 두 경로가 모두 반각이라 정합했으므로 이 발산은 #6080이 새로 만든다. SVG 경로는
`svg_text_length_attrs`가 `textLength`+`lengthAdjust="spacingAndGlyphs"`로 글리프를 advance에 맞춰
누르기 때문에 발산이 가려진다 — 원 PR 증적이 SVG라 여기서는 보이지 않고, skia(PDF/PNG)와 WASM
브라우저에서만 나타난다.

보정 내용과 근거는
[통합 구현 기록](pr_6075_6077_6079_6080_6084_review_impl.md#6080--반각-판정의-측정페인트-발산-확정-결함-136a94677)에
있다. 페인트가 측정 결정을 그대로 되묻게 바꿔 판정 기준을 하나로 만들었다.

## 범위 밖 관찰

- 페인트는 `U+2018`–`U+2027`을 글꼴·글리프 폭과 무관하게 0.5×로 누르는데 측정은
  `glyph_w >= em_size`일 때만 반각으로 잡는다. devel에도 있는 선행 상태라 이번 보정 범위에
  넣지 않았다.
- 폰트명 따옴표 trim은 CSS 체인 문자열(`'맑은 고딕',sans-serif`)에서 첫 face를 제대로 뽑게 하는
  개선이다. PR 본문에 설명은 없지만 측정 경로 전반에 영향을 주는 변경이라 여기 기록한다.

## 검증 근거 (통합 head `136a94677`)

- 원 PR 회귀 `issue_6060_fullwidth_cjk_quote`와 보정이 추가한
  `issue_6060_cjk_quote_paint_measure_parity` 6건이 통합 head 전체 회귀에 포함돼 통과했다.
- 보정 전 구현으로 되돌려 같은 시험을 실행하면 회귀 2건
  (`monospace_faces_outside_name_list_match_measurement`,
  `name_lookalike_outside_metric_db_is_not_forced`)만 실패하고 #2020·#6060 앵커 2건은 통과한다.
  시험이 실제 결함을 잡고 기존 계약은 바꾸지 않음을 이 대조로 확인했다.
- 시각 검증(한컴 기준 PDF 대조): 원본 `lastSavedWith.product`가 `hancom-office-2024`인
  `samples/hwp3-sample16-hwp5.hwpx` 5쪽에서 `가)「국가를당사자…` 줄을 비교했다.

  | 대상 | 「 advance | font size | em 비 |
  | --- | --- | --- | --- |
  | 한컴 기준 PDF `pdf/hwp3-sample16-hwp5-2022.pdf` p5 | `12.00` | `12.96` | `0.926` |
  | 통합 head `rhwp export-svg -p 4` | `16.00` | `17.33` | `0.923` |

  통합 head의 낫표 advance는 같은 줄의 인접 한글(`15.95`, `0.920 em`)과 사실상 같아 전폭이다.
  반각 강제였다면 약 `0.5 em`이어야 한다.
- 메트릭 데이터 확인: `DotumChe`·`GulimChe`·`BatangChe`·`GungsuhChe`는 em_size `1024`, Latin 폭
  전부 `512`(고정폭), `U+300C` 폭 `1024`. `D2Coding`은 em_size `1000`, `U+300C` `1000`.
  `Dotum`·`Batang`·`HYHeadLine-Medium`·`휴먼명조`는 비례 글꼴이라 반각 오버레이 대상이 아니다.

## 최종 권고

**보정 후 수용.** 원 변경의 방향(휴먼명조·HY헤드라인M 전폭)은 한컴 기준 PDF로 확인했고, 측정·페인트
발산은 메인터너 보정 `136a94677`로 해소했다.
