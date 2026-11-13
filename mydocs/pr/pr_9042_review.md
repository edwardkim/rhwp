# PR #9042 리뷰 — 같은 문단의 제목·본문 표 배치

## 최종 판정: 메인테이너 보정 후 수용 가능

**승인된 로컬 보완과 필수 검증을 완료했다. 기존 차이 예외는 적용하지 않았다. 원격 push·새 PR·공개 review/comment·merge는 승인 대기다.**

원 기여의 같은 문단에 앵커된 제목·본문 표가 3쪽으로 밀리는 문제를 재현했고 2쪽으로 개선됨을 확인했다. 메인테이너가 Header/Footer 경계, 실제 저장 중첩 표의 여백, 원 합성본의 오래된 저장 줄과 실제 편집·반복 저장까지 보완했다. [상세 보완과 소비 경로](pr_9042_review_impl.md), [최종 전쪽·빌드·검사 원장](assets/pr9042/final-validation/README.md).

## 소스·권한

- [원 PR #9042](https://github.com/edwardkim/rhwp/pull/9042), puckqook의 첫 rhwp 기여. 직접 연결된 해결 이슈는 없다.
- 원 head `3ae4eeb332c3b0edbb3a8fbe334daf2b0a48115b`, 원 base `180a4ec97205b67c4dc4f90416063c01ecfdf396`. 원 PR의 base ref는 devel이며 API의 오래된 base SHA로 최신 devel을 대체하지 않았다.
- 검증 제품 `d66d5f05f9efa4d182228494cd970614d479fb7a`, 반영한 최신 devel `959300e66c58241a47e6d46030b4b0854e59604b`. 작업 branch는 `review/pr-9042-coanchored-table-pagination`이다.
- 원 기여를 적용한 `637410dbfa`와 메인테이너 보정을 분리했다. 원 ref·작성자뿐 아니라 원 SHA 자체도 조상으로 보존했다. 이미 반영한 원 기여를 이력에 연결한 `495787589a` 전후 tree는 같다. [실제 이력 연결 증거](assets/pr9042/final-validation/original-ancestry-preservation.json).
- fork `puckqook/rhwp`의 `push=false`를 확인했다. upstream 수용 PR을 준비하며 원 fork branch는 보존한다. 현재 원격 mutation은 접수 때 승인받아 지정한 reviewer edwardkim뿐이다.
- 기본 maintainer_general과 보조 intake_and_review·local_validation·visual_fixture_evidence·post_merge·first_time_contributor를 적용했다. 최초 사전 조회/fetch가 reviewer 지정 전에 실행된 순서 누락도 접수 기록에 보존했다.

## 조판 원칙·원인과 공통 결과

| 입력과 원인 | 공통 생산 → 측정·fit·배치·저장 소비 | 판정 |
| --- | --- | --- |
| Header/Footer는 본문 흐름을 밀지 않는 제어인데 공동 host 판정만 거절 | `stored_inline_host_has_separate_float_frame` → stored first line/whole-fit → float 원점·예약·paint | 같은 비본문 분류와 실제 저장 2쪽 확인 |
| 실제 저장 중첩 TAC의 최소 선언과 닫힌 상자 높이를 같다고 가정 | `table_row_frame`의 내용·줄 사다리·여백 포함 증명 → 공통 tail fit → format·예약·paint | 실제 한컴 저장 3종으로 발동, span·rewind·폭 미입증은 기존 경로 |
| RHWP 생성 원본은 현재 글자 크기 650 HU와 달리 1900/1100 HU 저장 줄·과거 문단 전진을 보유 | `stored_text_metrics` 관리 캐시 생산 → 렌더 사본·최초 유효 셀 편집·저장 사본 | 32개 의도적 빈 문단을 유지하며 실제 한컴 재저장의 650/553 HU와 전진을 확인 |
| 같은 줄의 글자 편집도 세션 전체 편집 플래그로 제목 원점을 버리고 저장 높이는 원시 행 합을 사용 | 공통 닫힌 끝 행 → format·whole-fit·예약·paint·grown-table writeback | 실제 줄 재배치·내용 하한 검사는 유지, BodyX 화면·저장 모두 2쪽 |
| 최신 #9054 그림 자원 정리와 저장 예상 문서의 연결 | 같은 `with_hwpx_export_document` 사본에서 캐시·marker·pruning을 수행 | 삭제 그림 제외, 원본 미참조 자원·글머리표·복원은 보존 |

저장 줄 수용은 실제 BodyText digest가 관리 marker와 일치하는 구역의 단순 한 줄 셀에 한정한다. 이것은 소유 확인이며 입력의 신뢰·정확성 증명이 아니다. 정상 큰 strut·현재 전진과 맞는 캐시·일반 한컴 저장본은 기존 경로를 유지한다. 문서 ID·목표 px·임의 높이 상수·출력 숨김·잘못된 좌표 clamp로 맞추지 않았다. 읽기·렌더·저장만으로 live IR을 바꾸지 않으며 유효한 첫 셀 편집 때만 같은 메트릭을 인계한다. 빈/잘못된 경로는 readonly target 조회에서 거절된 뒤 mutable 접근하므로 대상 확인 전 인계하지 않는다.

실제 적용 경로는 [보완 기록](pr_9042_review_impl.md), 입력 성격과 수정 전 실패는 [초기 진단](assets/pr9042/README.md), [저장 줄 추적](assets/pr9042/source-line-cache/README.md), [중첩 표 증명](assets/pr9042/native-nested-validation/README.md)을 따른다. 원 해결 #8187의 공통 tail 계약과 #8682 보호를 추적하고 기대값부터 바꾸지 않았다.

## 실제 검증 결과

| 검증 | 최종 결과 |
| --- | --- |
| 목표 8종 전 16쪽, 독립 Print | Native·fresh 최적화 WASM 각각 최저 100.00%, 쪽수 일치 |
| BodyX 실제 편집 전 2쪽 | 양쪽 전쪽 100.00%, 표 프레임·내용 소속·순서 보존 |
| 대조군 포함 각 22종·40쪽 | Native 최저 90.01% / WASM 97.08%, 새 base 기능 포함 |
| 정본 보호 3,542쪽 | 공식 두 자리 표시값 하락 0건, 예외 없음 |
| 별도 Native 편람 383쪽 | handbook gate 통과 |
| 최소 글꼴 전체 회귀 | 13,473 PASS / 0 FAIL / 45 설정상 skip |
| Native 공식 3종 | 4,073 PASS, lib의 12 ignored는 성공에 합산하지 않음 |
| 집중 회귀 | 48 PASS, 이번 원본·경계 모듈 9건 포함 |
| fmt·Native/WASM32/workspace Clippy·workspace build·suite | 전부 exit 0 |
| 공개 무편집 저장 | CLI 18건은 strict IR 차이 0·2→2쪽, 브라우저 18건은 내용·쪽수·재열기 보존 |
| 실제 편집과 반복 저장 | HWP/HWPX 12건 + 별도 엔진 snapshot 복원 4건, 두 번 저장·재열기·내용 보존 |
| 직렬화 결과의 시각 재검증 | 무편집 16개 입력/PDF 조합(14개 byte 그룹)·양쪽 64쪽, 편집 2개 그룹·양쪽 8쪽 모두 100.00% |
| 검증 입력·산출물 보안 | 입력 30개 × 3종 탐지 90건 clean |
| clipping·실제 SVG 소비 | 대상 16종 회귀 0; SVG 원본·편집 모델 전 4쪽 최저 99.60% |

회귀 기대값·baseline을 완화하거나 검사를 이관하지 않았다. 새 7개 sample의 oracle 쪽수만 독립 2쪽 근거로 등록했으며 비영 래칫 허용치를 추가하지 않았다. 전체 suite의 IR·overflow·off-canvas·overlap·body overflow 검사도 통과했다. 가용 입력이 없는 기본 clipping 92종은 통과로 세지 않았다. 이전 shared target의 base 산출물을 잘못 fresh로 재사용한 진단 실행은 제외하고 실제 후보를 재빌드한 producer hash를 기록했다.

정식 회귀는 최소 글꼴 sandbox에서 실행했다. Native 시각 측정과 WASM에는 같은 HCR 프로그램을 공급하고 실제 loaded SHA를 확인했다. 초기 WASM이 Noto 대체를 쓴 캡처와 기존 설명을 정정했으며 그 캡처를 현재 완료 증거로 재사용하지 않는다. font 바이너리는 커밋하지 않았다.

## 직접 판독·입력 확인

원본, Header/Footer, 실제 한컴 저장 중첩, BodyX의 전체 페이지를 확인했다. 같은 영역을 독립 Print·Native·WASM 실제 384 DPI로 다시 출력해 제목·End of body의 하강부·괘선·Body end·후속 페이지의 포함·순서와 누락·겹침을 판독했다. [확대와 실제 PNG 해시](assets/pr9042/final-validation/live-edit/actual384-provenance.json). 원본의 두 페이지 flagged_pages는 양쪽 모두 0이며 [대표 review/overlay](assets/pr9042/final-validation/README.md)에 pixel·ink/proxy·raw를 구분했다. raw 100%를 RGB 동일로 해석하지 않는다.

입력은 공개 RHWP 합성 원본, 실제 한컴 2020 재저장·Print, 커밋된 대조군이다. 생성 경로·실제 실행 파일·독립 PDF·해시는 [입력 원장](assets/pr9042/README.md)에 연결했다. 공개 저장의 실제 byte 그룹을 커밋하고 실행 별칭을 기록했다. BodyX 모델은 RHWP 작성 모델을 한컴에서 Print한 것으로 한컴 SaveAs라고 기록하지 않는다. 현재 source의 본문 snapshot 검사는 엔진 API이며 Studio UI Undo/Redo E2E와 구분한다. 실제 수직 재조판·span·이어받기는 해당 비적용 조건과 기존 분할 경로를 유지하며 임의의 장문 편집 전체를 이번 수용 효과로 확대하지 않는다.

장시간 WASM 감독 도구 세션은 자식 명령의 모든 exit 0 뒤 외부 종료 코드 143을 보고했다. 이를 성공 코드로 치환하지 않았고, [독립 감사](assets/pr9042/final-validation/visual-chain-audit.json)에서 개별 전쪽 결과·입출력·strict 저장·패키지 해시를 짧은 별도 실행으로 다시 확인했다. Native·전체 회귀·lint를 수행한 필수 gate driver는 최종 exit 0이다.

## 원격 처리와 남은 절차

원 head CI·CodeQL의 Rust 분석 성공은 접수 근거다. 종합 neutral의 미실행 언어 구성 안내를 오류나 새 취약점으로 판정하지 않았으며 경고 분류·재실행은 하지 않았다. 이 원 CI를 메인테이너 새 후보의 CI 성공으로 재사용하지 않는다.

원격 승인 후 upstream 수용 branch push → devel 대상 Open PR → exact-head 필수 CI 성공·self-review → 원 기여를 보존하는 merge → 원 PR 완료 확인·존댓말 감사 댓글·운영 기록·동기화·종료 정리 순서다. 아직 해당 원격 동작은 실행하지 않았다. 직접 연결된 해결 이슈는 없으므로 관계없는 이슈를 닫지 않는다.
