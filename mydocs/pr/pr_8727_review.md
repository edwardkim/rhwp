# PR #8727 검토 — feat(native): add bounded Windows D3D Flutter texture bridge WIP (P101)

## 최종 판정

**메인터너 보정 후 수용 가능 — 필수 로컬 검증 완료.** 원 기여 head를 그대로 승인한 판정이 아니다. 검증한 통합 후보의 보정과 한계를 포함한 수용안이며, 기존 대조군 차이 예외·Draft 간접 수용 효과·원격 처리의 작업지시자 승인과 새 통합 head의 GitHub CI 성공 전에는 merge를 보류한다. 공개 댓글·push·새 PR·merge는 아직 수행하지 않았다.

## 접수와 검토 source

| 항목 | 관측값 |
| --- | --- |
| 원 PR / contributor | [#8727](https://github.com/edwardkim/rhwp/pull/8727) / seo-rii |
| 원 repository / branch | `seo-rii/rhwp` / `task/536-p101-flutter-windows-texture` |
| 고정 원 head | `ac393935254caa9fd27c1b0c92346913d4f015af` |
| 단계 비교 시작 | `d9570e04cc5e520820026d5208e2a93edd5fd50b` |
| reviewer | edwardkim; 원장에 배정·API 재확인 시각 기록 |
| 기준 devel | `b8dd67c5001dfb0410e9e192528676396cdd6a4e` |
| 검증한 기능·회귀 source | `263b1e753c006d1631d25794588b6638fa1172ae` |
| 최종 제품 보정 | `cda7c8cb33bb7ded970d033ed1543aabb49c63a2` |

원 commit·순서·직전 단계 경로·접수 CI는 [접수 원장](assets/seo-rii-20261107/intake.json)에 보존했다. 모든 원 head는 마지막 Ready tip `629f204f82e95c4f2f1a12d318698d1a01fe1cf7`의 조상이며 최신 devel과 clean merge로 통합했다. 과거 원 head CI와 새 통합 head CI를 혼용하지 않는다. #8666·#8671의 실제 과거 CI 시간 초과는 [별도 진단](assets/seo-rii-20261107/ci-failure-diagnosis.json)에 기록했다.

## 누적 소스의 개별 소비 경로 검토

실제 Windows SDK의 protected registrar 접근·변수 가림 compile 실패를 보정했다. 새 제품 코드의 MSVC/Native D3D compile/link, 실제 Windows ABI 4종과 하드웨어 D3D12 색상·알파·stale 거부 2종이 통과했다. 최종 Flutter D3D11 consumer sampling·지속 표시 qualification은 bounded foundation 완료와 구분한다.

[최종 Windows source 실행](assets/seo-rii-20261107/platform-execution/windows-final-correction.json)과 [SDK 보정 이력](assets/seo-rii-20261107/host-validation-progress.md)을 따른다. 최종 Flutter D3D11 consumer sampling·지속 표시 qualification을 완료로 표시하지 않는다.

## 필수 검증과 보정

- 최소 공개 글꼴 환경 전체 nextest **12,825 passed / 45 skipped**; 최종 Native 집중 **194 passed**.
- Native 공식 3종 **4,077 passed** = lib 4,040 + placeholder 2 + Direct PDF 35; lib의 기존 ignore 12건은 별도다. 집중 검사와 이 수치를 합산해 고유 검사 수로 표시하지 않는다.
- fmt·native/WASM32/workspace/Native FFI GL Clippy·workspace build·doctest·standalone Vulkan run/fmt/Clippy·Studio test를 포함한 최종 17개 묶음이 모두 통과했다. Studio는 **2,745 passed / 2 skipped**, TypeScript/Vite build도 별도 통과했다.
- 실제 Windows SDK·Native D3D와 Linux Qt·Flutter 결과는 [플랫폼 원장](assets/seo-rii-20261107/platform-execution/summary.json), [source·재사용 범위](assets/seo-rii-20261107/validation-scope.md)로 구분했다. 소프트웨어 GL/Vulkan을 물리 GPU 검증으로 표시하지 않는다.
- 최종 테스트 파일 주석 보완은 첫머리 module documentation뿐이다. 함수·assertion·fixture·제품 코드는 같으며 [11개 파일의 body hash](assets/seo-rii-20261107/test-header-metadata-receipt.json)로 검증 source와의 동일성을 확인했다. 변경 뒤 fmt·자동 수집 48/48·문서 검사를 통과했다.

정확한 command·시각·source·결과는 [최종 gate](assets/seo-rii-20261107/local-gates.json)를 따른다. Windows 공개 registrar·변수 가림, 공개 fixture font 공급과 실제 숫자 fallback의 메인터너 보정은 원 기여 SHA와 분리해 기록했다.

## 조판 원칙·입력·시각 검증

문서 조판의 줄·높이 생산값을 수정하지 않는다. 기존 PageLayerTree와 source/revision·배치 좌표를 소비하는 host 계약을 검토했다. 일반 출력 유지 대조와 실제 ABI/플랫폼 실행을 서로 대용하지 않는다.

[실제 입력/기준 PDF 18개의 commit 일치](assets/seo-rii-20261107/validation-input-commit-receipt.json), [빌드·artifact 출처](assets/seo-rii-20261107/build-provenance.json), [전쪽 표·Native/fresh WASM TSV·대표 review/overlay](assets/seo-rii-20261107/visual/README.md)를 확인했다. 각 경로 전 24쪽 모두 raw 90% 이상이고 독립 PDF와 쪽수 일치·누락 0이다. 같은 devel/후보 PNG 48쌍은 바이트 단위로 같으며, 최종 Native 편람 383쪽은 최저 raw 99.96%·기준 대비 회귀 0쪽이다.

직접 판독에서 기존 eq-01 본문 굵기·수식 간격/분수 모양과 picture-crop의 가는 테두리 차이를 확인했다. devel과 같은 이 대조군 범위의 예외를 수용안에 명시했으며, 90% 통과로 임의 면제하지 않는다. 이 결정은 아직 작업지시자 승인 전이다. 원 숫자 의미 오류나 새 회귀를 예외에 넣지 않는다.

## 원격 수용과 후속

[수행 계획](seo_rii_20261107_review_impl.md)과 [구체적 원격 처리안](assets/seo-rii-20261107/remote-action-plan.md)에 따라 upstream 통합 branch·Open PR의 새 head CI·self-review·원 기여를 보존하는 merge를 제안한다. Draft #8646은 별도 처리 대상에서 제외했지만 Ready history의 P73 두 commit을 merge하면 간접 MERGED로 표시될 수 있으므로 이 효과도 승인안에 포함했다.

원격 승인·실제 CI·merge SHA를 확인한 뒤 최종 운영 기록과 devel 동기화를 먼저 끝내고, 이 PR의 검증 범위·제한·감사 댓글을 게시한다. 문서 비교를 쓰는 댓글에는 해당 입력·쪽의 실제 review·overlay와 merge SHA 고정 raw URL을 첫 댓글부터 넣고 다운로드 hash·실제 표시를 확인한다. #536·#8701 전체 로드맵은 OPEN을 유지한다. 종료 정리는 canonical post_merge 절차를 따른다.
