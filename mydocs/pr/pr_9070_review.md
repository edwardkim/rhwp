# PR #9070 메인테이너 검토

**최종 판정: 머지 보류.** 원 기능과 승인된 보완의 로컬 검증은 통과했지만 공통 특별 보호값 재현 조건이 남아 있다.

- 작성자: JinHo-von-Choi. 원 head: `8669ec6dc4b91a57d6af8e6b82dd391dff0832af`.
- 원 이슈: #8912. 사용자 지시로 reviewer `edwardkim`을 지정했고 원 commit을 보존했다.
- 기본 경로: maintainer_general. 보조 경로: intake_and_review, local_validation, multi_pr_update_branch, visual_fixture_evidence, post_merge. 정본 router·선택표와 해당 자식 절차를 적용했다. 기존 #9052 수용 이력이 있어 API association만으로 첫 기여로 취급하지 않는다.

## 주장과 메인테이너 보완

일반 기호 7개 복원과 잘린 `ch == 29` 교차 참조 방어는 유효하며 최신 devel에도 문제가 남아 있었다. 기존 #5860·#6380의 실측 PUA override는 보존한다. 원 기여의 괘선 77개 중 65개는 독립 한컴 저장값과 달랐고 점선·굵은 선을 다른 모양으로 매핑했다.

[승인된 보완 계획](pr_9070_review_impl.md)에 따라 77개를 실제 한컴 HWPX의 plane-15 문자로 복원했다. 공통 `native_pua_fonts`의 측정·paint 경로에서 실제 SFNT 프로그램을 함께 소비한다. 기존 지원 11개 순서를 유지하고 중복 없는 75개를 추가했다. 글꼴 bytes·윤곽·문서별 좌표 상수는 추가하지 않았다. 원본 SFNT가 없을 때의 기존 대체 계약도 유지한다.

실제 CLI 대조는 괘선 오인이 65자에서 0자로 줄었다. 정상 specimen의 원 HWP3·HWP 저장본·HWPX 저장본은 각각 전 1쪽 Native 98.56783%·최적화 fresh WASM 98.55072%다. 실제 독립 한컴 HWP5 재열기·HWPX 저장에서도 77개 문자가 같다. [문자 대조](assets/pr9070/linechar-before-after.json) · [한컴 재열기 영수증](assets/pr9070/hancom-hwp5-reopen.json) · [전쪽 TSV·Print·글꼴 근거](assets/pr9070/README.md).

## 입력·시각 근거와 호스트 장애

공개 원 합성 HWP3의 용지 0·비율 0은 parser-only fixture로 분리했다. 실제 사양의 정상 specimen을 별도로 생성해 사용자가 수동 Print한 PDF를 정식 보존했다. 7개 괘선 집합·일반 기호를 같은 배율로 직접 읽었다. PDF의 실제 Haansoft Batang hmtx·전진을 근거로 명시 font environment를 공급했으며 제품 기본값을 바꾸지 않았다. 사용자가 안내한 로컬 HBATANG을 실제로 사용했다.

호스트 자동 인쇄 실패와 사용자 재부팅은 [중단·복구 기록](jinho_von_choi_20261114_interruption.md)에 남겼다. Windows 호스트의 Hwp.exe·COM·인쇄 자동화는 계속 비활성이다. 사용자 수동 Print와 별도 원격 MCP 정상 재열기를 호스트 자동화 성공으로 바꾸어 기록하지 않는다.

## 공통 검증과 남은 조건

제품 source `eeaecff59e834f28437c98b509089bbb36a7682d`와 검증 head `3b5e750d66351799fe81c0e1b67c5e78323763f5`의 제품 경로는 같다. 최종 전체 회귀 13,574개·실패 0, fmt·root/WASM32/workspace Clippy·workspace build·manifest 검사와 편람 Native 383쪽 gate를 통과했다. Native 공식 3종도 4,073개 모두 통과했다. [최종 실행 기록](assets/jinho-von-choi-20261114/local-validation.json).

Native/fresh WASM 각각 134쪽의 최저 raw는 93.30%·92.46%이고 독립 Print의 전체 쪽수가 일치한다. 새 devel 기능 보존 41쪽씩의 PNG 82쌍은 수정 전후 같다. 반면 정본 보호 3,542행 중 과거 공식값 미재현은 Native 51행·WASM 56행이다. Native는 최신 devel과 후보의 전 1,771행이 같으며 WASM 미재현 56행도 exact base와 값·PNG가 모두 같다. 특별 하락 금지 gate의 통과·예외를 아직 선언하지 않는다. [보호값 재현 조사](assets/jinho-von-choi-20261114/protected-environment-review.md).

초기 집중 60개에는 새 바탕쪽 모듈이 실제로 실행되지 않아 그 통과 주장을 정정했다. 이후 집중 126개와 최종 전체 회귀에서 새 검사 3개의 실제 PASS를 확인했다. `b60322c765`의 두 실패도 최종 source에서 보완·재검증했으며 정확한 기존 기대값은 유지했다. 원 head의 CI와 누적 보완 후보의 CI는 구분한다. 아직 공개 댓글·push·새 PR·승인 review·merge·issue close는 수행하지 않았다.


검증 입력 커밋 확인: **충족**. [113개 입력/PDF의 실제 Git blob·실행 SHA-256 대조](assets/jinho-von-choi-20261114/input-commit-verification.json), 확인 commit `0844c7d115`. 검증한 변환/재저장본도 동일 bytes로 보존했고 상용 글꼴 프로그램은 포함하지 않았다.
