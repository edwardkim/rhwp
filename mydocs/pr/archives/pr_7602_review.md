# PR #7602 리뷰 — 보안 입력·출력 경계 통합

## 최종 판정

**승인 — 다섯 보정 범위의 통합 source와 독립 경계·정상 대조 검증이 충족됐다.** 원격 병합 전에는 최신 제출 head의 모든 CI 완료·required Build & Test 성공·현재 devel과의 충돌 부재를 재조회한다. 공개 advisory·릴리스·신고 종료는 이 판정의 범위가 아니다.

## 접수와 경로

- PR #7602 / edwardkim / base devel `e8cc27778011ce8b54e823fa7edb446b979304a6`.
- 로컬 검증 code head `652057732e0efaaba522f11544c7e540c550efcc`; 문서·PNG 제출 head `cb97c7a9a9585f6c00bf51bb2446e6c41c18c07c`.
- 작성자 self-review이며 reviewer를 assign하지 않는다. 사용자 지시로 devel 코드 병합을 진행한다.
- 기본 경로 collaborator_self_merge; intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch, rework_and_exceptions, post_merge를 보조로 읽었다. 1,000줄 초과이므로 별도 통합 검토·simulation·전체 로컬 검증 뒤 CI를 확인하며 admin 우회하지 않는다.
- 준비된 다섯 source commit을 같은 base에서 순차 merge했으며 SVG 중복은 한 번만 반영했다. 자동 merge된 SVG/Safari 소비 코드를 직접 확인했다.
- 다섯 source 통합 뒤 source/test/config는 변경하지 않았다. 이후 commit은 문서와 PNG뿐이며 tree 비교로 source 동일성을 확인한다.

## 근거와 실제 결과

[검증 기록](../../plans/security_patch_integration_20261006.md)에 독립 계약·소비 경로·source/base SHA·명령·입력 hash·출력 PNG·미검증 범위를 연결했다. Native 생성 입력과 CDP 입력의 byte identity, 기존 HWP3 저장본과 commit의 byte identity를 확인했다.

fmt/Clippy 3개·workspace build·고정 base manifest/unit tier PASS, 전체 nextest 10,508 PASS/0 FAIL/50 skip, 보안 전용 17/17 PASS, Native Skia lib 4,109 PASS/13 ignore와 placeholder 2/2/PDF 4/4, JS 및 fresh WASM/CDP PASS다. 실제 원시 로그/관측 JSON은 ignored output에 보존하며 PR에 포함하지 않는다.

공통 조판 원칙 중 줄 소속·측정/배치·pagination/continuation·baseline 변경은 비해당이다. 조판 규칙과 저장 LineSeg를 변경하지 않았다. SVG 직렬화와 입력 resource policy는 최종 export 및 DOM/CSSOM으로 검사했다. Print/Screen 각각 12 geometry가 Native와 같고 동일 입력의 PNG 차이는 0이며 직접 판독했다. 합성 TTF의 A/가 glyph 직사각형은 의도한 모양이다. 한컴 PDF 조판 일치·90% Visual Sweep 승인으로 표현하지 않는다.

SVG CSS/XML 경계, mapped IPv6, HWP3 재귀 정책, raw DEFLATE 호환성, 연결 그림 실제 형식/XML budget의 적용·비적용 경로와 실패/정상 대조를 검증했다. Safari macOS 실기기·inner password fresh WASM 등 미검증 범위는 검증 기록대로 유지한다. 테스트 허용치·golden 변경은 없다.

## 병합과 후속 처리

최신 remote base/head의 merge-tree proof와 정확한 head CI를 확인한 뒤 일반 gh merge로 통합한다. 보호 규칙 우회, 강제 push, 태그·릴리스 생성은 하지 않는다. Merge SHA 확인 후 clean 기본 devel을 fast-forward하고 자동 duration 갱신의 성공 또는 증거 부족에 따른 보류를 확인한다. 전체 CI를 병합 뒤 수동 재실행하지 않는다.

PNG는 PR 본문에서 정확한 제출 head의 실제 Markdown 이미지로 표시한다. 코드 head 변화 시 재검증하고 다시 캡처하며, 문서-only trailing head에서는 source 동일성·PNG 동일성을 확인한 뒤 본문 URL을 고정한다. 비공개 후속 검토에 쓰는 기존 worktree와 private fork는 보존한다.
