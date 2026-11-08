# #8183 1단계 — 이슈 등록과 opt-in 설계

Issue: [#8183](https://github.com/edwardkim/rhwp/issues/8183)

상태: **설계·계획과 source 조사 완료. 새 코덱 기능의 구현·실행 검증 전이다.**

- 메인터너의 2026-10-23 지시에 따라 코덱별 opt-in과 빌드·사용자 가이드 보강을 이슈로 등록했다.
  `edwardkim` 담당, `v1.0.0` milestone, `pdf`·`rust`·`enhancement`·`documentation` label을
  지정하고 API로 본문·한글·담당·metadata를 다시 확인했다. milestone은 0.9.0 포함 시점의 확정이 아니다.
- 열린·닫힌 이슈를 검색하고 #7959 → #7970 → 실제 통합
  `515bd72dcde0eecdeece333666b675354b7fa91d`의 독립 출력·자원 검증과 현재 base 포함을 확인했다.
  기존 mask/색 공간 보정을 새 codec 지원 완료로 분류하지 않았다.
- 최신 `upstream/devel` `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867`에서
  `feat/issue8183-pdf-codec-opt-in-20261023`을 생성했다. 본인 열린 PR은 없었다.
- [수행 계획](../plans/task_m100_8183.md)과 [구현 계획](../plans/task_m100_8183_impl.md)에
  네 feature 조합·실제 handle/codec/mask/PNG 소비 경로와 사전 할당·누적 작업량 경계를 기록했다.
- 현재 본체의 metadata와 manifest에서 제안한 `pdf-jbig2`·`pdf-jpeg2000`이 없음을 확인했다.
  `cargo tree --locked --offline -p rhwp --depth 0`은 exit 0,
  여기에 각각 `--features pdf-jbig2`, `--features pdf-jpeg2000`을 추가하면
  해당 feature 부재로 각각 exit 101이었다. 이는 decoder의 결함 재현이 아닌 현재 빌드 선택의 부재 확인이다.
  `cargo metadata --no-deps`는 미정의 feature 인자를 거절하지 않아 feature 검증 근거로 쓰지 않았다.
- 제품의 기존 사전 픽셀 검사와 후처리 계약은 유지하면서 실제 decoder 내부의 예약·반복을
  제한해야 함을 구분했다. JPX 해상도 hint와 handle의 실제 크기 대조, codec source mask의 별도
  sample adapter도 구현 검토 범위에 포함했다.
- 빌드·CLI·패키지 README·신규 사용자 가이드의 수정 내용을 계획했다.
  현재 사용 가이드에 아직 없는 feature의 명령을 사용 가능으로 추가하지 않았다.

명령·종료 코드·source SHA-256·기준 manifest 정보는 ignored
`output/pr-review/issue8183/stage1/`에 보관한다. 실제 codec 입력 확보·디코딩·Native/WASM
네 조합 빌드·성능·시각 검증은 실행하지 않았으며 2·3단계의 필수 완료 조건이다.

제품 source·Cargo manifest/lock·기존 회귀·workflow는 이 단계에서 바꾸지 않았다.
다음은 구현 계획의 검토와 2단계 진행 승인이다.
[현행 Issue Workflow](../manual/codex/docs_and_git_workflow.md#issue-workflow)의
계획서 작성 → 작업지시자 승인 → 구현과 테스트 순서를 따른다.
