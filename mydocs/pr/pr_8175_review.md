# PR #8175 — 양식 캡션의 개체 영역 clipping

## 최종 판정

**머지 보류 — 검증 진행 중.** 최신 devel에서 원 증상과 통합 후보의 실제 Native/fresh WASM 출력을 확인한 뒤 판정을 갱신한다. 현재 상태는 기여 코드의 거부 판정이 아니다.

## 접수와 범위

- 원 PR: https://github.com/edwardkim/rhwp/pull/8175; davindev, CONTRIBUTOR, Open, non-draft, base devel.
- 원 head: `5e0c3562846f16078fa1d4e2e32a7301766197c1` (`kidsnote/rhwp:fix/form-caption-clip`).
- 기준 devel: `2d535febe00a61585b6770cb95860fee912bbe54`; 검토 브랜치 `review/davindev-20261030`.
- 관련 이슈: #8166. 메인테이너 `edwardkim`의 reviewer 지정을 API로 확인했다(2026-10-30).
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, visual_fixture_evidence, multi_pr_update_branch. #8175는 rework_and_exceptions도 적용한다.
- 사용자 승인: 세 PR 검토. #8175의 SVG·Canvas2D 충돌은 최신 기울임꼴·글꼴 임베딩을 보존하고 clipping·대체 목록을 통합하는 메인테이너 보정을 별도로 승인했다. 원격 게시·push·merge는 아직 수행하지 않았다.

## 선행 이력과 검증 계획

#8174의 통합 f125c735c7에서 들어온 공통 기울임꼴과 SVG 글꼴 임베딩을 유지한다. 캡션 clip은 최신 devel에도 없어 원 문제의 실제 출력을 확인한다. 폭 9921/3000/1100HU의 같은 저장본·Print PDF를 Native, fresh WASM, Studio Canvas2D/CanvasKit에서 대조한다.

- 같은 입력/PDF·compat 2022·96dpi·실제 글꼴 환경으로 최신 base와 후보의 전쪽 TSV를 산출한다. 90% 최소값·2%p 초과 하락·쪽수·내용 소유를 확인하고 대표 PNG를 직접 판독한다.
- 원 PR의 기존 GitHub CI는 과거 원 head의 참고 증거다. 메인테이너 보정 통합 후보는 새 코드의 focused·필수 lint·회귀·최적화 WASM 검증을 수행한다.
- 임시 로그·원격 metadata: `output/pr-review/davindev-20261030/`. 수용에 사용하는 원본·PDF·최종 TSV·대표 PNG는 정식 경로에 보존한다.

## 통합 후보의 로컬 검증 (2026-10-30)

공통 `form_drawing`의 양식 bbox·caption 원점을 그대로 소비하고, SVG·WebCanvas·Native Skia·
Studio CanvasKit 각각에서 같은 개체 bbox를 clip으로 적용한다. 캡션과 primitive의 배치 수치를
변경하지 않으며 save/restore 또는 그룹 종료로 뒤 개체에 clip이 누출되지 않게 한다.
SVG 충돌에서는 `font_codepoints` 등록을 유지했고, Canvas2D와 함께 공통
`text_italic::CSS_STYLE`을 유지했다. 기존 devel의 글꼴 기능을 삭제해 충돌을 푼 것이 아니다.

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
  관련 회귀 26건은 통과했다. 전체 회귀·Native Skia 필수 검사는 진행 중이다.

공유 Cargo 캐시가 한 번 기준 바이너리를 후보 빌드에 재사용한 것을 바이너리 SHA 대조로
발견했다. 그 실행은 무효로 분리했다. 후보 root crate의 실제 재컴파일 뒤 Native SHA
`8114f5e93b0c544a66c7dad8bfe2a533691a9a5d5568c49a71b55915e28503a0`, WASM SHA
`ce942e2d6e3d37dc4fd593cae55423b5b9303f510405d868a91c57bf39196f8d`에서 위 결과를 다시 확인했다.

Print와 다른 기존 체크 상자 크기·라디오 외곽선 일부 및 글자 획 차이는 남아 있다.
수정 전후 확대 비교에서 이 차이는 기준에도 존재하고, 후보에서 새로 생긴 차이로 기록하지
않는다. 이 PR의 개선은 캡션의 개체 영역 제한이며 양식 appearance 전체의 Print 일치를
완료했다고 주장하지 않는다. 함초롬돋움 원본을 추가 공급한 글상자 진단도 기존 Native
픽셀과 같았으므로 남은 획 차이를 해당 글꼴의 단순 누락으로 단정하지 않는다.

2026-10-30 작업지시자는 남은 회귀 통과 뒤 원격 push와 머지를 승인했다. 통합 후보의
로컬 필수 검사와 원격 최신 head CI를 완료한 뒤 기여 이력을 보존하는 merge로 처리한다.

## 조판 원칙과 소비 경로 self-review

원본의 양식 폭(HWPUNIT) → 공통 배치 `FormObject` bbox → 공통 `form_drawing` →
SVG/WebCanvas/Skia/CanvasKit의 같은 bbox clip을 확인했다. 잘못된 조판 좌표를 숨기는
보정이 아니라 독립 Print에서 확인한 컨트롤의 그리기 경계다. backend별 clip API는 다르지만
경계를 각각 추측하지 않고 공통 bbox를 소비하며 save/restore가 다음 개체에 영향을 주지 않는다.
SVG의 `font_codepoints`와 공통 italic 스타일은 충돌 보정 뒤 유지한다.
폭 1100/3000/9921HU 원본·독립 Print와 합성 폭 계약을 구분했고, 기존
`form_classic_appearance` 검사와 함께 통과했다. 원본 단위에서 환산한 폭 계약을 유지했고
실물의 잘못된 절대 좌표나 golden을 추가하지 않았다. 페이지 분할·이어받기는 변경하지 않는다.
