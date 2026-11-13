# seo-rii 2026-11-12 PR 최종 로컬 검토

| 원 PR | 범위 | 현재 판정 |
| --- | --- | --- |
| #8952 | macOS Metal 두 타일 | 승인 — 사용자의 Mac 기여자 주장 수용 결정 적용, Apple SDK 실기 미실행 명시 |
| #8953 | 이동 참조앱 | 메인터너 보정 후 수용 가능 — Linux·Windows 실제 이동 실행·문서 열기·닫기·정상 종료 완료 |
| #8954 | Windows 두 타일 qualification | 메인터너 보정 후 수용 가능 — 최종 실제 runner의 표시·편집·resize·복구·자원 종료 완료 |

최신 devel `41c889cfd8a869ad23199d6018f8b569879a575c`와 원 기여 head 3개를 보존했다. 최종 제품 소스는 `4ab994ee67ff66e35aa9168ac6fae506b6b26122`다. 메인터너가 CMake junction·Windows 실행 파일 metadata·plugin messenger 종료를 보정했다. [상세 원인](seo_rii_20261112_windows_build_finding.md)·[최종 검증 원장](assets/seo-rii-20261112/windows-correction-20261113/README.md)·[누적 계획](seo_rii_20261112_review_impl.md)을 따른다.

실제 Native Flutter 280개(skip 0), 공통 19개 명령과 최종 metadata 관련 50개, 실제 Linux·Windows Native/runner 빌드, Windows 두 타일 4단계, 양 OS의 이동 번들 실행·1→6쪽 이동·문서/창 종료를 확인했다. Windows 최종 exit는 0이며 수정 전 fault를 정상 종료로 인정하지 않았다. 단일 실제 DPR 1에서 검증했고 consumer GPU fence·하드웨어 가속·target reuse·driver 메모리를 검증 완료로 주장하지 않는다.

Native/fresh WASM 각각 원 범위 7쪽은 전쪽 raw 100.00%, 최신 devel 기능 보존 대조군을 포함한 8쪽 최저는 각각 99.75%·99.86%다. 모든 쪽수는 독립 Desktop Print와 일치하고 직접 확대 판독도 완료했다. Rust 생산 입력은 최신 base와 같고 기존 source60 빌드를 최종 tooling/consumer 소스에 정당하게 재사용했다. 변경한 Windows C++ 소비자는 최종 소스로 다시 빌드했다.

원 CI는 같은 원 head에서 각각 33개 성공·6개 정상 skip이다. 본인 계정 열린 PR은 0건, 최종 병합 시뮬레이션은 exit 0이다. 원격 source push·새 수용 PR·승인 리뷰·공개 댓글·merge는 아직 수행하지 않았다. 새 수용 PR의 head CI·self-review를 완료한 뒤 통합할 원격 범위 승인을 요청한다. 원 PR 3건의 기여 이력을 유지하며 Draft #8962–#8967과 전체 로드맵 #536·#8701은 OPEN을 유지한다.
