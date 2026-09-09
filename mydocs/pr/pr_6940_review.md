# PR #6940 검토 기록

## 최종 판정

- **판정: 승인.** 검토 범위는 HWPX 각주/미주 numbering 토큰과 명시적으로 빈 장식 문자의 보존이다. 이 범위에서 코드 수준 차단 결함을 발견하지 못했고, 통합 head의 기존 회귀와 한컴 왕복 증적을 확인했다.
- 같은 branch의 #6938은 메인터너 보정과 추가 HWP/HWPX 직접 검증으로 단일축 폴백 차단 사유를 해제했다. 이 개별 승인이 최신 통합 head의 CI나 merge 승인을 대신하지 않는다.
- 인라인 `hp:autoNum`의 `USER_CHAR` 소실은 [#6941](https://github.com/edwardkim/rhwp/issues/6941)의 잔여다. 이번 검증에서도 원본 5개가 왕복본 0개로 바뀌었으며, #6872의 note-property 개선 범위와 분리한다.

## 대상과 적용

| 항목 | 기록 |
| --- | --- |
| 원 PR | [#6940](https://github.com/edwardkim/rhwp/pull/6940), 작성자 `planet6897` |
| base / 규모 | `devel`, 5파일, +339/-16, 2커밋 |
| 원 head | `52660ccb2c322b17b520103af06e3b5760899542` |
| 검토 기준 | `upstream/devel` `9a96eef92458112d5d7998f4b3390261c0e3c811` |
| 검토 branch | `review/planet6897-6938-6940-20260909` |
| 누적 검증 head | `71783397d478cf383be933e03d48cff0bad84f77` |
| reviewer | `jangster77` 지정 완료 |
| 마지막 원격 조회 | 2026-09-09, OPEN, head 유지, MERGEABLE / CLEAN |

#6938 뒤에 `c15df158`과 `52660ccb`를 `-x` 체리픽해 각각 `2ca81c05870b610cdd8b5812f43c13c30ab26727`, `71783397d478cf383be933e03d48cff0bad84f77`로 적용했다. 충돌은 없었다. 원격 상태는 조회 시점 참고값이며 병합 전 최신 head/CI를 다시 확인해야 한다.

- [고정 head 변경 전체](https://github.com/edwardkim/rhwp/pull/6940/changes/52660ccb2c322b17b520103af06e3b5760899542)
- [FootnoteShape의 원본 장식 문자 식별](https://github.com/edwardkim/rhwp/pull/6940/changes/52660ccb2c322b17b520103af06e3b5760899542#diff-b0ba58c8b6b3462e66d4ab3f89217fcb8ed4f4a31e6dfd27e910d07843b1d332)
- [빈 속성 파싱](https://github.com/edwardkim/rhwp/pull/6940/changes/52660ccb2c322b17b520103af06e3b5760899542#diff-62acabe111b06382b5bc9d8bf7db1bb3e461fa1269de3f5cfeb66e3415f5b4dd)
- [토큰과 접미 문자 직렬화](https://github.com/edwardkim/rhwp/pull/6940/changes/52660ccb2c322b17b520103af06e3b5760899542#diff-92f3a0dbee550022162128d076dfbb04ca9cc36795fee64d5538b73a82833816)

## 코드와 문서 검토

`ON_PAGE`/`ON_SECTION`을 출력하고, HWPX 원본에서 읽은 빈 장식 문자와 미설정 IR을 `deco_chars_from_source`로 구분한다. 미설정 IR의 `)` 기본값 계약은 유지한다. production/test source에 메인터너 보정을 추가하지 않았다.

원 PR의 comment/review/inline comment는 없었고, 관련 #6872와 #6941 본문/comment를 확인했다. 원 PR 작업 문서의 마지막 절은 잔여 원인을 템플릿의 앞 두 슬롯 치환으로 설명하지만, 최종 PR 본문과 #6941은 인라인 `AutoNumber` 형식 표가 `USER_CHAR`를 표현하지 못하는 문제로 정정하고 있다. 후속 작업은 #6941의 정정된 설명을 기준으로 해야 한다. 작업 문서의 테스트명/4,208개 수치도 최초 code commit의 기록이며 최종 source-side 정책 검사 결과는 4,205개다.

새 시험 3개는 직렬화 경로를 확인하지만 빈 속성 파싱 자체를 직접 고정하지는 않는다. 향후 작은 HWPX 입력의 parse/serialize 회귀로 보강할 수 있다. 이번 검토에서는 실물 두 문서의 전체 note-property 왕복으로 해당 경로를 직접 확인했다.

## 로컬 및 CI 결과

- Rust lint 묶음, workspace build, fmt check, manifest, source-side 테스트 정책을 모두 통과했다.
- 공통 focused 53/53과 전체 회귀 9,346/9,346을 통과했다(46 skipped, 409.746초).
- [원 head CI](https://github.com/edwardkim/rhwp/actions/runs/34326010958)의 Build & Test, lint, Native Skia와 [Canvas visual diff](https://github.com/edwardkim/rhwp/actions/runs/34326010692)가 성공했다. 이 결과는 원 head CI이며 새 통합 branch의 원격 CI를 실행한 것으로 쓰지 않는다.
- 명령과 범위는 [통합 검토 기록](pr_6938_6940_review_impl.md)을 따른다.

## 한컴 원본/왕복 대조

원본 위치는 기존 `/home/tsjang/Downloads/korea_downloads/` 코퍼스이며 [#6872](https://github.com/edwardkim/rhwp/issues/6872)가 지목한 두 문서다.

| 항목 | 156584446 | 156513948 |
| --- | --- | --- |
| 원본 상대 경로 | `통계청/156584446_2023년 2분기 제조업 국내공급동향 보도자료.hwpx` | `고용노동부/156513948_6.29 2022년 상반기 직종별사업체노동력조사 결과(노동시장조사과).hwpx` |
| 원본 SHA-256 | `0ff8f7c152842e14d40fec26510d74fc7cfcd8ef812e605c678141dbdb81c22f` | `d1d618c0a38d0efdb3348d21ec81fc400780a8c801ccbbc367c8ea08b495482d` |
| format / lastSavedWith | `hwpx` / `hancom-office-2018`, `10.0.0.12438` | `hwpx` / `hancom-office-2018`, `10.0.0.11529` |
| MCP 원본 job | `2822abbd-ec4f-46fe-957e-946d4babea39` | `088001bb-e69f-4276-9d29-203bea58e240` |
| MCP 왕복 job | `004f6dd9-7207-4272-8268-d9d45cd33ca7` | `96673057-7a55-49a3-9392-cd190976ea38` |
| PDF 페이지 원본/왕복 | 36/36 | 32/32 |
| 전체 `pdftotext -layout` | 144,391 bytes 완전 일치 | 198,258 bytes 완전 일치 |
| note-property 슬롯 | 2개 전체 속성 일치 | 8개 전체 속성 일치 |
| 대표 페이지 | 10쪽: `1)` 각주 | 13쪽: `*` 표시 |
| pixel / ink proxy | 100% / 100%, 차이 0픽셀 | 100% / 100%, 차이 0픽셀 |

원본 저장 버전 정책에 따라 네 변환 모두 MCP `--engine 2020`을 사용했다. job의 `succeeded`, download의 `success`, byte 수와 SHA-256을 확인했다. 실제 PDF metadata는 모두 Creator `Hwp 2022 0.0.0.0`, Producer `Hancom PDF 1.3.0.550`, PDF 1.6, A4 595x841 pt였다. engine bucket과 metadata를 구분한다. 원 contributor의 2024 측정은 참고 기록이며 이번 직접 검증은 위 2020 bucket의 결과다.

- 기준 A: [pr6940-156584446-source-2020.pdf](../../pdf/pr6940-156584446-source-2020.pdf), 940,153 bytes, SHA-256 `c601f5d2f4e5e071a73253f31bc18954d784d8a1faa9c636979c80d5f9c1ab5a`, SHA-1 `71b8f771cd5094ab8c1c8083ae59df94e83bd98f`.
- 기준 B: [pr6940-156513948-source-2020.pdf](../../pdf/pr6940-156513948-source-2020.pdf), 1,171,148 bytes, SHA-256 `b1d69f9ac6ac3347172f1963bc2ceb0a357e6bbc2287141ba57b8bf94d3059f2`, SHA-1 `780558003ece60ce1c9a14d10f9e854842852385`.
- 왕복 HWPX SHA-256: A `1e19ecabf5773c3d98c068942c5bde898d4cd9a8143baa0707517cbe9b67d98f`, B `4bfef79cd83ab8d5aab992d3c1634158e6f16635790da66e28026477f104d1ce`.
- 왕복 PDF SHA-256: A `da6ae0ce8f60ad1eb5aee715d5c2178fc67c22979d4a088a75a88f23b9412598`, B `b73641e735bb9df0930af69e2eebfa0f30a938feb41f5341e83c1fd6b20b5ef4`. 왕복 HWPX/PDF는 재생성 가능한 중간 파일로 `/tmp/rhwp-review-6938-6940/visual/`에 보관하며 커밋하지 않는다.
- 최종 [A 10쪽 패널](assets/pr_6938_6940_20260909/pr6940-note-a-p010-review.png)의 SHA-256은 `f46247dec2032287d91de17da1e2c38a767d2421fe498c81ed0aca12ccd22927`이다.
- 최종 [B 13쪽 패널](assets/pr_6938_6940_20260909/pr6940-note-b-p013-review.png)의 SHA-256은 `b83643a31ede4885c8b1760038659f47b737284ad61bb44fedbe01d298d6d105`이다.

대표 패널은 실제로 열어 각주 표시·본문·한글 및 도구 라벨을 확인했다. 원본과 rhwp 왕복본을 **동일 한컴 엔진으로 PDF 변환한 뒤** 비교한 것이며, rhwp 자체 SVG와 PDF가 100% 일치한다는 뜻이 아니다. 전체 68쪽은 텍스트를 대조했고 raster 판독 범위는 위 2쪽이다.

## 비교 재현 및 임시 경로

통합 바이너리의 `export-hwpx <원본> <왕복.hwpx>` 뒤 [MCP 가이드](../manual/mcp_hwp2024Convert_usage.md)의 `start -> status -> download`를 원본/왕복 양쪽에 실행했다. `pdftotext -layout` 전체 텍스트를 비교하고, `pdftoppm -f 10 -l 10 -r 96 -singlefile -png` 및 13쪽 대응 명령으로 raster를 만들었다.

[Visual Sweep 정본](../manual/verification/visual_sweep_guide.md#github-merge-comment)의 `make_overlay_page`/`make_review_panels`를 사용했다. threshold는 32였으며 기준/왕복 raster 차이 자체가 0이었다. 한컴 PDF 간 대조이므로 구조 flagged 후보 수는 해당 없음이며 0건으로 가장하지 않는다.

임시 위치는 `/tmp/rhwp-review-6938-6940/visual/notes/` 아래 `compare_010.png`, `overlay_010.png`, `review/review_010.png`와 `013` 대응 파일이다. 원시 raster, JSON, 비교 중간 파일은 최종 패널과 중복으로 커밋하지 않는다.

## Merge 후 contributor PR comment 계획

- 정본: [Visual Sweep GitHub merge comment 절](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment).
- #6872의 numbering/빈 장식 문자 범위에서 전체 68쪽 PDF 텍스트 일치, 대표 A 10쪽/B 13쪽 pixel 및 proxy 100%, 직접 판독에서 표시 보존을 확인한 사실을 기록한다. 한컴 PDF 간 대조이며 사람 판정 정확도나 rhwp 전체 렌더 fidelity의 수치가 아님을 명시한다.
- 이미지 URL 형식은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/pr_6938_6940_20260909/pr6940-note-a-p010-review.png`와 `pr6940-note-b-p013-review.png`다.
- asset이 실제 merge commit으로 `devel`에 존재하고 최신 대상 head의 merge가 완료된 뒤 `--body-file`로 게시한다. 게시 후 API에서 실제 Markdown/이미지를 확인한다.
- #6941 잔여는 OPEN 유지한다. #6872 close 여부는 이 PR의 실제 수용 이후 판단하며 현재 close나 merge 완료를 주장하지 않는다.

## 자산 정책

새 장기 증적은 기준 PDF 2개와 위 대표 PNG 2개다. 왕복 HWPX/PDF, SVG, 원시 raster, 중복 compare/overlay/contact sheet, metric/MCP JSON, 실행 로그와 파생 테스트 파일은 제외한다.
