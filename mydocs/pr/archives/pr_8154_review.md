# PR #8154 리뷰 — Native CLI Kitty graphics 출력

## 최종 판정

**승인 — 정확한 code head의 Full CI와 CLI 전송·실사용 증거를 대조했으며 수정이 필요한 결함을 발견하지 않았다.**
이 판정은 작성자 self-review 기록이다. GitHub approve·merge·이슈 close는 수행하지 않았다.
merge 전 조건은 문서-only trailing head의 실제 CI/재사용 판정, 최신 merge tree·원격 SHA 재확인과
작업지시자의 merge 승인이다. 작성 시점 code head는 Open·비 Draft·MERGEABLE이었다.

## 접수 정보

| 항목 | 값 |
| --- | --- |
| PR / 작성자 / base | [#8154](https://github.com/edwardkim/rhwp/pull/8154) / edwardkim / devel |
| 검토 code head | `d030087e240fe851edfe9ad486176bd1624dcf40` |
| code CI 기준 devel | `2e087a5b4bc1515ad877e8020b30114639a59392` |
| trailing 사전 점검의 최신 devel | `f7abae4a8e70c062c378db06ad64d2d2eacb88cd` |
| 이슈 | [#8150](https://github.com/edwardkim/rhwp/issues/8150), Closes 연결 |
| 지시 / reviewer | 2026-10-22 「self-review 를 진행하세요」 / 작성자 self-review, reviewer 요청 없음 |

기본 경로는 `collaborator_self_merge`, 보조 경로는 `intake_and_review`, `local_validation`,
`visual_fixture_evidence`, `rework_and_exceptions`, `review_only_fast_pass`다.
[라우터](../../manual/pr_review_workflow.md)와 [선택표](../../manual/pr_review/README.md), 해당 자식 문서를 읽었다.
1,000줄 초과 변경이므로 코드·증거·병합 검토를 분리했으며 즉시 admin merge하지 않는다.

## 코드와 주장 대조

| 구현 주장 | 적용 경로와 확인 | 근거와 판정 |
| --- | --- | --- |
| Native 전용 기능 | `renderer/mod.rs`의 비 WASM·native-skia gate → `renderer/kitty.rs`; CLI catalog·dispatch·capabilities·help 일치 | 일반 빌드 feature 거부와 Native help/capabilities 계약 PASS. Studio/WASM API·공통 producer 불변. 충족 |
| 흰 배경의 PNG 색 유지 | `encode_kitty_png` 12~32행 → `RasterPixels::validate/opaque_rgba_on_white` → 기존 Skia PNG encoder | premultiplied alpha·stride·독립 RGBA 기대값 테스트, 전쪽 22건 Pillow 복원과 같은 조건 PNG 일치. 충족 |
| 청크·패딩·종료·quiet | `write_kitty_png` 37~61행: 원문 3,072바이트씩, payload 최대 4,096, 첫 a=T/f=100/t=d/q=2, 마지막 m=0·CRLF | 1/2/3·3,071/3,072/3,073·6,144/6,145바이트 경계와 손상 스트림 독립 검사. 충족 |
| 오류와 stdout 분리 | `cli/outputs/kitty.rs` 29~116행 옵션·로딩·쪽 범위, 121~179행 렌더 완료 후 저장/전송·flush 처리 | feature 누락·잘못된 옵션 exit 2, 입력 실패 exit 1·빈 stdout, 중간 writer 실패 전파, 파일 모드 빈 stdout. 이미 보낸 바이트는 복구하지 않는 문서 계약. 충족 |
| 쪽 선택·파일·프로필·폰트 | 같은 CLI 121행 이후 → 기존 `render_page_pixels_native_at_width`의 profile/width/font 전달 | 기본 첫 쪽·전쪽/선택 쪽 파일·마지막 쪽·4 profiles×2 compat·반복 폰트 경로 실행 증거. 충족 |
| 실제 로컬/SSH 표시 | 직접 PNG 전송이며 로컬 파일 공유·응답 대기·외부 kitten 실행 없음 | kitty 직접/파일·SSH 화면 5건과 사용자 Windows 11 WezTerm 직접 SSH 한글 확인. 충족 |
| 자동 실행 수집 | tests/cases 원본 → generated suite 자동 배정 → Native CI helper 호출 | CI 로그에서 suite_020의 Kitty 8건 실제 PASS, Sixel 7건 PASS. 새 individual workflow/job·의존성 추가 없음. 충족 |

문서 이름/본문을 APC 제어 값에 넣지 않으며, 전송 writer는 완성된 PNG만 소비한다.
제품 소스·계약·검사기 14개 파일과 frozen producer의 출처는
[3단계](../../working/task_m100_8150_stage3.md) 및 [제출 증적](../assets/issue_8150_kitty/README.md)에 연결했다.
새 옵션·오류·PNG 조건은 기존 Sixel 경로의 의미와 대조했다. 기존 PNG/Sixel 코드는 변경하지 않았다.

## 조판 원칙과 검증 입력

공통 조판 규칙 변경은 **비해당**이다. 새 모듈은 기존 래스터 결과 이후의 소비자이며,
문단/줄 소속·메트릭·점유 원점·pagination·font producer를 생산하거나 덮어쓰지 않는다.
`document_core/queries/rendering.rs:2057`의 기존 경로가 `PageLayerTree → with_font_paths → render_pixels_at_width`를
제공하고 새 CLI가 그대로 소비함을 코드로 확인했다. 별도 clamp·문서별 예외·새 실물 golden은 없다.
WASM/Studio 실행은 같은 실제 소비 경로 근거로 비해당이며 WASM32 Clippy는 통과했다.

검증한 공개 HWP/HWPX 4개·독립 한컴 Print PDF 4개의 원문 경로·크기·SHA와
검토 commit의 Git blob 동일성을 [입력 표](../assets/issue_8150_kitty/README.md#기존-커밋-입력-확인)에서 확인했다.
모두 기존 커밋 파일이며 새 사본을 중복 추가하지 않았다. 50 MiB 이상 PDF는 없고 비공개 corpus는 사용하지 않았다.

## 정확한 code head GitHub CI

다음 run은 같은 저장소·PR #8154·source branch·code SHA의 `pull_request` 실행이며 모두 completed/success였다.
CI Impact Policy도 같은 SHA의 Full 실행(`mode=full;rfp=0`)을 확인했다.

| 검사 | 실제 성공 run |
| --- | --- |
| CI / Build & Test·Lint·Native·archive A/B/C/D | [39762459350](https://github.com/edwardkim/rhwp/actions/runs/39762459350) |
| CodeQL·4개 Analyze와 GHAS CodeQL | [39762459339](https://github.com/edwardkim/rhwp/actions/runs/39762459339), [GHAS check](https://github.com/edwardkim/rhwp/runs/119771176660) |
| Render Diff | [39762459103](https://github.com/edwardkim/rhwp/actions/runs/39762459103) |
| Adapter inter-diff | [39762459292](https://github.com/edwardkim/rhwp/actions/runs/39762459292) |
| Proptest roundtrip | [39762459214](https://github.com/edwardkim/rhwp/actions/runs/39762459214) |

Native CI의 실제 [job](https://github.com/edwardkim/rhwp/actions/runs/39762459350/job/119769866582)에서
`issue_8150_kitty_contract` 8건·기존 Sixel 7건이 실행되어 성공했다. WASM Build·frontend unit 등은
정책상 skipped이며 이를 실행 성공으로 세지 않았다. 패키지 gate·Render Diff 등 실제 실행된 별도 범위도 성공했다.

이번 self-review에서는 [중복 로컬 전체 회귀 생략](../../manual/pr_review_workflow.md#322-녹색-github-code-head의-중복-로컬-전체-회귀-생략)에
따라 전체 Cargo·Native·WASM 빌드를 다시 실행하지 않았다. code source·test·fixture·workflow 보정이 없다.
PR 전 로컬 전체 11,361 PASS·45 skip, Native lib 4,041 PASS·12 ignored와 focused·lint는
[3단계](../../working/task_m100_8150_stage3.md)의 실제 수행 기록을 재사용한다.

## 실사용·시각 증거와 남은 차이

[공개 화면·전쪽 TSV](../assets/issue_8150_kitty/README.md)와 PR 본문의 SHA 고정 이미지/TSV를 대조했다.
로컬 kitty 표 화면은 한글·표 외곽과 이미지 아래 `EXIT=0`·프롬프트가 보인다.
대표 Native review/overlay 라벨은 읽을 수 있고 기존 글꼴·획 굵기 차이가 보인다.
이 보조 비교는 당시 고정한 `2e087a5b...`의 Native 출력 보존 증거이며 최신 devel 재출력으로 표시하지 않는다.
11쪽 raw 최저는 표 96.64066%, 각주 98.95047%, crop 99.95079%, 복합 100%이며 당시 base 대비 하락 0%p다.
새 Native binary의 11쪽 PNG/tree/TSV와 22개 Kitty 스트림이 실제 터미널 검증본과 같음을 재실행으로 확인한 기록도 대조했다.

PNG 독립 무손실과 터미널 캡처를 구분했다. 각주 터미널 화면 2건의 한 픽셀에는 최대 1/255 채널 차이가 있었다.
글꼴 누락은 렌더링하는 원격 호스트에 공개 번들·라이선스를 공급한 후 재출력해 해소했고,
사용자의 「네 이제 한글도 보여집니다」 확인을 Windows 직접 SSH의 최종 표시 근거로 썼다.
Sixel/Kitty 양쪽 매뉴얼의 폴더·개별 파일·반복 옵션·원격 절대 경로·저장 파일 재생성 안내도 확인했다.
완전한 한컴 fidelity·Sixel 대비 속도 향상·미실행 터미널/중계 지원은 주장하지 않는다.

## 최신 base와 후행 기록

검토 중 devel에 #8155가 반영되어 `f7abae4a...`로 전진했다. 공통 표 가로 앵커의 13행 추가/7행 삭제이며
Kitty에서 호출하는 API signature는 바꾸지 않는다. 녹색 code head를 유지하고 rebase/merge하지 않았다.
따라서 리베이스 후 기능 보존 검증은 비해당이며, 기존 출력을 새 base 실행 결과로 바꾸어 기록하지 않는다.
최신 devel의 기존 오늘할일과 source 기록을 보존하며 이번 PR 기록만 다른 section 경계에 추가한다.

후행 commit은 이 self-review·오늘할일·기존 결과보고의 PR 상태만 변경한다.
공통 절차 3.2.1에 따라 정확한 최신 base/trailing head의 merge-tree 종료 0·공백·실제 merge tree의
문서 링크와 기존/신규 오늘할일 보존을 push 전에 검사한다. 검사 신원과 결과는 self-review 최종 상태 보고에 남긴다.
CI 실행 표면 변경이 포함된 PR이므로 새 head의 trusted controller 재사용 판정을 실제 확인한다.
아직 실행 전인 trailing CI 성공을 이 code head의 성공으로 대신하지 않는다.

실행으로 확인한 회귀·코드 검토상 blocker·필수 증거 부족은 발견하지 않았다.
남은 merge 승인·최신 head CI·원격 확인 조건은 이 기록의 승인 판정과 별도로 유지한다.
