# PR #8134 — 저장 줄 없는 글상자의 세로 정렬

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel에서 원 증상과 통합 후보의 실제 Native/fresh WASM 출력을 확인한 뒤 판정을 갱신한다. 현재 상태는 기여 코드의 거부 판정이 아니다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8134; davindev, CONTRIBUTOR, Open, non-draft, base devel.
- 원 head: `77d5673d533ee3cc43666ef824200b74207ad8a3` (`kidsnote/rhwp:fix/no-ls-textbox-valign`).
- 기준 devel: `2d535febe00a61585b6770cb95860fee912bbe54`; 검토 브랜치 `review/davindev-20261030`.
- 관련 이슈: #8122. 메인테이너 `edwardkim`의 reviewer 지정을 API로 확인했다(2026-10-30).
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch. #8175는 rework_and_exceptions도 적용한다.
- 사용자 승인: 세 PR 검토. #8175의 SVG·Canvas2D 충돌은 최신 기울임꼴·글꼴 임베딩을 보존하고 clipping·대체 목록을 통합하는 메인테이너 보정을 별도로 승인했다. 원격 게시·push·merge는 아직 수행하지 않았다.

## 선행 이력과 검증 계획

#8134가 #8147의 조상이다. 이전 메인테이너 지적은 앞 간격을 측정에서 별도로 합산해 실제 배치와 어긋난 문제였고, 기여자가 a8a86c106a에서 실제 배치의 줄 끝을 소비하도록 재작업했다. 최신 head를 대상으로 원 사례와 앞 간격·빈 문단·위/가운데/아래 정렬 경계를 확인한다.

- 같은 입력/PDF·compat 2022·96dpi·실제 글꼴 환경으로 최신 base와 후보의 전쪽 TSV를 산출한다. 90% 최소값·2%p 초과 하락·쪽수·내용 소유를 확인하고 대표 PNG를 직접 판독한다.
- 원 PR의 기존 GitHub CI는 과거 원 head의 참고 증거다. 메인테이너 보정 통합 후보는 새 코드의 focused·필수 lint·회귀·최적화 WASM 검증을 수행한다.
- 임시 로그·원격 metadata: `output/pr-review/davindev-20261030/`. 수용에 사용하는 원본·PDF·최종 TSV·대표 PNG는 정식 경로에 보존한다.

## 통합 후보의 로컬 검증 (2026-10-30)

조판 경로: `shape_layout.rs:3622`에서 저장 줄이 없는 Center/Bottom 경로를 고르고,
3729 이후 Top 흐름으로 배치한다. 3789의 실제 자식 노드 끝이 내용 높이가 되어 3794에서
남는 공간을 계산한다. 3801의 subtree 이동과 3803의 문단 테두리 범위 이동,
3819 이후 inline 개체 위치 등록의 이동이 같은 `vert_offset`을 사용한다.
3837 이후 지연 배치하는 inline 개체도 그 오프셋을 소비한다. 저장 줄이 있는 경로는 기존
저장 위치 계약을 유지한다. 앞 간격 600HU·빈 문단·Top/Center/Bottom 변형의 정렬 불변식도 통과했다.
페이지 분할·이어받기 알고리즘은 변경하지 않는다.

코드 head `770b073d24085b056c8a0f4263e2214b8b53ae08`에서 검증했다. 이후 새 테스트의
첫머리에 실측 근거 주석을 추가했으며 구현·assertion은 바꾸지 않았다.

- [전쪽 TSV·직접 비교·출력 출처](assets/davindev_20261030/README.md): 5개 원본, 각 1쪽. Native/fresh WASM 10건 모두 90% 이상.
- 관련 focused 회귀 24건 통과: #8122, #8118, #8166, #7642, #8168, #7062, 기존 양식 appearance.
- Studio 단위 검사 2,673건 통과, 2건 skip, 실패 0건. TypeScript 검사와 renderer-contract E2E 통과.
- Canvas2D·CanvasKit compat/default의 screen/print 18회 실제 캡처, 비교 12건 통과.
- Native·WASM 최적화 빌드 완료. SVG 글꼴 임베딩과 공통 italic 정책을 보존했다.
- 전체 Rust 회귀 12,071건 통과, 45건 skip, 실패 0건. 실행 2,252.895초(컴파일 별도).
- Format·Native/WASM Clippy·workspace build·workspace all-targets Clippy 통과.
- 측정 주석 추가 뒤 파생 harness drift는 `--prepare`로 재생성하고 manifest 검사를 통과했다. 같은 source 전체를 위 회귀에서 실행했다.
- 이전 후보의 Native Skia lib: rhwp 3,858건 및 하위 라이브러리 182건 통과, 12건 ignored.
- 검증 도중 #8389·#8390이 devel에 통합되어 기준이 `cb9c45cf774a03eddb5d56fdb1fd28919f10a969`로 바뀌었다. 뒤의 Native 그림·PDF 검사는 완료하지 않고 이전 후보용 빌드를 중단했다.
- 새 기준과의 merge-tree는 충돌 없이 생성된다. 위 결과는 이전 기준의 역사 기록이며, 최신 기준의 완료 판정을 대신하지 않는다. 최신 devel 반영 뒤 영향 경계·출력·필수 검증을 확인한다.
