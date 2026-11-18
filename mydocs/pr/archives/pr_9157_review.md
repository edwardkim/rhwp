---
kind: snapshot
status: historical
canonical: mydocs/manual/pr_review_workflow.md
last_verified: 2026-11-18
---

# PR #9157 리뷰 — HWP3 그림 소속과 Native 글꼴 선택

## 최종 판정

**병합 완료 — 작업지시자의 현재 상태 병합 지시에 따라 처리했습니다.**
보호 문서 검증은 Mac에 한정하지 않고 모든 OS에 적용합니다. 현재 후보의 정본 보호 18종 전쪽 Native/fresh WASM 검증은 미완료이며, 통과로 변경하지 않았습니다.
작업지시자가 이를 설명받은 뒤 “일단 머지 후 후속 처리”, “그냥 머지 후 후속처리해”라고 명시했습니다. 이 건의 병합 지시를 기록하며 일반 정책·보호 하한·기준값은 바꾸지 않았습니다.

- [원 PR #9157](https://github.com/edwardkim/rhwp/pull/9157), 작성자 `jangster77`, 최종 head `56bd8652077b8d0a186e1e07b9adeea43c8fbc2f`.
- 최신 base `8612752c0eb50ea74755906539e121aa9abd29d7` 반영 후 필수 check 성공과 mergeability를 확인하고 2026-11-18 13:11:43 KST에 merge했습니다. 확정 merge: `af54158960ec91504d42faf842b0cb3229366236`.
- [최종 head CI](https://github.com/edwardkim/rhwp/actions/runs/44234549351)와 CodeQL·Render Diff·Proptest·Adapter는 성공했습니다. current-base merge 검증으로 기존 성공 결과를 재사용했으며 heavy worker skip을 새 검사 실행으로 세지 않습니다. [실제 실행한 trailing CI](https://github.com/edwardkim/rhwp/actions/runs/44150810947)는 Archive 13,524 PASS/45 skip, Native 247 PASS였습니다.
- [병합 영수증·각 workflow·이미지 hash](../assets/issue_8371/essential-validation-20261117/merge-followup-20261118/receipt.json). [duration 작업](https://github.com/edwardkim/rhwp/actions/runs/44235622959)은 성공 종료했으나 검증된 실측값이 없어 갱신을 보류했습니다. 병합 뒤 검증 workflow를 재실행하지 않았습니다.
- 새 base의 Studio 로컬 추가 확인: 단위 2,806 PASS/0 FAIL/2 skip, TypeScript PASS, #9165 E2E PASS. #9164 E2E는 base와 checkout에 모두 없는 로컬 webfont 때문에 완료하지 못했습니다. 임시 글꼴 재시도도 테스트 시작 전 준비에서 실패했으므로 PASS로 기록하지 않습니다. 제품·검사·글꼴 파일을 변경하지 않았습니다.
- #8371/#8218은 기존 조사 맥락이며 이미 CLOSED였습니다. 이번 PR로 새로 해결·종료했다고 표시하지 않습니다.
- 본 문서·증적·오늘할일의 확정값은 권한에 맞게 별도 후속 기록 PR로 반영합니다. 이후 devel 동기화, 원 PR 댓글, 작업 전용 branch/worktree 및 output/debug 정리로 종료합니다. 사용자 요청 백업과 다른 작업은 보존합니다.

## Merge 후 contributor PR comment 계획

원 PR의 merge `af54158960ec91504d42faf842b0cb3229366236`에 고정한 아래 PNG 5개의 HTTP 응답 형식·bytes·SHA-256을 저장소 원본과 대조했습니다. 후속 기록 PR 병합 뒤 원 PR에 한국어 존댓말로 실제 이미지를 포함해 게시하고 API 본문을 확인합니다.

- 제품 producer `fcf2e11b1727297743343446333132e1409737f1`의 sample4 전체 36쪽 Native/fresh WASM 최소 raw 97.66586/97.66564%. 25·26쪽만 바뀌고 나머지 34쪽은 동일합니다. 변경 그림이 25쪽에서 26쪽으로 이어지는 소속과 설명·본문 보존을 직접 판독했습니다. 기존 코드 배경 하단 밴드 차이는 남습니다.
- sample4 Native/WASM 각각 `source-layout-enlarged.png`와 `sample4-overlay-026.png`, Windows `symbol200-review.png`를 merge SHA raw URL로 포함합니다. Windows 편람 200쪽 raw 100%, 검은 기호 3개와 주변 본문을 직접 판독했습니다. 실제 경로는 위 영수증의 `mergeImages`에 명시했습니다.
- 최종 head CI의 검증된 재사용과 실제 실행 producer를 구분하고, 보호 18종 미완료 및 전 OS 적용·사용자 병합 지시를 적습니다. 수치는 내용 픽셀 중심 자동 일치율 보조값이며 사람의 판정 정확도가 아님을 명시합니다.
- [Visual Sweep 댓글 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment)을 직접 링크합니다. #8371/#8218은 맥락으로만 연결하며 상태를 바꾸지 않습니다.

## 병합 지시 이전의 검토 이력

아래의 “현재”, “머지 보류”, 해제 조건은 2026-11-18 병합 지시 이전의 판단입니다. 현 상태는 위 최종 판정과 병합 영수증을 따릅니다. 당시 확인한 실행 결과와 미완료 사실은 그대로 보존합니다.

### 당시 판정

**머지 보류 — 현재 후보의 정본 보호 18종 동일 공급 조건 확인과 전쪽 검증이 남아 있다.**
정확한 code head `0b54ec2b3f4fcb416c215f3c152c37b299202443`의 Full CI·CodeQL·Render Diff·Adapter·Proptest 성공 뒤 작성한 self-review다. 이후 문서 trailing head의 CI는 별도로 확인한다.
원본 글꼴 공급 조건의 확인·전쪽 Native/fresh WASM 보호 검증을 마치기 전에는 승인·병합하지 않는다.

## 접수와 권한

- PR: https://github.com/edwardkim/rhwp/pull/9157, 작성자 jangster77, devel 대상 Open PR. collaborator self 경로이며 reviewer를 지정하지 않는다.
- 제출 head: `0b54ec2b3f4fcb416c215f3c152c37b299202443`.
- 제품 producer: `fcf2e11b1727297743343446333132e1409737f1`; 검사 이관 producer: `699c0afbbc86db1252128b8dcc09630038a8d971`.
- 최초 base: `22da83586aab1baf960f2159abda347062ecf7ed` (#9156). 확인한 최신 base: `fbf8081db57c0c6abc9bad418290c5018193e24f` (#9155/#9160).
- GitHub 실제 merge commit `371190ca3d995ca6a27ae6965625526bd33387dd`의 두 parent가 위 최신 base와 제출 head이며, tree `df2f4f09a0808be4f3e22d488a1de1895b968196`는 로컬 simulation과 같다.
- #8371/#8218은 닫힌 기존 조사의 맥락이다. 새 해결 이슈로 표시하거나 자동 close하지 않는다.
- 작업지시자의 테스트·PR·후속 처리 승인 범위에서 진행한다. 현재 보류를 원격 CI 성공이나 과거 PR로 면제하지 않는다.

## 구현과 공통 결과

HWP3 저장 그림 호스트는 앞 쪽에 남지만 그림 프레임이 그 쪽을 넘고, 명시적 다음 쪽 경계 뒤 저장 빈 줄이 그림 전체 높이를 예약한 경우 그림의 소속을 다음 쪽으로 옮긴다.
`float_placement.rs::hwp3_square_picture_before_page_break_placement`는 실제 저장 LineSeg, 유효한 미편집 상태, 단일 단, Square·문단 기준 Top 앵커와 완전한 예약 밴드를 확인한다. 파일명·문서 ID·특정 px 상수 분기는 없다.

데이터 흐름은 `frame_height/occupied_bottom/anchor_y → section/controls.rs의 defer_stored_frame → state/transition.rs의 다음 쪽 소유 확정 → column의 paragraph_float_placements → layout.rs의 Shape 배치`다.
저장 그림의 `next_flow_y=0`은 다음 저장 줄이 이미 예약한 그림 높이를 다시 더하지 않는 계약이다. 같은 공통 placement를 소비하는 paint가 첫 본문 원점·이전 문단 상태를 그림 호스트로 덮어쓰지 않는다.
표의 rowspan·행 분할 컷은 변경하지 않는다. 실제 새 경계는 전체 그림의 앞/다음 쪽 소속과 뒤 제목·설명·본문의 보존이다.

Native의 빈 기호 검사는 DWrite가 직접 stream을 주지 않을 때 SFNT table 복구를 사용한다. 독립 Print의 U+26AB 윤곽과 실제 Windows p200의 원 3개를 확인했다.
원본 컨테이너 순서는 family → CSS style → glyf TTF → glyf TTC face → CFF OTF → 알 수 없는 프로그램 순이다. 다른 family의 대체 TTF가 원 family의 OTF를 앞서지 않는다.
직접 공급한 bytes에서 원본 형식을 보존하며, OS가 감춘 설치 프로그램은 PostScript 이름과 모든 table의 동일성으로만 복구한다.

## 실제 검증과 범위

증적 정본: [모든 검증 결과·TSV·대표 PNG](../assets/issue_8371/essential-validation-20261117/README.md).

| 범위 | 실제 결과·producer |
| --- | --- |
| 새 검사 head의 기존 Native CI 호출 | 24 PASS, 이관한 두 RGBA 검사 이름의 실제 실행 확인 / 699c0afbbc |
| 새 검사 head 집중 | 33 PASS, 나머지 990개는 명시 filter 제외 / 699c0afbbc |
| 새 검사 head 전체 nextest | 13,718 PASS / 0 FAIL / 45 skip, 890.457초·4 slow / 699c0afbbc |
| 새 검사 head lint | fmt·root/Native/WASM Clippy·Native all-targets·workspace build/all-target Clippy·suite 정책 PASS / 699c0afbbc |
| 제품 head의 Native 광범위 | lib 3,853 PASS·12 ignored, workspace 182 PASS, missing-picture 2·direct-PDF 36 PASS / fcf2e11b17 |
| 제품 head doctest·suite 수집 계약 | 8 PASS·3 ignored / Node 24 PASS / fcf2e11b17 |
| sample4 Native/fresh WASM | 전 36쪽, 최저 97.66586 / 97.66564%, 변경 PNG는 p25·26뿐이고 나머지 34쪽 동일 / fcf2e11b17 |
| 명시적 편람 Native gate/fresh WASM | 각 383쪽, 최저 99.96504 / 99.96448% / fcf2e11b17 |
| 새 devel의 band/tab 원문 | Native/WASM 모두 100%; 기존 7개 집중 기능 검사 PASS / fcf2e11b17 |
| 일반 기여자 공통 4문서 | Native/WASM 총8건 PASS; 전체 보호 18종과 구분 / fcf2e11b17 |
| 실제 소비 | 원본 글꼴 Studio Canvas2D·Native SVG, 지원 TTF paint의 실제 CanvasKit p26, 일반 뷰 9상태·복원 PASS / fcf2e11b17 |
| 실제 Windows DWrite | dev Native p200 raw 100%, 기호 3개와 주변 본문 직접 판독 / fcf2e11b17 |

검사 이관은 두 helper/assertion의 본문을 보존했다. 제품·빌드 입력은 동일하며, 제출 head에서 이후 변경은 mydocs 증적뿐이다.
Cargo 실행의 실제 fc-list는 허가한 OSS 공급 목록과 같고 시스템·개인 글꼴 디렉터리를 숨긴 bwrap 안에서 실행했다. 두 Native 프로그램은 저장소 OFL 합성 자원으로 명시 공급한다.
허용치·기대값·baseline을 낮추지 않았고, CI workflow·action·실행/재사용 정책을 수정하지 않았다. 따라서 후속 기록의 옵션2 대상이 아니다.

## 조판 원칙 준수 검토

| 항목 | 판정과 근거 |
| --- | --- |
| 사양·독립 근거와 원인 계층 | 충족: 원본 HWP3 저장 줄·그림 프레임·쪽 경계와 독립 Hancom Print를 대조했다. Native 글꼴은 실제 프로그램 table·독립 PDF 기호 윤곽을 사용했다. |
| 측정·paint 공통 결과 | 충족: 위 placement 생산→defer→다음 쪽 column→layout 소비와 그 뒤 원점 덮어쓰기 차단을 추적했다. |
| 분할·이어받기 | 충족: 그림 전체 이월의 한 번 소유, 저장 밴드 중복 계상 방지, 앞 제목과 다음 설명·본문 보존을 실제 출력·관계 검사로 확인했다. 표 컷/rowspan 변경은 비해당이다. |
| 저장 정보와 편집 | 충족: 실제 저장·미편집 HWP3만 새 수용 경계에 해당한다. 합성/dirty 줄과 편집 후 재조판은 이 수용 조건을 적용하지 않는다. |
| 회귀 추가·기존 검사 | 충족: sample4 전쪽 Native/WASM ≥90%; p21은 양쪽 100%와 독립 Print 좌표 근거로 기존 ±2px를 유지했다. 새 p26은 소속·순서·보존·비겹침 관계다. |
| 입력 커밋 | 충족: 8개 원문·독립 Print 및 환경 파일 총18개의 실제 실행 bytes를 commit과 대조했고 이후 검사 이관에서 변경되지 않았다. 합성 TTF/TTC는 별도 프로그램 계약이다. |
| devel 대비 하락 | 충족: 동일 조건 전쪽 base/candidate TSV, sample4 양쪽 모든 쪽의 2%p 초과 하락 없음; band/tab 100% 유지. |
| 리베이스와 새 base | 충족: #9156의 진단·band·tab 실제 기능 보존을 확인했다. 이후 #9155/#9160은 제품 경로 변경 없이 CI 운영만 전진했고 실제 GitHub merge tree 동일성을 확인했다. |
| 최종 출력 소비 | 충족: 원본 CFF의 CanvasKit 요청이 Canvas2D로 fallback된 결과를 실제 CK 성공으로 세지 않았다. p26 지원 TTF 대조군은 원본 font fidelity와 분리해 geometry·resource·복원을 검사했다. |
| 확대 판독 | 충족: Print/base/candidate의 p21·25·26, 알려진 p22·26 차이, 편람 130·200과 Windows 200, 공통 8건을 직접 판독했다. 원본 공급 Studio와 정상 뷰 9상태도 확인했다. |
| 정본 보호 전 18종 | 미검증: 동일 원본 공급 조건의 현재 후보 전쪽 Native/fresh WASM 검증이 없다. 이것이 통합 blocker다. |
| 최신 원격 CI | 충족: 정확한 제출 head 0b54의 5개 workflow가 모두 completed/success이며 실제 Native 두 검사 PASS도 확인했다. 이전 95a7 checkout 취소는 재사용하지 않는다. |

## 남은 차이와 해제 조건

sample4 p22·26 코드 배경의 기존 하단 밴드 차이는 base와 같고 별도 광범위 배경 변경은 제거했다. 문서 전체 완전 일치를 주장하지 않는다.
원본 CFF의 미지원 CanvasKit 자원과 탐색 대조군의 glyph mapping 실패는 base에서도 재현했다. 실제 기본 소비 경로와 지원 자원 대조군의 증거 범위를 구분했다.

병합 보류 해제에는 동일 공급 원본 글꼴·입력·Print·compat·DPI로 정본 보호 18종 전쪽 TSV를 산출해 공식 표시값 하락·누락이 없음을 확인하는 증거가 필요하다.
현재 PC·Ubuntu·기존 배포 백업·다운로드 보관본·설치 vendor 글꼴을 확인해 일부 원본을 추가 확보했지만 전체 공급 정합은 미완료다. 미확인 파일 전부가 실제 선택된다는 뜻으로 이 재고를 확대하지 않는다.
원본 공급 재고는 309개 선언 중 정확한 SHA 286개를 찾았고 23개가 미확인이다. 이 재고는 실제 선택 결과가 아니다. [조건·해시 확인과 현재 미완료 범위](../assets/issue_8371/essential-validation-20261117/native-ci-registration/protected-font-qualification.json)를 보존하며 사용자에게 특별 검증을 제출 의무로 전가하지 않는다. 위 필수 증거가 없으면 판정은 계속 머지 보류다.

## 병합 뒤 후속 계획

보류 해제와 최신 trailing head CI를 확인한 뒤 승인된 범위에서 정상 PR merge를 수행한다. 확정 merge SHA와 duration 갱신 상태를 확인하고, 기존 닫힌 이슈는 다시 종료하지 않는다.
PR에는 정확한 CI와 source·검증 수치, merge SHA에 존재하는 sample4 확대/overlay·Windows p200 증거를 넣은 한국어 댓글을 게시한다. raw 응답 hash와 게시 본문을 확인한다.
devel 동기화와 이 작업 전용 branch/worktree 정리, output 전체 및 종료한 debug 정리를 수행한다. 사용자 요청 백업·다른 소유 작업과 공유 target의 나머지는 보존하고, 정리 로그는 output 밖에 둔다.

## 정확한 code head CI와 trailing 기록

[CI](https://github.com/edwardkim/rhwp/actions/runs/44139541354),
[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/44139541317),
[Proptest](https://github.com/edwardkim/rhwp/actions/runs/44139541171),
[Adapter](https://github.com/edwardkim/rhwp/actions/runs/44139541008),
[Render Diff](https://github.com/edwardkim/rhwp/actions/runs/44139540822)는 모두 제출 head 0b54에서 성공했다.
[실행·PR 신원과 실제 worker 결과](../assets/issue_8371/essential-validation-20261117/native-ci-registration/code-ci-0b54.json),
[Native TTF/TTC 두 검사와 24 PASS 실제 로그 발췌](../assets/issue_8371/essential-validation-20261117/native-ci-registration/native-ci-job-excerpt.json)를 연결한다.

#9156의 18종·각 1,771쪽 통과는 유효하며, base 수용 결과·입력·Print와 보호 하한을 재사용한다.
그 이후 그림 배치·Native 글꼴 선택을 바꾼 이번 후보의 전 18종 실행으로 대신 표시하지 않는다.
이 기록 commit은 제품·검사·허용치를 바꾸지 않는다. 실제 trailing head CI, 문서 병합·링크·오늘할일 보존 확인과 보호 검증 완료가 남은 병합 조건이다.

## 오늘할일 병합 경계

새 `mydocs/orders/20261118.md`가 최신 base와 source에서 각각 생성되어 실제 `git merge-tree`가 add/add 충돌을 보고했다.
source에 공유된 변경 없는 section 경계가 없으므로, 고정 current base `fbf8081db57c0c6abc9bad418290c5018193e24f`의 기존 #9155 기록과 이번 기록을 함께 보존하는 한 번의 문서 bridge를 적용한다.
수동 충돌 해결 경로는 이 오늘할일 한 파일로 제한한다. GitHub가 code head0b54를 시험한 실제 merge tree와 비교해 mydocs 이외 바이트가 같아야 하며, current-base merge 검사기·최종 merge tree의 링크·공백·기록 보존을 모두 통과한 뒤 push한다.
