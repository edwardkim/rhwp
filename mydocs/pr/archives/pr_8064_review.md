---
kind: report
status: active
last_verified: 2026-10-19
---

# PR #8064 리뷰 — 수식 메트릭·미주 조판과 출력 정합성

## 최종 판정

수용 가능 — 검증한 코드 head `a3ca43733dc90fd02d25bd35a046ae3003ef4161`의 필수 CI가 모두 성공했고,
[PR #8064](https://github.com/edwardkim/rhwp/pull/8064)는 정상 merge로 병합했다.
Merge SHA는 `9b01d5d1447220b3a7345703e79ad42cf3f502f4`이며 `upstream/devel` 포함을 확인했다.
작성자 `jangster77`의 self-review다. 작업지시자가 자동 PR·정상 병합·후속 처리와
**옵션 2의 별도 review·오늘할일 PR**을 승인했다. `--admin`이나 required check 우회는 사용하지 않았다.

- 코드 PR 기준 devel: `224a326063a5aeab5a1ca8dcaa92918f55843ffc`.
- 엔진 실출력 source: `d7d4d2ea6ed427f2f3bc8b59a936a5f99aca852b`. 이후 제품 함수 동일성을 확인했다.
  Studio Print DOM 보정은 `fa84ca12576ee455767afb382ca1fbe617e08d35` 이후 실제 브라우저 출력이다.
- base 실출력 source: `6ed1682c3267ea786734eb2e75252916a861d1fd`; 위 devel과 런타임 동등하다.
  devel SHA에서 새로 컴파일했다고 보고하지 않는다.
- 코드 CI 성공 뒤 번호가 확정된 기록을 별도 branch에서 작성했다. 기록 PR의 CI·병합과
  최종 issue/comment·정리 결과는 해당 PR과 원 코드 PR의 후속 comment에서 확인한다.

## 구현 주장·소비 경로·실행 증거

| 범위 | 공통 결과와 최종 소비 | 실제 증거·한계 |
| --- | --- | --- |
| 수식 글꼴·장식·첨자 | `renderer/equation/layout`의 source frame·advance·기준선을 SVG/Native/Canvas가 소비한다. 장식 예약 크기와 표시 잉크를 구분한다. | [최종 수식 대조](../assets/issue7642/cifonts/README.md): 3종 전체 6쪽, 양쪽 99.77% 이상. 원래 첨자 결함의 before FAIL/after PASS는 stage171, dyad는 stage165 기록이다. |
| 언어별 메트릭·중립 공백 | `composer`의 `neutral_space_origin`, `layout/paragraph_layout`·terminal tracking이 같은 언어 슬롯·전진을 소비한다. 원본 글꼴 공백 계약은 유지한다. | [영어 증거](../assets/issue7642/english_neutral_space_20261019/README.md), HWP3 서명 회귀: 수정 전330db의 중립 슬롯 23불일치·5줄 실패, 수정 후 0. 완료 영어 5쪽은 양쪽99.56% 이상이며 독립 원점±2px와 내용 관계를 함께 보호한다. |
| 저장 줄·표·미주 이어받기 | `typeset/notes/endnotes/{prepare,measure,fit,row_projection,emit}`와 stored TAC가 원본 줄 소속·소유·후속 내용을 보존한다. | 52개 새 시험지 검사와 표 분할53조건, #7969의13개 독립 대조군을 최종 전쪽 범위에 포함했다. 페이지 수만으로 내용 보존을 대신하지 않는다. |
| Print 그림 변환·crop | `image_resolver/print_tone`의 `requires_print_bitmap_transfer`를 payload와 `source_image_variant`가 함께 소비한다. `png_sampling` metadata와 bounded cubic을 Native/Canvas2D/CanvasKit이 소비한다. | [그림 증거](../assets/issue7642/bitmap_visible_palette_20261019/README.md). 실제 변환 그림만 표식이 있으며 대조5개는 `[true,false,true,false,false]`. 원본·비변환·회전·큰 축소를 같은 정책으로 강제하지 않는다. |
| 실제 Studio Print | `PageRenderer`의 `renderProfile === 'screen'` 분기 뒤 Print는 같은 프로필의 static canvas를 사용한다. | 실제 진입점 Node before 1 PASS/3 FAIL → after 4 PASS. 실제 브라우저의 독립 Print RGB 138/171을 Canvas2D/CanvasKit에서 확인했다. 좁은 대역 검사만으로 전체 화면을 입증했다고 쓰지 않는다. |
| 새 시험지 회귀 | `verified_exam_source_content_and_pages.rs`의52개 새 함수가 원본 SHA·수식 다중집합·본문/각주/미주 소유·순서·독립 Print 쪽 소속을 검사한다. | [52개 원본 증거](../assets/issue7642/exam_source_relations_20261019/README.md). 50개 문서는 완료 페이지의 독립 구조 경계 시작점±2px도 검사한다. eq-002 두 입력은 관계만 검사한다. 전체 글리프·선 끝점을 px로 고정한 검사가 아니다. |

[최종 검증 종합](../../working/task_m100_7642_stage4178.md),
[실제 소비 경로·보정 이력](../../working/task_m100_7642_stage3905.md),
[CI 글꼴 의존성 보정](../../working/task_m100_7642_stage4183.md)에 구체적인 계약·명령·전후 관측을 연결했다.
52개 보존 검사가 모두 수정 전 결함을 검출한다고 주장하지 않는다. 개별 결함의 before FAIL/after PASS와
보존 회귀를 구분한다.

## 전체 시각 검증과 직접 판독

- [Native/fresh WASM 원문 TSV·입력/Print 색인](../assets/issue7642/final_sweep_20261019/README.md):
  각 179조건·2,082쪽 전쪽 raw≥90%, 독립 한컴 Print 쪽수 일치, 누락·측정 실패0.
  Native 최저 90.16447%, WASM 최저 90.10782%다. 조건 수를 고유 문서 수로 바꾸어 보고하지 않는다.
- 같은 base·입력·PDF·글꼴·DPI·backend의 비교 가능한 각 1,660쪽에서 0.1%p 이상 하락 0이다.
  base의 쪽수가 이미 다른18조건은 점수 비교에서 분리했고 후보 전쪽 수·대응 내용을 독립 Print와 확인했다.
- 수식 진단 3종도 같은 고정 base와 양쪽 각 6쪽을 추가 대조해 **12쌍 모두 개선**됐다.
  dyad Native 82.88512→99.77969%, WASM 82.22342→99.88334%다.
  [base 원문 TSV와 페이지별 표](../assets/issue7642/cifonts/README.md#고정-devel과-추가-전쪽-대조)를 보존했다.
- 수학 12쪽의 머리글 4·문항 29/30·수식·표·동전·꼬리말, 아래첨자·근 지수·dyad,
  영어 간격, 그림/차트 경계와 사회 #7763 영역의 review·standalone overlay를 직접 대조했다.
  raw 90% 통과로 확대 판독을 면제하지 않았다. 글리프 굵기·안티앨리어싱·선색 잔차가 있으며 동일 이미지라는 뜻은 아니다.
- 최신 PrEP MCP Print의 전체 140쪽은 기존 독립 Print와 raster가 바이트 동일하다. 기준 PDF를 바꿔 하락을 숨기지 않았다.
  새 50 MiB 이상 PDF·실행 로그·중간 JSON은 Git에 넣지 않았다.

## 로컬 검증과 정확한 head CI

| 검증 | 결과 |
| --- | --- |
| 최종 전체 Rust nextest | **11,014 PASS / 0 FAIL / 45 SKIP** |
| Native Skia library | **4,041 PASS / 0 FAIL / 12 IGNORE** |
| Native 집중5묶음 | **1+2+2+2+14 PASS** |
| fmt·diff·suite/unit 정책·workspace all-target Clippy | 보정 후 재실행 포함 exit0 |
| Native/WASM32/Native Skia Clippy·workspace build·release/lib/doc | 관련 소스 동일성 확인 후 성공 증거 재사용 |
| Studio 타입·전체 Node·4종 패키지 빌드 | **2,338 PASS / 2 SKIP**, 전체 7단계 exit0 |
| 실제 Canvas2D·CanvasKit·SVG 소비 | 수식4경로·그림 프로필·원본/대조군·공개 SVG 실제 출력 확인 |

최초 코드 CI `39181669838`은 Archive B/D의 글꼴 의존 검사 2건이 실패했다. 이를 제품의 새 회귀나
골든 갱신 사유로 오판하지 않았다. 공개 AB 글꼴을 명시하고 PUA 아래첨자의 실제 SVG 방출·변환 뒤
기준선을 검사하도록 보정했다. 시스템 글꼴 없는 정확한 4개 검사도 4 PASS다. 사설 글리프의 실제 윤곽을
CI에서 검증했다고 주장하지 않으며, 그 모양은 별도의 Native/fresh WASM·독립 Print로 확인했다.
보정 뒤 전체 Rust와 Clippy를 다시 실행했다. 최종 커밋의 검사 바이트와 로컬 검증 파일 SHA가 같음을 확인했다.

최종 코드 head `a3ca43733dc90fd02d25bd35a046ae3003ef4161`에서 다음 workflow와 required status가 모두 성공했다.

- [CI — Archive A/B/C/D·Native·Lint·패키지·집계](https://github.com/edwardkim/rhwp/actions/runs/39186195834)
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/39186195811)
- [Render Diff](https://github.com/edwardkim/rhwp/actions/runs/39186195773)
- [Adapter inter-diff](https://github.com/edwardkim/rhwp/actions/runs/39186195843)
- [Proptest roundtrip](https://github.com/edwardkim/rhwp/actions/runs/39186195822)
- CI Impact Policy: SUCCESS. head·base・MERGEABLE/CLEAN을 병합 직전 다시 확인했다.

병합 후 [Refresh nextest target duration data](https://github.com/edwardkim/rhwp/actions/runs/39188194988)와 저장소의 [이슈 종료 운영 workflow](https://github.com/edwardkim/rhwp/actions/runs/39188195081)가 성공했다. 보호 branch push의 검증 CI 재실행은 없었다.

duration 수집은 `ready=true`, source run `39186195834`·head `a3ca43733d`·B/C/D attempt1을 검증했고,
metrics 전용 branch에 `884f8d4a`를 게시했다. devel에 검증 CI를 다시 실행하지 않았다.

병합 뒤 검증 CI를 새로 시작하거나 `--admin`으로 보호 규칙을 우회하지 않았다.

## 조판 원칙 준수 검토

| 검토 항목 | 판정 | 확인 근거 |
| --- | --- | --- |
| 제출 전 로컬 검증·Open PR | 충족 | 위 로컬 게이트와 정확한 코드 head CI. Draft 우회 없음. |
| 기존 이슈·해결 PR 선행 분석 | 충족 | #1950·#7969와 golden 원 PR #5412/#5458/#5438의 계약·실제 입력을 기존 단계 기록에서 대조. |
| 구현 근거·일반성 | 충족 | 원본 source frame·언어 슬롯·실제 보이는 crop·프로필 조건으로 구현. 문서명/페이지 분기나 출력 은폐 없음. |
| 측정·배치 일관성 | 충족 | 위 공통 메트릭·줄 구성·이미지 payload의 실제 소비 경로와 정식 관계 검사. |
| 리베이스 기능 보존 | 충족 | #7969의13조건과 표 분할53조건을 포함한 최신 전쪽 비교. 같은 base 대비 하락 0. |
| 최종 backend 소비 | 충족 | Native/fresh WASM와 실제 Canvas2D/CanvasKit/SVG 출력. Screen 성공으로 Print를 대신하지 않음. |
| 분할·이어받기 | 충족 | 저장/재조판·컷·소유·후속 내용 관계 및 전쪽 경계 증거. 세부 경로는 기존 stage3905에 연결. |
| 줄 소속·점유 높이 | 충족 | 저장 LineSeg와 재조판을 구분하고 원본 문단/셀/미주 소유·누락·중복·순서를 검사. |
| 사례·증거 독립성 | 충족 | 한컴 Print와 합성 계약을 구분하고 실제 before FAIL/after PASS와 보존 검사도 구분. |
| 기준값 변경 | 충족 | 시험지의 기존 고정/공용 검사 이관과 새 정상 관계 검사, 새 1/1 fixture를 구분. 실패 golden을 일괄 갱신하거나 공차를 완화하지 않음. |
| 회귀 추가·변경의 선행 조건 | 충족 | 관련 전쪽 양쪽90% 이상 뒤 등록. 완료 페이지 양쪽98%·직접 판독 조건일 때만±2px. |
| 90% 후 확대 판독 | 충족 | 위 수식·간격·괘선·그림·사용자 지적 영역의 양쪽 review/overlay. |
| 주장·검증 범위 | 충족 | 실제 producer·SHA·명령·결과를 구분. 아래 미해결 이관 범위를 완료·Closes로 승격하지 않음. |

## 해결 범위·별도 이슈와 후속 처리

- **Closes #7763**: 원본 사회 HWPX의 표 위치·쪽번호 잘림·보기 자모를 확인했다.
  [전체 4쪽98.07087%/98.05641%와 지적 영역](../assets/issue7642/social_issue7763_20261019/README.md).
- #7445, 편집 가능한 차트 #8044, 수식 카탈로그 #8046, WMF/EMF #8047, 합성 저장 줄 #8048,
  영어의 별도 전진 잔차 #7730 등은 완료로 보고하거나 닫지 않는다.
  [실패 계약 3건의 원문·독립 자료·복원 조건](../assets/issue7445/remaining-regressions-20261019/README.md)을 보존했다.
  이는 사용자가 승인한 분리 범위이며 미실행 항목을 통과로 간주한 것이 아니다.
- 별도 기록 PR 완료 후 최종 devel 동기화, issue/PR 후속 comment와 이번 작업 전용 branch/worktree/output 정리를 수행한다.
  필요한 원본·Print·글꼴·로그·재현 자료의 영구 보존 루트는
  `/home/tsjang/rhwp-local-validation/issue7642-20261019`다. 공유 `target/pr-review`는 보존한다.
  기존 output 경로와 영구 경로의 대응은 `relocation-index.jsonl.gz`, 실제 실행 결과는 원 PR 후속 comment와 종료 로그에 남긴다.
