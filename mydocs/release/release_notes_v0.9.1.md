# rhwp v0.9.1 — 표·저장·편집과 배포 안정성

## [0.9.1] — 배포 준비

표·문단·글꼴의 조판과 저장 후 재열기, 붙여넣기·작은 화면의 조작을 보강합니다. Native 호스트 기반과 배포·검증 환경의 누적 변경도 포함합니다. 아래 항목은 릴리즈 후보이며 실제 게시 상태는 채널별로 확인합니다.

### 조판·편집·저장

- 중첩 표의 첫 줄·이어받기·마지막 조각, 셀 안 그림·도형과 표 뒤 본문의 점유·이월을 보정했습니다 ([#8984](https://github.com/edwardkim/rhwp/pull/8984), [#9066](https://github.com/edwardkim/rhwp/pull/9066), [#9069](https://github.com/edwardkim/rhwp/pull/9069), [#9106](https://github.com/edwardkim/rhwp/pull/9106)).
- 저장 줄·빈 문단·머리말/꼬리말·바탕쪽의 배치와 수식·기호·실제 글꼴 프로그램 선택을 보강했습니다. 기존 문서의 모든 조판 차이를 해결했다는 의미는 아닙니다 ([#9074](https://github.com/edwardkim/rhwp/pull/9074), [#9083](https://github.com/edwardkim/rhwp/pull/9083), [#9096](https://github.com/edwardkim/rhwp/pull/9096), [#9106](https://github.com/edwardkim/rhwp/pull/9106)).
- HTML 표 붙여넣기의 테두리·수직 정렬과 반복 저장, 번호·글머리표의 글자모양 보존을 개선했습니다 ([#9017](https://github.com/edwardkim/rhwp/pull/9017), [#9140](https://github.com/edwardkim/rhwp/pull/9140)).
- 제품정보와 공통 대화창의 작은 화면·확대 표시를 보정했습니다 ([#9092](https://github.com/edwardkim/rhwp/pull/9092), [#9136](https://github.com/edwardkim/rhwp/pull/9136)).

- 문서·글꼴·쪽 나눔 상태가 유효하면 Canvas2D 편집한 쪽의 호스트 글꼴 요청만 다시 수집해 입력 비용을 줄였습니다. 다른 변경에서는 전체 요청 수집을 유지합니다 ([#9168](https://github.com/edwardkim/rhwp/pull/9168)).

- HWP 다운로드 자동 열기의 첫 페이지가 실제로 그려진 뒤 완료를 표시하며 글꼴 준비 경합을 처리합니다 ([#9170](https://github.com/edwardkim/rhwp/pull/9170)).

- HWP3 저장 그림의 쪽 소속과 Native 글꼴 프로그램 복구·원본 TTF/TTC 우선순위를 보강했습니다 ([#9157](https://github.com/edwardkim/rhwp/pull/9157)). 릴리즈 준비에서 Native/fresh WASM 18종 전쪽 보호 검증을 완료했습니다.
- Chrome worker 종료로 설정 응답을 받지 못하면 배지·미리보기를 임의로 켜지 않고 채널 오류를 처리합니다.

- 문서 열기가 처리 중일 때 시작 화면의 빈 문서·자동 복구가 요청을 덮지 않도록, 완료·실패·교체의 공통 수명을 추적합니다. SDK의 준비 직후 파일 열기와 기존 첫 화면 완료·취소 동작을 함께 검증했습니다.

### PDF·CLI·Native와 배포 환경

- PDF ToUnicode 범위 항목의 토큰·주석 처리를 보정했습니다. PDF 가져오기·암호·선택 쪽·자원 한도·Worker 사용법은 기존 PDF 개발자 안내를 따릅니다 ([#8922](https://github.com/edwardkim/rhwp/pull/8922)).
- CLI의 그림·도형 편집과 Native 글꼴 진단을 보강했습니다 ([#9052](https://github.com/edwardkim/rhwp/pull/9052), [#9147](https://github.com/edwardkim/rhwp/pull/9147)).
- Qt/Flutter Native 호스트의 preview·타일·다중 페이지 표시·캐시·수명 관리를 확장했습니다. 참조앱·실험 기반의 지원 범위와 플랫폼별 검증 한계는 각 Native 안내를 따르며, 모든 플랫폼의 완성된 GPU 편집기를 약속하지 않습니다 ([#8809](https://github.com/edwardkim/rhwp/pull/8809), [#9132](https://github.com/edwardkim/rhwp/pull/9132)).
- 보안 진단의 본문 출력과 qcms 테스트 포인터를 정리했습니다. 보안 탐지 규칙은 유지합니다 ([#8884](https://github.com/edwardkim/rhwp/pull/8884)).
- Ubuntu 24.04 기준과 26.04 호환 검증, Fuzz 빌드/실행 분리·codegen 분할을 반영했습니다. 기존 runner 종료의 원인을 OOM으로 확정한 변경은 아닙니다 ([#9155](https://github.com/edwardkim/rhwp/pull/9155)).
- deprecated Atomic 호출 13곳을 MSRV 1.93.1 호환 구현으로 교체했습니다. 자원 상한·ID 고갈·메모리 순서 계약은 유지합니다.

### 패키지와 호환성

제품·npm·VS Code·브라우저 확장 후보는 0.9.1입니다. crates.io는 변경·의존성 갱신이 필요한 8개를 새 버전으로 준비하고, 동일한 기존 6개를 재사용합니다. 게시된 버전을 덮어쓰지 않으며 Rust의 최소 버전은 1.93.1입니다. 실제 배포·독립 소비 검증은 게시 단계에서 확인합니다.

### 기여자

v0.9.0 이후 기능 기준 `197ce19e99438c2cba452c654ae1af504dab85ec`까지 **1,238개 커밋**의 Git 저자·공동 저자·원 PR·통합 출처를 감사했습니다. 사람 **19명**과 Dependabot에 감사합니다. 로컬 deprecated 보정과 릴리즈 메타데이터는 이 명단에 이미 있는 메인테이너의 후속 변경입니다.

<!-- release-contributors:v0.9.1:start -->
- @Bongseop-Kim
- @chrisryugj
- @donggyun112
- @edwardkim
- @gitprogam
- @hyunmyungJaneLee
- @jangster77
- @jeong-sik
- @JinHeeWond
- @JinHo-von-Choi
- @johndoekim
- @pblsketch
- @planet6897
- @postmelee
- @puckqook
- @sacru2red
- @semanticist21
- @seo-rii
- @signspark
<!-- release-contributors:v0.9.1:end -->

Bot: @dependabot[bot]. 기여 근거와 identity 판정은 [공개 원장](../tech/investigations/release-v0.9.1/contributor-ledger.json)에 보존합니다.
