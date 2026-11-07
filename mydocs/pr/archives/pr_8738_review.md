# PR #8738 리뷰: 새 표·셀 단색 배경의 무늬 없음 저장

## 최종 판정

**승인 — 새 표·셀의 단색 배경 저장과 부분 편집 보존 범위의 검증을 충족했다.**

새 코드 회귀는 확인하지 못했다. 이번 검토의 해결 범위는 새 단색의 무늬 없음(-1), 기존 무늬 0의 저장·재열기 보존, 부분 편집 시 생략한 배경 속성 보존이다. 기존 raw0 페이지 해치 누락을 해결했다고 주장하지 않는다. 점수 100%만으로 내부 무늬의 시각 일치를 판정하지 않는다.

사용자가 ready 전환과 self-review 후 병합을 요청했다. 이 요청은 미검증을 통과로 바꾸거나 branch protection을 우회하는 근거로 사용하지 않는다. 최신 devel 비교까지 완료했고, 문서-only trailing head CI 및 최신 head/base·mergeability를 확인한 뒤 정상 ready·merge 경로를 진행한다.

## 접수와 검토 경로

- PR: [#8738](https://github.com/edwardkim/rhwp/pull/8738), 작성자 edwardkim, base devel, 작성자 self-review. 외부 reviewer를 지정하지 않았다.
- 관련 이슈: [#8733](https://github.com/edwardkim/rhwp/issues/8733). Refs이며 전체 해결 또는 자동 close를 주장하지 않는다.
- 최초 고정 base: b8dd67c5001dfb0410e9e192528676396cdd6a4e.
- 후속 원격 확인의 최신 devel: e4ffabea38062e716f6c333cc950fa31b42b4dd7. #8739 통합을 포함한 최신 base의 별도 비교를 완료했다.
- 검증한 제품·테스트 코드 head: e551fcc76e0794790e25a565f2509a4891eac6cb.
- 검토한 증적 포함 head: 436730aa9855a481d7c2eb95a6ac43d8c41e093c. 뒤의 두 commit은 합성 입력·문서·시각 증적이며 제품 코드가 같다.
- 2026-11-07 확인 당시 open/Draft, 미병합, mergeable true, mergeable_state clean이었다. 이는 최종 병합 직전 확인을 대신하지 않는다.
- base route: collaborator_self_merge.md.
- modifiers: intake_and_review.md, local_validation.md, visual_fixture_evidence.md, review_only_fast_pass.md.
- loaded documents: pr_review_workflow.md, pr_review/README.md, 위 경로 문서, review_template.md, visual_verification_governance.md, mcp_hwp2024Convert_usage.md.

## 코드 검토

- 두 대화상자의 새 무늬 없음 기본값을 0에서 -1로 고쳤다. 가져온 0은 숨김 보존 선택지로 유지하고 미리보기는 가로 무늬로 표현한다. 대화상자 재사용·대상 전환에서 배경이 없으면 -1로 초기화한다.
- 공통 create_border_fill_from_json은 새 배경에만 기본값을 적용하고 기존 BorderFill을 참조한 부분 편집에서는 생략한 배경색·무늬색·무늬 종류를 보존한다. 명시적 0과 -1을 혼동하지 않는다.
- HWP 원시 BorderFill 바이트, HWP/HWPX 재열기, 명시적 0 보존, 부분 편집을 검사한다. 수정 전 UI 9건·Rust 3건 실패와 수정 후 통과를 대조했다.
- #7606 회귀는 기존 절대 크기 기준 및 줄 간격 플래그 보존을 보강한다. [#7608](https://github.com/edwardkim/rhwp/pull/7608)이 이미 base에 포함된 근거를 확인했으며 조판 엔진을 중복 수정하지 않았다.
- Windows fixture의 URL.pathname을 fileURLToPath로 바꾼 것은 파일 경로 해석 보정이다. 제품 동작·기대값 완화가 없다.
- renderer/style_resolver.rs, renderer/web_canvas.rs, renderer/skia/fill_shader.rs에는 base 대비 변경이 없다. 기존 raw0 렌더링 차이는 저장 보존 주장과 분리한다.

## 조판 원칙과 주장 대조

| 항목 | 판정과 근거 |
| --- | --- |
| 제출 전 로컬 검증 | 충족. Rust/Studio 전체·lint·빌드 완료. 실제 Print 조건은 출력자가 확인했고 최신 e4ffabea의 14개 page/backend 비교를 완료했다. |
| 기존 이슈·해결 PR 분석 | 충족. #8733의 무늬 저장 원인과 #7606/#7608의 이미 반영된 크기·줄 간격 보정을 구분했다. |
| 구현 근거와 일반성 | 충족. 공통 BorderFill 생성기와 두 대화상자의 의미값을 수정했다. 파일명·특정 좌표·샘플 전용 분기나 출력 은폐가 없다. |
| 측정·배치 일관성 | 비해당. 줄 메트릭·좌표 계산·배치 알고리즘은 바꾸지 않는다. 저장된 배경 속성의 최종 출력은 별도로 검사한다. |
| 리베이스 기능 보존 | 비해당. 고정 base 이후 다른 PR을 병합·리베이스하지 않았다. |
| 최종 출력 소비 | 충족. Native Skia와 실제 Chrome이 수신한 fresh WASM을 각각 검사했다. 최신 base와 후보의 14개 실제 렌더 PNG가 바이트·RGB 픽셀 모두 같다. |
| 분할·이어받기, 줄 소속·점유 높이 | 비해당. 해당 구현 변경이 없고 #7606 검사는 기존 속성 비트의 보존만 강화한다. |
| 사례·증거 독립성 | 충족. 실제 한글 2020 Print 출력과 합성 저장 입력을 구분했다. 출력자의 HWPX 원본 대응 확인이 있다. 가로·세로 100%·원래 용지·전체 페이지·한 쪽씩은 출력자가 7개 모두에 대해 확인했다. UI 직접 관찰과 구분한다. |
| baseline·golden·허용치 변경 | 비해당. 정상 수치를 맞추기 위한 baseline 또는 허용치 변경은 없다. |
| 회귀 선행 조건 | 충족. 구조·원시 바이트·대화상자 계약 회귀이며 절대 px 배치 검사를 추가하지 않았다. 저장값의 시각 영향에 대한 Native/WASM 및 독립 Print 검증을 별도로 완료했다. |
| 90% 이후 확대 판독 | 충족(변경 범위). 새 단색·명시적 제거 6개는 채움·배치가 일치한다. raw0 대조군은 양쪽 페이지에서 가로 해치가 누락되므로 시각 통과로 쓰지 않는다. |
| 주장과 검증 범위 | 충족. 전체 무늬 매핑 수정, 기존 손상 문서 자동 복구, #8733 전체 해결과 전체 시각 일치를 주장하지 않는다. |

## 실행 검증

[전체 결과 보고서](../../report/task_m100_8733_report.md)와 [집계·producer 해시](../assets/issue_8733/validation-summary.json)를 함께 본다.

- Rust release-test 전체 12,767 통과, 45 skip, exit 0. #8733과 #7606 focused 각 3건 통과.
- fmt, Native/WASM/workspace all-target Clippy, workspace build, suite manifest와 unit-tier 정책 검사 통과.
- Studio 전체 2,754 통과, 2 skip, 실패 0. TypeScript noEmit 통과.
- 이번 self-review에서 설치된 Node 24.22.1로 새 UI 9건을 다시 실행해 모두 통과했다. 전체 시험을 다시 실행했다고 주장하지 않는다.
- 최종 optimized WASM은 Rust 1.93.1·wasm-pack 0.15.0·locked wrapper·wasm-opt -O로 빌드했다.
- Chrome 155에서 수신 WASM 해시를 검증한 19개 확인 항목을 통과했다. 셀/표 HWP·HWPX 저장 재열기, 가져온 0 보존, 명시적 제거, 취소·재열기, 실제 undo/redo를 포함하며 브라우저 오류는 0건이다.
- Native Skia optimized release 빌드와 각 입력의 전체 1쪽 측정·리뷰가 exit 0으로 끝났다.

## 편람 383쪽 검증의 적용 여부

비해당. 이번 제품 변경은 편집 명령의 채움 속성 생성·부분 변경 보존이며 치수, 줄 간격, 조판, 페이지 분할, renderer를 수정하지 않는다. create_border_fill_from_json은 표·셀·페이지 배경 편집 명령을 통해 소비되고, 기존 HWP/HWPX 편람을 보통 열어 렌더하는 경로를 바꾸지 않는다. #7606은 이미 구현된 크기·줄 간격 비트 보존의 assertion을 강화했을 뿐 새 조판 보정이 아니다. 너비·간격 수정을 완료했다고 주장하지 않으며, 변경된 인쇄 모양에 필요한 7쌍의 Native/fresh WASM 비교는 별도로 수행한다.

## 검증 입력과 시각 증거

- 입력 7개 및 실제 Print PDF 7개의 저장소 경로·SHA-256·원본 대응 근거는 [Print manifest](../assets/issue_8733/hancom-print-manifest.json)에 있다. 모두 검토 head 436730aa9에 포함되어 있으며 실제 검증 파일과 해시를 대조했다. 모든 PDF가 50 MiB 미만이므로 대용량 예외는 적용하지 않는다.
- 후보 Native/fresh WASM의 전체 1쪽 raw silhouette는 7개씩 모두 100.00%다. 내부 해치 누락을 이 면적 지표가 잡지 못하는 한계를 직접 판독으로 확인했다.
- [정확한 명령](../assets/issue_8733/visual-commands.md), [Native 직접 판독](../assets/issue_8733/native/inspection.json), [WASM 직접 판독](../assets/issue_8733/wasm/inspection.json).
- 대표 새 단색: [Native review](../assets/issue_8733/native/cell-new-solid-hwp-review.png), [overlay](../assets/issue_8733/native/cell-new-solid-hwp-overlay.png), [WASM review](../assets/issue_8733/wasm/cell-new-solid-hwp-review.png), [overlay](../assets/issue_8733/wasm/cell-new-solid-hwp-overlay.png).
- 남은 raw0 차이: [Native review](../assets/issue_8733/native/imported-raw-zero-hwp-review.png), [WASM review](../assets/issue_8733/wasm/imported-raw-zero-hwp-review.png). 독립 Print의 내부 가로선 2개가 양쪽 출력에서 누락된다. renderer 소스가 base와 같다는 사실만으로 base 출력의 TSV를 대체하지 않는다.
- 최초 고정 base b8dd67c5의 Native/fresh WASM 빌드와 같은 7개 입력/PDF의 전쪽 비교를 완료했다. 14개 TSV가 후보와 같고 Decimal 최대 하락폭은 0.00%p, 2%p 초과 하락은 0쪽이다. 원래 렌더 PNG 14개의 RGB 픽셀도 전부 같고 대표 PNG 18개도 후보 공개 파일과 바이트 단위로 같다. [역사적 고정 기준 비교](../assets/issue_8733/base/comparison.json)를 보존한다. 최신 e4ffabea의 별도 비교도 완료했다. [최신 기준 비교](../assets/issue_8733/latest-base/comparison.json)에서 14개 모두 raw 100.00%, 최대 하락 0.00%p, 실제 PNG 바이트·RGB 픽셀 동일을 확인했다. 최신 base의 대표 compare/overlay/review 18개도 후보와 바이트 단위로 같다.
- [Print 관찰과 제한](../assets/issue_8733/hancom-print.md): 실제 Print 경로·프로듀서는 확인했다. 가로·세로 100%, 원래 용지, 전체 페이지, 한 쪽씩은 2026-11-07 출력자 확인으로 보완했다. 과거 UI 직접 관찰 또는 PDF metadata 검증이라고 주장하지 않는다.
- 설치된 Xpdf는 -bbox-layout을 지원하지 않아 marker 추출을 생략했다. 글자가 없는 단일 셀 입력의 PDFium/PNG 비교와 직접 판독은 수행했다.

## GitHub CI와 다음 조건

동일 코드의 0352c9deee13130f374cfc8015384512ecc6256c에서 [Full CI](https://github.com/edwardkim/rhwp/actions/runs/42349923309)와 [Canvas Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42349923301)가 실제 실행되어 성공했다.

436730aa9855a481d7c2eb95a6ac43d8c41e093c의 [CI](https://github.com/edwardkim/rhwp/actions/runs/42354424312), [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/42354424311), [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42354424313), [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/42354424319), [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/42354424329)는 모두 성공했다. 증적-only 변경에 대해 CI/Render Diff의 이전 성공 재사용이 적용되었고 빌드·시험 worker가 skip되었다. 이를 실제 전체 시험 재실행으로 표현하지 않는다.

남은 원격 조건은 다음과 같다.

1. PR 본문에 검토한 대표 review/overlay 이미지를 SHA 고정 inline Markdown으로 표시한다.
2. 이 review·오늘할일 및 비교 증거를 문서-only trailing commit에 포함하고 merge-tree·링크·기존 오늘할일 보존을 검사한다.
3. 새 trailing head의 required check와 실제 재사용 여부, 최신 head/base·mergeability를 확인한다. 조건을 충족한 뒤 ready 및 정상 merge를 수행한다. 보호 규칙을 우회하지 않는다. 재사용된 CI를 최신 merge tree의 새 컴파일로 표현하지 않는다.

GitHub integration의 branch-protection 세부 조회는 403이었다. 다른 경로로 제한을 우회하지 않았으며, 이후 정상 merge API가 보호 조건을 적용하도록 한다. 아직 ready 전환·review event·merge는 수행하지 않았다.
## Merge 후 contributor PR comment 계획

이 PR은 작성자 self PR이므로 별도 외부 기여자에게 감사 또는 재작업 댓글을 보낼 대상은 없다. 병합된 경우 같은 PR에 최종 merge SHA, 확인한 CI URL, [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment), merge SHA에 고정한 대표 Native/WASM review·overlay, 저장 수정의 수용 범위와 기존 raw0 차이를 기록한다. API 재조회와 PR 화면에서 실제 이미지 표시를 확인한다. #8733은 여러 증상을 포함하므로 자동 close하지 않는다.

아직 merge 결과나 후속 댓글을 미리 완료로 기록하지 않는다. 현재 사용자 또는 다른 작업의 파일을 삭제하거나 작업공간을 덮어쓰지 않는다.
## 검토 중 최신 base 확인

최종 원격 확인에서 devel e4ffabea가 #8739 통합을 포함함을 확인했다. 두 배경 대화상자, html_table_import.rs, style_resolver.rs, web_canvas.rs, skia/fill_shader.rs는 변하지 않았으나 PageLayerTree 공유 캐시와 Native replay·culling·이미지·수식 경로가 달라 최신 base의 실제 비교를 추가했고 완료했다. 기존 green source head를 유지하고 불필요한 rebase는 하지 않는다. e4ffabea와 436730aa9의 canonical merge-tree는 exit 0, tree d7a0177701525c8a7b63342abab1964a57a36a71로 충돌 없이 계산했다. 문서 trailing commit 후 다시 검사한다.


최신 Native는 저장소 Visual Sweep의 기본 dev profile과 native-skia, 후보 Native는 release profile이다. 두 경로의 실제 PNG 7개가 바이트·RGB 픽셀 모두 같다. 최신 WASM은 optimized release, 명시적 codegen-units=16 및 wasm-opt -O를 사용했다. 후보의 정확한 codegen partition은 보존 로그에서 확인하지 못했으므로 모든 컴파일 옵션이 같다고 주장하지 않는다. 동일 원본/PDF·Print profile·Windows 글꼴·backend·compat 2022·96 DPI에서 각 실제 출력을 비교했다. [최신 재현 명령](../assets/issue_8733/latest-base/REPRODUCE.md)과 producer 해시를 연결한다.

최신 base Native는 연결 복구 뒤 기존 프로세스·로그·미완료를 확인하고 캐시 빌드 한 번으로 4분 48초, WASM은 12분 28초에 성공했다. 기존 raw0 해치 누락은 최신 base에서도 동일하며 이 PR의 저장·부분 편집 보존 수용과 구분한다.
