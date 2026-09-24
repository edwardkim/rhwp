# PR #7405 리뷰 — 공통 호스트 글꼴 공급과 CanvasKit·Canvas2D

## 최종 판정

**승인.** 아래 로컬·브라우저·시각 검증으로 명시한 화면 지원 범위를 확인했다.
이는 작성자 self-review이며 GitHub approve나 merge 승인이 아니다.
merge 전 조건은 최신 PR head의 required CI 통과, mergeability 재확인과 작업지시자의 merge 승인이다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR·작성자·base | [#7405](https://github.com/edwardkim/rhwp/pull/7405) / postmelee / devel |
| 관련 이슈 | [#7403](https://github.com/edwardkim/rhwp/issues/7403), 자동 종료 없이 참조 |
| 검증 source | `eb91a11f1052b88ae6efdfeb7e42b6233dbfde84` |
| 제출 candidate | `d8f65793be33890deeaef8f2e0ccbe019a02a147`, 이후 source/test 변경 없음 |
| 고정 base | `505661360e9a2d596f55300d0cb0c5222f0e14b4`, 제출 직전 최신 devel과 동일 |
| 최초 규모 | 55 files / +2,796 / -129, 코드·테스트·보고서·PNG 포함 |
| 작성 시점 참고값 | Open, MERGEABLE, CI 대기; reviewer는 self PR 규칙에 따라 지정하지 않음 |

- base route: `collaborator_self_merge.md`
- modifiers: `intake_and_review.md`, `local_validation.md`, `visual_fixture_evidence.md`, `review_only_fast_pass.md`, `rework_and_exceptions.md`(1,000줄 초과)
- loaded documents: `pr_review_workflow.md`, `pr_review/README.md`, 위 기본·보조 문서와 `review_template.md`, `github_operations.md`
- 원격 권한: push=true, admin=false. protected branch 설정 API는 404로 조회 불가였으므로 PR checks의 실제 결과를 사용한다.
- 대형 PR은 별도 검토·simulation·시각 검증을 거쳤다. 이번 작업에 merge/admin 우회는 포함하지 않는다.

## 변경과 검토 범위

목록·바이트 읽기·변경 구독을 `HostFontSource`에서 분리하고 `local-fonts`의 face 선택을 두 renderer가 공유한다.
연결 세대/revision/ID로 이전 데이터 재사용을 방지하고, 목록 조회 시 파일 바이트는 읽지 않는다.
CanvasKit Typeface와 Canvas2D FontFace는 각각의 문서 자원 경계에서 준비·정리한다.

Canvas2D의 보조 측정·paint는 동일 session의 동기 별칭 scope를 사용한다. 공급자 변경은 측정 캐시와
페이지 자원을 무효화하고 실제 갱신을 유발한다. `withPortableMetrics`의 scope 해제로 HWP/HWPX·SVG에
내부 이름이 유출되지 않도록 했고 실제 저장·재열기/교체 전후 SVG/undo·redo를 대조했다.
메타데이터와 로컬 폰트 CSS 사용 가능 판정·영속 저장은 분리돼 있다.

기존 E2E manifest 3건 누락의 등록 보완이 별도 commit에 포함돼 있다. 검증 도구·기준값을 완화하지 않았다.
네이티브 IPC·OS 권한·출력 환경 FontFace 공급·PDF/인쇄·출력 snapshot은 호스트 책임이며 Rust 조판 변경은 없다.

## 검증 입력과 결과

[최종 보고서](../../report/task_m100_7403_report.md)의 명령·계약별 관측값·독립 기대값·RED/GREEN을 재사용한다.
fresh web/Node dev WASM, TypeScript, Studio production build, **1,787 PASS / 0 FAIL / 0 SKIP**,
양쪽 renderer E2E, 기존 CanvasKit 글꼴 coverage, manifest 140/140, 합성 글꼴 재생성 hash 검사가 통과했다.
Rust/Cargo·Rust test/baseline helper를 바꾸지 않았고 새 글꼴은 TypeScript 브라우저 fixture이므로 Rust 전체 lint/integration은 비해당이다.
Native CLI는 Visual Sweep용으로 빌드했다. optimized release WASM을 검증했다고 주장하지 않는다.

실제 제품 API와 CanvasView에서 동명 글꼴 교체·read 실패 복구·해제·문서 전환을 검사했다.
abort를 무시하는 응답 및 reset 뒤의 FontFace.load 완료가 재등록·재paint하지 않는 반례도 통과했다.
Oblique-only family 회귀는 수정 전 실제 선택 실패와 수정 후 통과를 보존했다.
기존 머리말/꼬리말 소스 연결 검사의 `method!()` 인식 실패는 검사식 보정 뒤 통과했으며 제품 결함으로 집계하지 않는다.

### 검증 입력 커밋 확인 — 충족

실제 사용한 바이트와 제출 candidate의 Git blob을 직접 대조했다. 모두 기존 저장소 파일을 재사용했다.

| 입력 | 역할 | SHA-256 |
| --- | --- | --- |
| `samples/re-01-hangul-only-hancom.hwp` | 실제 한컴 원문 1쪽 | `61538931d2e2cf38f35050618ce7698960823938884d0d8977812c94587e85fd` |
| `pdf/re-01-hangul-only-hancom-2022.pdf` | 동일 원문의 한컴 기준 PDF | `ceb38d48887392ade56aef29be6bb65d77dd89e77a49c4e9b04a9b5323159684` |
| `samples/basic/issue2007_nested_cell_pagination_42065.hwp` | 실제 17쪽 문서의 10쪽 charOverlap | `bebd4ce3691246b0fb3ae332e1d40bc51d9035cddb9fc3d378466b6a8a2b5626` |

합성 TTC/TTF는 독창적인 outline·advance의 계약 입력이다. 한컴 기준 출력으로 취급하지 않는다.
HCRBatang 공급 파일은 PDF subset과 outline 동일성을 확인했으나 라이선스 바이너리는 PR에 추가하지 않았다.

### 조판 원칙 준수 검토

| 항목 | 판정 | 근거 |
| --- | --- | --- |
| 구현 근거·일반성 | 충족 | face ID·스타일·revision 계약으로 분기. 특정 문서/좌표 clamp 없음 |
| 측정·배치 일관성 | 충족 | Canvas2D 보조 측정과 paint에 같은 FontFace. 독립 TTF 픽셀 및 43.2/51.84/57.6px advance 대조 |
| 분할·이어받기, 줄 소속·점유 높이 | 비해당 | Rust 줄 구성·LineSeg·pagination·좌표 알고리즘 변경 없음 |
| 사례·증거 독립성 | 충족 | 합성 TTF 계약과 실제 한컴 원문/PDF를 분리. 실제 charOverlap 호출 검사 |
| baseline·golden·허용치 변경 | 비해당 | 변경 없음 |
| 주장과 범위 | 충족 | [기계 판독 증거](../../report/assets/issue7403/validation.json), 아래 미검증 범위 구분 |

## 시각 증적과 남은 차이

[Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#pr-body-visual-evidence)을 적용했다.
동일 원문·PDF 1쪽의 한글 3줄을 Native/fresh WASM/CanvasKit/Canvas2D로 산출하고 review·overlay를 직접 확인했다.
누락·겹침·줄바꿈 변화는 없고 가장자리·농도 차이와 screen guide가 남는다.
Native/WASM layout·text clip·cell boundary 후보는 0, 텍스트 차이는 0/0이었다.
표·그림이 없는 원문이므로 OVR은 비해당이다.

| 경로 | pixel match | 엄격 내용 픽셀 | 2px 실루엣 / gate |
| --- | ---: | ---: | --- |
| Native / fresh WASM | 99.42164% | 28.63272% | 100% / passed |
| CanvasKit | 99.38160% | 16.46720% | 100% / passed |
| Canvas2D | 99.37218% | 21.60762% | 100% / passed |

대표 PNG는 `mydocs/report/assets/issue7403/{native,wasm,host-canvaskit,host-canvas2d}-{review,overlay}-001.png`에 있다.
PR 본문은 head SHA 고정 raw 이미지로 네 경로를 직접 표시한다. 실루엣 점수를 전체 한컴 호환성으로 해석하지 않는다.
로컬 실행·중간 산출물은 `/private/tmp/rhwp-7403-validation/common-*`다.

다른 브라우저 엔진·WebGL/WebGPU·대규모 다쪽 성능, patch·머리말/꼬리말의 개별 호스트 글꼴
브라우저 수용 사례는 미검증이다. 같은 scope 연결의 코드 확인을 개별 실행 증거로 승격하지 않는다.

## CI와 병합 전 확인

제출 candidate의 [CI run](https://github.com/edwardkim/rhwp/actions/runs/35974531896)이 시작됐다. 기록 commit의 push는 candidate CI 완료 후 수행하고,
최신 trailing head의 required checks와 mergeability를 별도로 확인한다. 완료한 로컬 검증을 반복하지 않는다.
문서 trailing commit은 최신 base와 merge-tree·공백·변경 문서 링크·오늘할일 기존 기록 보존 검사를 통과한 뒤 push한다.

## Merge 후 contributor PR comment 계획

본인 PR이다. merge가 승인되면 최종 merge SHA·CI URL과 [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을
연결하고 같은 네 경로의 PNG를 `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-sha>/mydocs/report/assets/issue7403/<파일>`로 표시한다.
위 1쪽·후보 0·지표·직접 판독 및 미검증 범위를 함께 쓰고 `--body-file` 게시 후 API로 본문을 대조한다.
이 계획은 현재 merge 또는 후속 comment 게시를 수행했다는 뜻이 아니다.
