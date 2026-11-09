# PR #8851 검토 — chrisryugj HWP3 PR 2건 통합

## 최종 판정

**승인.** 원 기여 head를 보존한 `9fa72c60365f6b1f3f168223e7ac56d2fa4b6402`에서 필수 로컬 검증·최신 CI·self-review를 완료했고, 2026-11-09 KST merge `50b6098853afdb45625e5d505dc3b2070998fc46`으로 통합했다. SO-SUEOP의 기존 조판·Native 긴 줄표 fallback 차이에 한정한 메인테이너 예외를 기록했다. 새 셀 누락은 예외로 면제하지 않고 보정했다.

## source와 원 기여 보존

- 기준 devel: `b768bee1c4c0023087687f79a0a95f12108743e9`.
- 원 #8789: `3c2c5fdf1bea13912b2eefaf4008ea974a2b4fd9`. [개별 판정](pr_8789_review.md).
- 원 #8791: `383f66acca8c966da9a198fd046c3da0ed94c3b6`. [보정 이유·개별 판정](pr_8791_review.md).
- 공통 이웃 색인 보정: `4fabb47ae28b99d371e2aac3560e2610e327c311`. 최종 테스트 source: `581d22855f60af2de5fed3208949f7dd81258a0a`. 이후 변경은 문서뿐이다.
- 실제 CI tested merge `98c67a8a7e017b9e403c1452f041c2e6750dfc9a`와 최종 merge의 tree는 모두 `b4cf97bbbdf398598464b027953d61fc0e2891fe`다. fresh-base merge-tree·MERGEABLE/CLEAN과 required Build & Test·CI Impact Policy를 확인한 뒤 exact head로 병합했다.

원 #8789·#8791은 모두 자동 MERGED다. contributor commit과 fork branch를 보존했고 upstream 임시 통합 branch를 사용했다. GitHub 자동 종결 연결은 없다. 실제 관련 이슈는 [원격 후속 점검](../assets/chrisryugj-20261109/remote-completion.md#관련-이슈-종료-여부-추가-점검)에 별도로 기록했으며 #7807·#7445는 미완료로 OPEN을 유지한다. 첫 기여자 절차는 기존 기여 #8224·#8233에 의해 비해당이다.

## 검증과 self-review

- 로컬 전체 회귀 13,013 PASS·기존 45 skip, Native 공식 4,077 PASS·기존 12 ignored, 집중 8 PASS. 최소 글꼴 환경과 명령은 [원장](../assets/chrisryugj-20261109/README.md)에 있다.
- Rust format·기본/WASM32/workspace all-target Clippy·workspace build·suite 정책 통과. Native 편람 383쪽은 raw 최저 99.96%·기준 대비 회귀 0이다.
- [CI](https://github.com/edwardkim/rhwp/actions/runs/42601579278) attempt 1에서 A/B/C/D/E 모두 성공했다. 3,801/1,776/2,199/2,455/2,588건, 합계 12,819 PASS·기존 45 skip이며 집중 8건도 실제 실행됐다. 로컬의 추가 194건은 CLI binary 단위 검사 12건과 workspace 라이브러리 182건이다.
- [CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42601579257)·[Adapter](https://github.com/edwardkim/rhwp/actions/runs/42601579324)·[Proptest](https://github.com/edwardkim/rhwp/actions/runs/42601579260)·[최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42605435017) 성공.
- CodeQL 실제 Rust 분석 `2125715656`의 오류·결과 및 PR ref의 열린 보안 경고는 0건이다. Python·JavaScript/TypeScript·Actions 분석은 경로 정책으로 skip됐다. GitHub의 별도 CodeQL check는 이 구성 생략을 neutral로 안내했다. 해당 파일의 base 대비 diff는 0이며 네 언어를 모두 분석한 것으로 표현하지 않는다. 보안 탐지 설정·경고 분류는 바꾸지 않았다.
- 현행 분류는 `mode=selective;rust=1;fe=none;render=0;skia=0;ql=rs`다. Frontend·Native Skia·별도 Render Diff를 CI 실행 성공으로 표시하지 않는다. Native/fresh WASM의 실제 전쪽 검증과 Native 공식 검사는 로컬에서 완료했다.
- [최종 self-review](https://github.com/edwardkim/rhwp/pull/8851#pullrequestreview-5824947905)는 정확한 head의 공개 review comment이며 게시 뒤 API 원문·실제 브라우저 한글 렌더링을 확인했다.

## 조판 원칙·입력·시각 수용 범위

적용 대상이다. HWP3 문자 → IR·본문/각주 저장 및 셀 geometry → 공통 이웃 검색 → row/col/span·겹침 해소 → layout·저장·paint의 실제 소비를 대조했다. 두 축은 같은 검색을 사용하고 문서 ID 예외·좌표 clamp·내용 숨김을 추가하지 않았다. 새 continuation·pagination 분기는 없다.

검증 원본·독립 PDF·독립 한컴 변환본은 검토 commit에 이미 포함된 파일을 재사용했고 실제 SHA-256과 전쪽 TSV를 원장에 보존했다. 원본 111쪽씩의 Native/fresh WASM을 base/후보로 대응시켰다. 표 원본 65쪽은 양쪽 전쪽 90% 이상이며 페이지별 하락은 없다. 원 네 표와 문장부호의 같은 영역을 직접 확대 판독했다. 큰 합성 입력은 parser 계약 증거와 한컴 Print 증거를 구분한다.

SO-SUEOP의 기존 굵기·열 흐름과 독립 HWPX/base 대조에서 재현된 Native 긴 줄표 간격은 메인테이너가 명시적으로 승인한 한정 예외다. 새 누락·미실행 검사에는 확대하지 않는다. [전쪽 수치·직접 판독·제한](../assets/chrisryugj-20261109/so-sueop-visual-hold.md)을 확인한다.

## 후속 기록과 종료 게이트

후속 기록 방식은 maintainer 직접 반영(option M)이다. CI 실행 정의·판정 정책 변경이 없어 옵션 2 강제 대상은 아니다. 검토 archive·오늘할일·이 원격 확정 기록만 devel에 반영한다.

[실행 시간 메타데이터 갱신](https://github.com/edwardkim/rhwp/actions/runs/42605953144)은 성공했다. 이번 PR CI의 B/C/D/E attempt 1 실측으로 `ready=true`·`published=true`이며 `ci-metrics/nextest-target-durations`에 commit `c1c416d805795ae4ba910c37c5843aa007e1953a`를 게시했다. source merge는 `50b6098853afdb45625e5d505dc3b2070998fc46`이다. 병합 후 제품 검증 CI를 다시 실행하지 않았다.

운영 기록 push·devel 동기화 뒤 asset이 실제 devel에 있는 것을 확인하고 원 PR 2건에 감사 댓글을 게시한다. 마지막으로 이 작업의 임시 upstream branch·local worktree/branch/ref, 루트 output 전체와 사용 종료된 공유 debug를 정리한다. 실제 게시 URL·정리 결과는 `target/pr-review/cleanup-pr8851-20261109.json`에 남긴다. 사용자·다른 작업의 source/WIP·worktree·local Docker 소비 환경·설치 글꼴·Studio JS/WASM·다른 target profile은 보존한다.
