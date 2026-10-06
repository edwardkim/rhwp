# PR #7024 검토: 저장 API의 공유 참조 계약

## 판정: 승인

- 원 PR: https://github.com/edwardkim/rhwp/pull/7024
- 기여자: `lpaiu-cs`; 관련 이슈: [#7021](https://github.com/edwardkim/rhwp/issues/7021).
- 최초 source head `733f6d56b3499439e850d4bf215a857941bfb59c`, 현재 head `4635c615fe34f1865f24711257229cbdfbaaaa67`.
- 로컬 체리픽 `15b13f300`·`4a1be2162`. 원격에서 재작성된 두 커밋과 `git range-diff` 패치 동일(`=`)을 확인했다. 추가 코드 변경은 없다.

## 본문 계약과 검토

DocumentCore 및 WASM의 HWP 저장·암호 저장·verify 저장 진입점을 `&mut self`에서 `&self`로
좁혔다. 함수 본문은 변경하지 않았다. 저장이 편집 동작으로 취급되는 불필요한 borrow 제약을
줄이는 변경이며, read-only 함수가 된 세 건의 #2724 mutation 면제 항목도 함께 제거했다.
면제 목록 전체를 늘리거나 guard를 무력화한 변경이 아니다.

## 검증과 보류 해소

기존 공개 fixture `samples/ta-pic-001-r.hwp`를 사용하는 공유 참조 export 세 경로와 반복 저장
동일성 회귀를 통합 전체 테스트에서 실행했다. 전체 Rust 9,473개 통과·46개 skip, fmt·기본/
workspace/WASM Clippy·workspace 및 WASM 빌드를 완료했다. 최신 원격 head의 CI rollup도
`SUCCESS`였다. 검증 미실행이라는 초기 보류 사유를 해소했으며 별도 제품 보정은 필요하지 않았다.

[통합 실행 결과·명령·provenance](pr_7024_review_impl.md)를 따른다. 문서 레이아웃 변경이 없으므로
불필요한 PDF 변환이나 시각 캡처로 저장 API 계약 검증을 대체하지 않았다.

## 원 PR/이슈 후속 기록 계획

merge 후 #7024·#7021에는 실제 merge SHA, 검증 결과, 이 리뷰와 공유 참조 회귀 테스트를 연결한다.
체리픽 통합 수용임을 명시하고 기존 comment가 있으면 수정한다. `closingIssuesReferences`가
비어 있어 #7021 자동 close를 가정하지 않는다. 통합 PR 생성·최종 CI·merge·후속 comment는
아직 수행하지 않았다.
