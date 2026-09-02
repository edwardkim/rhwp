---
kind: visual-sweep-record
status: local-candidate-reviewed
canonical: mydocs/manual/verification/visual_sweep_guide.md
last_verified: 2026-09-02
source_prs: [6595, 6602, 6605, 6607, 6609, 6618, 6620, 6626, 6629, 6631]
---

# jeong-sik #6595--#6631 통합 후보 visual sweep

## 범위와 provenance

후보는 `upstream/devel` `51043f5f8d0453b9bc929233de443fa60cb3df4b` 위에서 source head를 provenance-preserving cherry-pick했다. code cherry-pick head는 `9088bd705cafd004d703fcf4fa1a40002e9e3bee`이며, #6629 native HWP5 wrapper height와 single-page SVG 탐색의 메인터너 보정은 이 trailing candidate에 포함한다.

실행 파일은 `target/pr-review/release-test/rhwp`이며, `fidelity_compare.py`에 `RHWP_BIN`으로 명시했다. 기준은 각 `pdf/` canonical 한컴 PDF이고 source/ref/실행 파일 및 stable asset SHA-256은 [manifest](../assets/pr_6595_6631_jeong_sik_integration_20260902/manifest.tsv)에 기록했다. endpoint, token, private font 경로는 기록하지 않는다.

공통 명령은 다음과 같다.

```bash
RHWP_BIN=target/pr-review/release-test/rhwp \
  venv/bin/python tools/fidelity_compare/fidelity_compare.py <start> <end> \
  --source <samples-source> --reference-pdf <pdf-reference> \
  --reference-grade 'Hancom canonical PDF' --out-dir /tmp/rhwp-jeong-sik-visual-20260902
```

## direct PDF/SVG 결과

| 원 PR | fixture/pages | direct diff | 판정 범위 | stable asset |
|---|---|---:|---|---|
| #6595 | satellite release p5 | 13.27% | TAC picture bottom-caption contract | [PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6595_issue6593_p005.png) |
| #6602 | 3 float-picture fixtures p1 | 16.52%, 7.73%, 2.80% | outer-margin box and ink origin | [assets](../assets/pr_6595_6631_jeong_sik_integration_20260902/) |
| #6605 | duplicate source | - | #6595와 patch-id 동일, separate sweep 없음 | - |
| #6607 | hwp3-sample14 p1, draw-group p1 | 9.62%, 0.91% | TAC picture and group outer-margin | [assets](../assets/pr_6595_6631_jeong_sik_integration_20260902/) |
| #6609 | pic-in-head-02 p1, p6 | 17.51%, 14.09% | header-frame origin | [assets](../assets/pr_6595_6631_jeong_sik_integration_20260902/) |
| #6618 | hwp3-sample14 p1 | 9.62% | PDF nested-SVG bitmap path | [PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6618_issue6612_p001.png) |
| #6620 | bitmap p1 | 0.12% | inverted-window WMF ink and placement | [PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6620_issue6617_p001.png) |
| #6626 | 5 OOXML chart kinds p1 | 1.08--1.84% | title, plot, axes, grids, legend | [assets](../assets/pr_6595_6631_jeong_sik_integration_20260902/) |
| #6629 | exam_social p1 | 23.84% | wrapper padding and declared-height scope | [PNG](../assets/pr_6595_6631_jeong_sik_integration_20260902/review_6629_issue6621_exam_social_p001.png) |
| #6631 | exam_eng p2, exam_kor p14 | 17.63%, 18.45% | stored-vpos vertical alignment | [assets](../assets/pr_6595_6631_jeong_sik_integration_20260902/) |

`diff%`는 정본 합격 임계값이 아니다. 글꼴, 텍스트 shaping, 기존 pagination 차이를 모두 포함하므로 각 issue가 주장하는 picture/caption, margin, frame, WMF, chart, wrapper, vpos 범위를 focused 회귀와 stable PNG에서 함께 판정했다. 특히 #6629와 #6631의 높은 전체 수치는 대상 좌표 외 텍스트 차이를 포함하며, 이 기록은 전체 문서 fidelity 완료를 주장하지 않는다.

## 비교기 보정

한 페이지 문서는 `rhwp export-svg`가 `<document>.svg`를 생성하지만, 기존 비교기는 `*_001.svg`만 탐색해 WMF·차트·draw-group을 `rhwp SVG 없음`으로 오판했다. 현재 후보는 1쪽에서만 무접미 SVG를 허용하고 multi-page suffix 규칙을 우선하는 helper와 Python 회귀 3건을 추가했다. 수정 뒤 #6607/#6620/#6626은 위 direct PDF/SVG 비교를 모두 완료했다.

## 최종 판정

- #6595, #6602, #6607, #6609, #6618, #6620, #6626, #6631: 승인.
- #6605: 머지 보류. #6595와 중복이라 통합 후보에서 별도 적용하지 않는다.
- #6629: 메인터너 보정 됨 수용 가능. native HWP5 wrapper 선언 높이의 CI 반례를 scoped 보정과 회귀로 해결했다.

이 기록은 local candidate 검토 기록이다. source PR comment/close, contributor branch 처리, integration PR 생성·push·merge는 작업지시자의 별도 승인 뒤에만 수행한다.
