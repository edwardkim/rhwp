# PR #9106 리뷰 — 분할 셀 정렬과 Print 소비 경로 보정

## 최종 판정

**승인 — #7732의 분할 병합 셀 정렬 후속 범위와 필수 검증을 완료했습니다.**
검토한 code head는 `5de0973ab13d468c0da5defcc3214398efc847d7`이며 해당 head의
CI·CodeQL·Render Diff·Proptest·Adapter가 모두 성공했습니다. 이 문서와 오늘할일의
문서 전용 trailing head는 별도 검사 후 정상 squash 병합합니다. #7445 전체 해결은 아닙니다.

## 접수 정보와 경로

| 항목 | 값 |
| --- | --- |
| PR·작성자 | [#9106](https://github.com/edwardkim/rhwp/pull/9106) / `jangster77` |
| 기본 경로 | collaborator self-merge |
| 보조 경로 | intake_and_review, local_validation, visual_fixture_evidence, review_only_fast_pass, post_merge |
| 고정 base | `67b4ff8d3a55e5bb50c46a44315de749a4ef07a7` |
| PNG·WASM 생산 소스 | `2749e491e843ec851de12467751ec200ab551014` |
| Native SVG CLI 후속 | `8472674645` — PNG와 같은 프로그램 준비 연결 |
| 최종 코드·증적 head | `5de0973ab13d468c0da5defcc3214398efc847d7` |
| 관련 이슈 | 이미 닫힌 [#7732](https://github.com/edwardkim/rhwp/issues/7732)의 후속, [#7445](https://github.com/edwardkim/rhwp/issues/7445) 참조 |
| 작성 시점 | 2026-11-16 06:32 KST, Open·mergeable, reviewer 요청 없음 |

프로젝트 정본과 `collaborator_self_merge.md`·`review_only_fast_pass.md`·`post_merge.md`의
순서를 적용했습니다. 이 PR 자체에는 CI 실행 정의·재사용 정책 변경이 없으므로 옵션 2가 아닌
동일 PR의 문서 trailing 경로입니다. 기여자 원 PR을 대체하는 통합 PR이 아닙니다.

## 변경과 조판 원칙 검토

| 변경 | 원인과 공통 소비 | 독립 근거·반례 |
| --- | --- | --- |
| 분할 병합 셀 정렬 | 배치와 paint가 수용한 cut_units·저장 프레임을 같이 소비합니다. 임의 컷의 Top 계약은 유지합니다. | 각주 37쪽 숫자31·정책실현 기준선, 내용 소유·이어받기 관계와 정상 대조군 |
| 구형 Print 표시 em | 실제 static TTF 잉크의 장치 em 투영과 논리 전진을 구분합니다. 화면·2024는 적용 대상이 아닙니다. | 독립 8/10/11/13pt 표본·공개 합성 TTF·원문 전진 계약 |
| SVG 가는 표 선 | 공통 paint가 장치 픽셀 힌트를 전달합니다. 표 내용의 위치를 임의로 이동하지 않습니다. | 독립 PDF와 선 속성 대조, [원인 기록](../assets/issue7732_split_span_20261115/svg-print-stroke/cause.md) |
| 표 바깥여백 | 비점유 앵커와 ParagraphFloatPlacement의 원점 소유를 보존합니다. 오른쪽 정렬은 가용 폭을 소비하며 TAC 계약은 유지합니다. | 한 필드만 바꾼 독립 2020 Print 3종·전 17쪽, 위140HU·오른쪽141HU의 축별 변화, 보정 전 2 FAIL→관계형 3 PASS |
| Native layer SVG | 기존 PNG의 prepare_native_otf_fonts를 페이지 열거 전 재사용합니다. legacy·Skia 미포함 빌드의 개선을 주장하지 않습니다. | 공개 합성 TTF CLI/API 1FAIL→1PASS, 실물 7쪽 94.37→100.00%, 28쪽 97.19→99.23% |
| CanvasKit 작업 예산 | 문서 사전 검사 한도를 200,000으로 늘렸습니다. 미지원 연산·글꼴 준비·fallback guard는 유지합니다. | 한도 경계 회귀와 지원되는 각주 3쪽의 실제 두 모드 출력 |

문서 ID나 페이지 번호로 출력 위치를 분기하지 않았습니다. 측정·수용 컷·프레임·최종 paint의
연결과 적용되지 않는 경로를 [검증 원장](../assets/issue7732_split_span_20261115/README.md),
[바깥여백 근거](../assets/issue7732_split_span_20261115/outer-margin-stage/README.md),
[SVG 프로그램 원인](../assets/issue7732_split_span_20261115/native-svg-source-program/cause.md)에
기록했습니다. 내용 순서·소유·누락·중복과 주변 내용까지 확인했습니다. 98% 이상·직접 판독을
충족한 완료 쪽의 허용된 px 검사와 관계형 검사를 구분했습니다. 판정 하한·PDF·쪽수를 낮추지 않았습니다.

## 검증 입력의 신원

각주 입력은 [원본 HWP](../assets/issue7445/issue1937_rowbreak_footnote_overpagination.hwp),
기준은 [한컴 2020 Print PDF](../../../pdf/issue7382-regression-review/issue1937_rowbreak_footnote_overpagination-2020.pdf)입니다.
둘 다 검토 commit에 포함되어 있으며 실제 사용 파일과 SHA를 대조했습니다.

- HWP SHA-256: `a3a075594994e4741a7fbe973bc230d95ae75fdee6578ac6017cec0d52844990`
- PDF SHA-256: `7f14bc85eb389a68cc88bd40fcafeb7a638d8454293a30bb2c8fe884a16978eb`
- PDF: 한컴오피스2020 `11.0.0.9136`의 독립 Print, 전 50쪽. 웹한글기안기 PDF를 정답으로 사용하지 않았습니다.
- 중첩 표도 원본과 대응하는 독립 2020 Print 전 17쪽을 유지했습니다. 다른 보호 문서는 manifest의 2020/2024 조건을 유지했습니다.
- 비공개 글꼴·원 윤곽 좌표·글꼴 복원용 임시 표본 PDF는 제출하지 않았습니다.

Rust 회귀는 Mac 최소 글꼴 sandbox에서, Visual Sweep은 원본 글꼴을 공급한 일반 Mac 환경에서
실행했습니다. Gulim 원본 파일 전체의 hash가 같다고 주장하지 않습니다. Native 5.03과
실제 WASM 2.21 각각의 PDF 사용 366글리프 윤곽·전진 일치는
[공급 자격 증거](../assets/issue7732_split_span_20261115/outer-margin-stage/gulim-supply-qualification.json)로 확인했습니다.

## 완료한 로컬 검증

| 검증 | 실제 결과·증거 |
| --- | --- |
| 최신 Native SVG 후속의 전체 nextest | 최소 글꼴 Mac, 13,617 PASS·0 FAIL·45 SKIP, 실제 검사 561.270초. [명령·결과](../assets/issue7732_split_span_20261115/native-svg-source-program/current-validation/results.json) |
| 새 Native SVG 회귀 | 수정 전 1 FAIL·2.596초→수정 후 1 PASS·2.439초. 기존 suite 수집 경로·native-skia feature에서 실제 실행 |
| Rust 필수 게이트 | fmt·Native/WASM Clippy·workspace build·all-target Clippy·Native Skia 범위 PASS. [최종 lint](../assets/issue7732_split_span_20261115/native-svg-source-program/current-validation/push-lints.json) |
| Studio·TypeScript | 2,798 PASS·0 FAIL·2 SKIP, TypeScript PASS |
| suite·unit 정책 | 2,374소스·48target, unit 4,094건 분류 검사 PASS. [최종 수집 기록](../assets/issue7732_split_span_20261115/final-local-acceptance.json) |
| 보호 계약 | Python 33 PASS, 전체3,542행이 수용한 원문 TSV와 정확히 일치 |
| 명시적 편람 gate | 새 Native SVG 후속 실행 파일로 Native 383쪽 PASS. 중첩 표만 상향한 새 정본으로 동일 383행을 canonical 재대조해 PASS. [gate](../assets/issue7732_split_span_20261115/handbook-current-847/gate.json) |
| 공개 4문서 | Native/fresh WASM8건 공식 두 자리100.00%, crop의 반올림 전 99.99756%. [결과](../assets/issue7732_split_span_20261115/rebase-67b4-20261116/public-four-validation.json) |

Mac WASM은 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드입니다.
Docker 최적화 통과로 보고하지 않습니다. 순차 Cargo·jobs8·threads8 및 capture 총8 이하를 유지했습니다.

## 전쪽 시각 검증·새 base 보존

canonical 96DPI·Print·2px raw 실루엣으로 전체 쪽을 확인했습니다.

| 대상·출력 | 전쪽 결과 |
| --- | --- |
| 각주 Native PNG 50 / fresh WASM PNG 50 | 각각 최저 99.29%, 누락·미달0 |
| 각주 Native SVG 50 / fresh WASM SVG 50 | 각각 최저 99.08% /99.16%, 누락·미달0 |
| 중첩 표 Native 17 / fresh WASM 17 | 각각 최저 99.86%, 보호 하락 0 |
| 정본36데이터셋·3,542쪽 | 하락 0, 상승14행. [전체 TSV 목록·hash](../assets/issue7732_split_span_20261115/final-2749-67b4/protected-validation.json) |
| 최신 고정 base67b4 대비 Native/fresh WASM134쪽 | 각 경로 각주 50+중첩 17쪽 전쪽, 모든 행 하락 0. [생산·조건·행별 차이](../assets/issue7732_split_span_20261115/current-base-67b4/validation.json) |

보호 대표36쌍과 변경 영역의 같은 쪽 review·overlay를 직접 판독했습니다. 숫자31·정책실현,
괄호 전진·글자 기준선·가는 선·표 앞뒤 문단과 다음 쪽의 소유를 확인했습니다. 공식 100.00%가
원시 색상·모든 글리프 픽셀의 동일함을 뜻하지는 않습니다.
[현재 전쪽 TSV](../assets/issue7732_split_span_20261115/native-svg-source-program/sibling-outputs/validation.json)와
[실제 SVG 전쪽](../assets/issue7732_split_span_20261115/native-svg-source-program/after-native-svg/validation.json)을 보존했습니다.

상승한 중첩 표는 Native/WASM 각 전 17쪽 데이터셋 전체를 새 기준으로 수용했습니다.
14행만 상승했고 다른 34데이터셋 출처는 유지했습니다. 행별 최댓값을 혼합하지 않았습니다.
새 정본 SHA는 `2280e97844b2cb262ea73e9c49f09f56c434097fed49af75fd9a7f6198d43da7`이며
[상향 영수증](../assets/visual-protected-baseline/updates/issue7732-20261116.json)에 이전 출처를 남겼습니다.

리베이스 유입 About UI·suite 정책·공개 보호 검사·CI 분류는 별도 실제 검증을 완료했습니다.
[입력·읽기 경로 감사](../assets/issue7732_split_span_20261115/rebase-67b4-20261116/source-read-path-reuse.json)로
변경 없는 renderer·PNG·WASM의 증거를 재사용하고, CLI 변경 후 Native SVG·회귀는 새로 검증했습니다.
과거 base TSV를 현재 base 결과로 인용하지 않았습니다.

## 실제 소비 경로와 지원 한계

Studio 8쪽 screen/print 48캡처와 별도 SVG 8캡처, 독립 PDF Print 32쌍을 확인했습니다.
모두 98% 이상이며 최저는 각주 37쪽 Canvas2D99.08%입니다.
[실제 출력·프로그램 신원·모드 행렬](../assets/issue7732_split_span_20261115/actual-current-2749/validation.json)과
[직접 판독](../assets/issue7732_split_span_20261115/actual-current-2749/direct-reading.json)을 보존했습니다.

**CanvasKit 직접 출력은 각주 3쪽의 compat/default 두 모드 12캡처입니다.** 중첩 5쪽은
기존 전쪽 preflight의 미지원 연산과 지정 글꼴 자원 미준비로 Canvas2D에 전환됐으며,
전환 계약 20건을 별도로 확인했습니다. 이 20건을 CanvasKit 직접 성공으로 세지 않았습니다.
엄격한 backend 간 픽셀 비교 reportOnly 12 FAIL도 보존했으며 canonical 2px PDF 판정과 구분했습니다.
작업 한도·지원 검사·fallback guard를 우회하지 않았습니다.

root pkg·Studio public·실제 초기화 WASM 응답의 hash가 일치했습니다. 실제 JS 응답은 Vite의
WASM URL rewrite 1곳과 inline source map을 구분해 나머지 내용의 엄격한 동일성을 확인했습니다.
초기화 성공만으로 실제 paint를 대체하지 않았습니다.

## 정확한 code head의 GitHub CI

[전체 CI 원장](../assets/issue7732_split_span_20261115/code-ci-9106.json)은 다음 5개 workflow의
완료 success와 같은 code SHA를 기록합니다.

- [CI43760923507](https://github.com/edwardkim/rhwp/actions/runs/43760923507): Lint, Frontend package, Native Skia, Rust archive A~E, Build & Test 모두 success.
- [CodeQL43760923432](https://github.com/edwardkim/rhwp/actions/runs/43760923432): success.
- [Render Diff43760923348](https://github.com/edwardkim/rhwp/actions/runs/43760923348): success.
- [Proptest43760923405](https://github.com/edwardkim/rhwp/actions/runs/43760923405): success.
- [Adapter43760923363](https://github.com/edwardkim/rhwp/actions/runs/43760923363): success.

검증 CI가 성공한 뒤 review·오늘할일만 trailing commit으로 추가합니다. trailing head의
실제 preflight·required check·정확한 head·mergeability를 다시 확인한 뒤 병합합니다.

## Merge 후 contributor PR comment 계획

자체 후속 PR이므로 원 contributor PR의 추가 close는 없습니다. 병합 후 PR #9106과 이미 닫힌
이슈 #7732에 동일 merge·증거의 댓글이 없는지 확인한 뒤 한국어 존댓말로 후속 결과를 게시합니다.

- 실제 merge SHA와 위 CI 결과, 각주 원본의 한컴 2020 Print 기준·전체50쪽 결과를 명시합니다.
- [Visual Sweep 정본](https://github.com/edwardkim/rhwp/blob/devel/mydocs/manual/verification/visual_sweep_guide.md#github-merge-comment)을 연결합니다.
- 같은 각주 37쪽 Native/fresh WASM PNG의 raw·PDF·review·overlay가 merge commit에 존재함을 확인합니다.
  이미지 URL은 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/issue7732_split_span_20261115/native-svg-source-program/sibling-outputs/<native-png-or-wasm-png>/review/review_037.png`와 대응 `overlay/overlay_037.png`입니다.
- 본문에 review·overlay를 Markdown 이미지로 표시하고, 수치는 raw 2px 자동 보조값이며 원시 픽셀 동일성을 뜻하지 않음을 적습니다. CK 직접·전환 범위도 구분합니다.
- merge SHA 고정 URL의 실제 PNG hash를 확인한 뒤 게시하고 API 본문을 재조회합니다. #7445 전체 완료나 다른 서브이슈 close를 주장하지 않습니다.
- `post_merge.md`에 따라 duration 결과·devel 동기화·소유 임시 branch/worktree/debug를 정리합니다.
  사용자 요청 글꼴 ZIP `output/visual-sweep-protected-fonts-20261115.zip`은 명시 보존 지시에 따라 유지하며 비공개로 취급합니다.
