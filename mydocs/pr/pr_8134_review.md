# PR #8134 — 저장 줄 없는 글상자의 세로 정렬

## 최종 판정

**메인테이너 보정 후 수용 가능 — 로컬 필수 검증 완료.** 통합 후보의 원격 최신 head CI를 확인한 뒤 승인된 merge를 진행한다. 원 기여 커밋의 이력은 모두 보존한다.

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

## 최신 devel 반영 뒤 검증 (2026-10-30)

기준은 `cb9c45cf774a03eddb5d56fdb1fd28919f10a969`, 제품 소스는
`d189352a1b3b0e09718022f7da47daaa5205fab9`다. 로컬 `d94c7aa124`는 실측 주석과
증거 문서·TSV만 갱신했으며 assertion과 제품 코드는 바꾸지 않았다.

- [새 기준의 전쪽 TSV](assets/davindev_20261030/rebase-cb9c45/README.md): 원 사례 5개, 각 1쪽.
  후보 전쪽 최저 Native 96.21%, fresh WASM 95.98%; 독립 PDF와 전체 쪽수 일치.
- 글상자는 Native 85.19→99.55%, WASM 85.70→99.42%다. Native에서 제목 영역 밖의
  본문·그림 픽셀은 기준과 동일하다. 그림 단일 출력·소유 줄 내부·후속 본문 순서도 보존한다.
- 좁은 캡션은 Native 88.77→96.21%, WASM 88.15→95.98%다. 넓은 캡션 대조군의
  하락은 각각 0.14%p·0.15%p로 2%p 기준 이내다. 실제 1100/3000/9921HU에서
  캡션 숨김/경계 잘림/전체 표시를 Native·WASM·Studio의 같은 영역에서 직접 확인했다.
- SVG 5건의 실측도 90% 이상이며 임베딩 글꼴 2종의 Unicode cmap 검사를 통과했다.
- Studio TypeScript, 단위 검사 2,692건(2건 skip), renderer 계약·CanvasKit 글꼴 E2E 통과.
  실제 Canvas2D·CanvasKit compat/default의 screen/print 18회 캡처와 비교 12건을 완료했고
  runtime 오류·hard gate 위반은 0건이다. CanvasKit은 software 경로다.
- #8389 보존: 편람 HWP/HWPX 각각 383쪽의 Native·WASM 총 1,532쪽은 최신 기준과
  모두 바이트 단위로 같다. 전쪽 최저 99.96%, 독립 PDF 쪽수 일치, 130·205쪽 확대 판독 완료.
- Format·Native/WASM Clippy·workspace build·workspace all-targets Clippy·manifest와
  관련 회귀 26건은 통과했다. 전체 Rust 회귀 12,198건 통과(45건 skip, 실패 0건),
  실행 524.345초다. Native Skia lib 4,040건 통과(12건 ignored), 그림 누락 2건과
  직접 PDF 출력 14건도 모두 통과했다.

공유 Cargo 캐시가 한 번 기준 바이너리를 후보 빌드에 재사용한 것을 바이너리 SHA 대조로
발견했다. 그 실행은 무효로 분리했다. 후보 root crate의 실제 재컴파일 뒤 Native SHA
`8114f5e93b0c544a66c7dad8bfe2a533691a9a5d5568c49a71b55915e28503a0`, WASM SHA
`ce942e2d6e3d37dc4fd593cae55423b5b9303f510405d868a91c57bf39196f8d`에서 위 결과를 다시 확인했다.

2026-10-30 작업지시자는 남은 회귀 통과 뒤 원격 push와 머지를 승인했다. 통합 후보의
로컬 필수 검사와 원격 최신 head CI를 완료한 뒤 기여 이력을 보존하는 merge로 처리한다.

## 조판 원칙과 소비 경로 self-review

저장 줄이 없는 Center/Bottom 글상자의 정렬 대상 높이는 실제 위쪽 배치 결과의 끝을 포함한다.
`shape_layout.rs:layout_textbox_content`의 `inner_area` → 실제 문단 배치 → `flow_bottom` →
남는 공간의 정렬량 → `translate_subtree_y` 순서로 소비한다. 문단 테두리 band와
inline shape 등록 좌표에도 같은 이동량을 전달한다. 저장 LineSeg가 있는 경로는 기존 계약을 유지한다.
원본과 독립 Print에서 두 줄의 상자 내부·정렬을 확인했고, 앞 간격·마지막 빈 문단·Top/Center/Bottom
변형은 정렬 불변식으로 별도 검사했다. 문서 ID나 시각 점수에 맞춘 분기는 추가하지 않았다.
해당 원본을 사용하는 기존 검사는 이 관계형 모듈이며 절대 px 핀 교체는 없었다.

## 제출 전 최종 상태

- 검증 제품/테스트 소스: `d94c7aa124`; 제품 출력 소스 `d189352a1b`와의 차이는 실측 주석뿐이다.
- 이후 로컬 변경은 mydocs 증거·검토 기록으로 한정된다. 코드·assertion·입력은 바꾸지 않았다.
- 중단된 전체 회귀 컴파일은 성공으로 세지 않고 로그를 분리했다. 재개 실행에서 전체 회귀와
  Native Skia 세 필수 항목의 exit 0을 확인했다. 이미 통과한 lint·Studio·시각 검증은 재사용했다.
- 제출 브랜치는 현행 명명 규칙에 맞춰 `review/integration-20261030-davindev-rendering`으로
  이름만 정렬했다. 기존 `review/davindev-20261030`과 같은 이력이며 rebase·코드 변경은 없다.
- 신규 PR 선행 조회에서 본인 PR #8392의 CI 진행을 확인했다. 실패·충돌 여부를 관찰하며
  다른 작업의 branch·source를 임의로 보정하지 않는다.
- LFS 신규 객체 없음, 문서 링크·입력 및 증거 해시 확인, devel merge-tree 충돌 없음.
