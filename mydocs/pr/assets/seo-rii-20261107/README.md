# seo-rii 2026-11-07 검토 증거

[접수 원장](intake.json) · [수행 계획](../../archives/seo_rii_20261107_review_impl.md). Ready 29건의 원 SHA를 보존하는 로컬 통합 검토다. 원격 수용과 두 예외를 승인받았으며 통합 PR #8739의 CI·self-review·merge를 완료했다.

## 현재 완료한 범위

- [실제 Linux·Windows 호스트 검증](host-validation-progress.md)과 [플랫폼 실행 요약](platform-execution/summary.json): 실제 SDK compile/link, Native ABI, Linux Qt/Flutter 실행과 Windows 하드웨어 D3D12 pixel 확인을 구분했다. Windows C++ compile 결함과 최소 글꼴 캐시 검사 의존성을 보정했다.
- [Metal 결정](metal-maintainer-decision.json): 작업지시자가 default-disabled 실험 기반으로만 예외 수용했다. Apple SDK·실물 Metal·지속 표시의 완료 주장이 아니다.
- [#8337 실제 숫자 대체 보정](positioned-equation-fallback-finding.md): `cda7c8cb33`에서 공통 producer 매핑을 재사용했다. 수식 face가 없을 때 4·3·AB를 직접 판독했고, 원 face 공급 전후 PNG는 동일하다. 원 사례의 Native/fresh WASM 전 1쪽은 각각 99.60%·99.48%이며 face 부재 Native는 99.63%다.
- [최종 Native 383쪽 편람 gate](platform-execution/handbook-native-final-correction.json): 보정 코드에서 최저 raw 99.96%, 기준 대비 회귀 0쪽이다.
- [실제 Direct PDF 6쪽](direct-pdf/README.md): 모두 raw 100%이며 radial 중심·반경·색 변화와 표식을 직접 확인했다.
- 최적화 fresh WASM을 Rust 1.93.1 Docker에서 공용 target으로 빌드했고, pkg/public JS·WASM hash 일치를 확인했다. [전쪽 Native/fresh WASM TSV](visual/)는 각 24쪽 모두 90% 이상이다. Native 전쪽 최저 95.46%, WASM 최저 94.65%다. 수치와 직접 판독을 분리한다.
- source `263b1e753c`에 실제 producer 숫자 회귀와 정식 시각 증거를 기록했다. Studio TypeScript·배포 빌드도 통과했다. 보정 전 제품 코드에서는 기존 5개 검사 통과·신규 실제 숫자 검사 실패를 확인했고, 복구한 보정 코드에서는 해당 검사를 포함한 Native 집중 회귀 194건이 모두 통과했다.
- 같은 회귀 source의 최소 글꼴 전체 nextest 12,825건이 통과했다. 45건 skip과 장시간 대형 표 검사 1건을 구분해 기록했다. Native 공식 3종은 4,077건(lib 4,040 + placeholder 2 + Direct PDF 35) 통과이며, lib의 기존 ignore 12건은 별도다.
- Rust lint·workspace·doctest·독립 Vulkan 도구·Studio를 포함한 최종 17개 묶음이 모두 통과했다. doctest 8 pass / 3 ignore, Studio 2,745 pass / 2 skip이며 TypeScript/Vite build도 통과했다. [공통 gate 원장](local-gates.json)을 따른다.
- 독립 Vulkan 도구도 Skia 0.156.0으로 실제 compile/run·fmt·Clippy를 통과했다. [응답](platform-execution/vulkan-final-response.json)의 실제 장치는 llvmpipe LLVM 20.1.2이며 hardwareAccelerationVerified/presentationVerified는 false다.
- 테스트 파일 첫머리 11개의 provenance·시각 표기를 보완했고 [실행 본문 hash](test-header-metadata-receipt.json)가 검증 source와 바이트 단위로 같음을 재확인했다. 제품·assertion·fixture 변경은 없다. 보완 후 fmt·48/48 자동 수집·diff 검사를 통과했다.

## 남은 원격 결정

최신 devel `b8dd67c500`과 같은 글꼴·입력의 전쪽 PNG 48쌍이 바이트 단위로 같음을 확인했다. [문서별 전쪽 수치·TSV·대표 review/overlay](visual/README.md)를 보존했다. 기존 그림의 가는 테두리·eq-01 글자 간격 차이는 base에도 동일하게 남으며 90%만으로 면제하지 않는다. [검증 source와 재사용 범위](validation-scope.md), [승인 전 원격 처리안](remote-action-plan.md)을 따른다.

원 PR별 판정은 **메인터너 보정 후 수용 가능 / 필수 로컬 검증 완료**로 갱신했다. 기존 대조군 차이의 한정 예외와 Draft #8646의 선행 P73 두 commit이 간접 MERGED로 표시될 수 있는 효과를 원격 수용안에 명시했다. 이 결정과 push·Open PR·새 head CI·self-review·merge·후속 처리의 승인을 기다린다. #536·#8701 전체 로드맵은 OPEN을 유지한다.

[제출 전 원격 상태](pre-submission-state.json)에서 devel·원 head 29개·reviewer가 그대로임을 확인했다. 별도 사용자 작업 Draft #8738은 실패 check가 없어 변경하지 않았다.

## 최종 수용

[통합 PR #8739](https://github.com/edwardkim/rhwp/pull/8739)의 head `0d2548fd88ea76b10e53eec841376b4bcbecbc4a`에서 필수 CI·self-review를 완료했고, 원 기여 이력을 보존하는 merge `f5f0fbee95b5fb0e32e99f8f639d228ae55389dd`로 수용했다. 작업지시자는 기존 eq-01 본문 굵기·수식 간격/분수 모양 및 picture-crop 가는 테두리 차이에 한정한 예외와 Draft #8646의 간접 MERGED 효과를 승인했다. 새 숫자 오류·회귀·필수 검사 실패는 예외에 포함하지 않았다. #536·#8701은 OPEN을 유지하며 #8337 완료·원 PR별 후속 댓글·정리는 정본 순서에 따라 이어 간다.

[운영 기록](remote/completion.md).
