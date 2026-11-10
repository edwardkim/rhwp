# seo-rii P111–P116 원격 처리안

## 준비한 결과

제품 검증 source는 `857594293fc7d8e8aec83fcc13a0b40a32a8f037`, 최신 기준 devel은
`411d4a91fd692f88fdeebd6b323eca0b295318b1`이다. 제출 때는 최종 검토·증적 문서 commit SHA를
정확한 head로 고정하며 제품 입력과의 차이는 검토 문서·증적뿐인지 확인한다.

원 PR #8869·#8870·#8871·#8872·#8873·#8874의 원 기여 SHA를 모두 보존했다.
승인한 보정은 Linux의 다중 페이지 인수 receipt 연결과 새 Mac native 타일 두 파일의 후속 분리다.
Mac plugin 전체는 최신 devel과 같으며 #8701에서 새 타일 구현을 이어 간다.
[최종 검증 원장](README.md)에 실제 플랫폼·빌드·계약·전쪽 시각 검증을 기록했다.

## 승인받아 실행할 범위

1. 최신 devel·원 head 6개·본인 계정의 열린 PR과 reviewer 상태를 재조회한다. 제품 변경이
   추가되면 단순 merge 성공으로 통과시키지 않고 영향 범위와 기능 보존을 확인한다.
2. 정확한 제출 head를 upstream의 `review/integration-8701-seo-rii-gpu-desktop`에 push한다.
   원 fork branch에 push하거나 원 기여를 rewrite/squash하지 않는다.
3. devel 대상 Open 통합 PR을 생성한다. 본문은 실제 source·Mac 후속·Windows SDK 검증 범위와
   Linux 실제 화면, 기본 Native/fresh WASM 각각 7쪽의 raw 최저 100.00%, 전체 TSV 원문·raw 링크와
   head SHA로 고정한 대표 PNG를 포함한다. 게시 뒤 API·raw 다운로드로 파일·본문·해시를 확인한다.
4. 해당 exact head의 필수 CI·최종 정책 감사를 확인한다. 로컬 검증이나 원 PR의 기존 CI로
   대신하지 않는다. 원격 workflow 취소·재실행·실행 정의·보호 설정·보안 분류 변경은 이 실행안에
   포함하지 않으며 실제 필요 시 원인과 구체적 범위를 분리한다.
5. self-review를 공개하고, 최신 base·정확한 head·required check·mergeable와 `merge-tree`를
   병합 직전에 재확인한다. 모든 필수 조건이 성공한 경우에만 원 기여를 보존하는 merge commit으로
   통합한다. expected head를 지정하고 CI·review 요건을 임의로 우회하지 않는다.
6. 원 PR 6건의 간접 MERGED 상태를 확인한다. 자동 반영되지 않으면 통합 수용 사실과 merge
   commit·각 고유 기능/보정·시각 증거를 알리는 댓글 후 완료 반영한다. 거부로 종료하지 않는다.
   원 fork branch는 보존한다. 부모 #536·#8701은 OPEN을 유지한다.
7. 원 PR별 감사 댓글에 merge SHA의 PNG를 실제 표시하고 API·PNG·TSV의 원본 해시를 확인한다.
   review archive·오늘할일 등 허용된 mydocs 운영 기록을 devel에 반영한 뒤 동기화한다.
8. 내 작업의 branch/worktree와 실행을 정리한다. 정본 post_merge의 종료 조건과 실제 사용 여부를
   확인한 뒤 기본 저장소 `output/` 전체·`target/pr-review/debug`를 자동 정리한다. 별도 백업·압축·
   이관을 만들지 않는다. 다른 작업공간·원 fork·설치 SDK 도구·로컬 crates.io 환경·다른 target
   profile과 정식 입력·PDF·커밋된 최종 증적은 보존한다.

이 계획은 작업지시자의 승인 뒤 실행했다. [통합 PR #8907](https://github.com/edwardkim/rhwp/pull/8907)은 `0f91d29ca81f00be3d5adb91857875b40ec5e355`로 머지됐고 원 PR 6건의 MERGED 반영을 확인했다. 운영 기록·devel 동기화 뒤 댓글과 종료 정리를 완료하며 실제 결과는 정리 로그에 남긴다. CI workflow를 수정하지 않았으므로 workflow 변경 PR의 옵션 2
후속 기록을 이 묶음에 기계적으로 적용하지 않는다.
