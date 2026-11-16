# seo-rii P120–P130 PR 11건 검토 계획

## 접수와 승인 범위

작업지시자는 seo-rii의 열린 PR을 조사하고 리뷰어가 없으면 메인테이너를 지정해 검토하도록 지시했다. 2026-11-16 KST, Ready PR 11건 모두 reviewer와 team 요청이 없음을 확인했다. 원 PR별 `edwardkim` 지정과 requested_reviewers API 재확인을 완료했다. 검토·로컬 통합·검증이 현재 범위이며 공개 댓글·승인 리뷰·원격 push·새 PR·merge는 수행하지 않았다.

기본 경로는 `maintainer_general`, 보조는 `intake_and_review`, `local_validation`, `multi_pr_update_branch`, `visual_fixture_evidence`, 대형 누적 diff의 `rework_and_exceptions`다. 모 workflow·선택표와 각 절차를 읽었다. 새 소스 보정은 실제 원인과 범위를 먼저 확정하며 원 PR head를 변경하지 않는다.

## 고정한 소스와 통합

기준 devel: `443dafb08107152e066689544e88d90988339567`.
원 마지막 head: `4379aa080b01a5da26f03f154287057ff6fb8269`.
자동 current-base 병합 source: `3c46e4dc550aeddc8b91c79f090239c538cd1506`.
실제 tree: `cb9c303399b1769d137751da9fe5c1ccdd8c1842`.
브랜치: `review/integration-8701-seo-rii-gpu-cache-lifecycle`.

| PR | 단계 | reviewer 지정 시 head |
| --- | --- | --- |
| #8962 | P120 Linux resize | `469ca8c04f5fab5a9a87bec1c8a09873032d4720` |
| #8963 | P121 producer-ready | `05d303178b0c8f14be8e3f6d1edf78892a66cd98` |
| #8964 | P122 창 종료·retirement | `d8251257d53c365de873fcab61b03f52ecdb67ff` |
| #8965 | P123 Metal tile cache | `2023cd3d2715a5eda2cadf30801ffcacefb4aab8` |
| #8966 | P124 D3D tile cache | `80062f4d0d11b69bfc1849d116150b96e7d7e899` |
| #8967 | P125 D3D private target pool | `41e8618fdff2a55a16ab60fc8c667f9255692179` |
| #9128 | P126 Metal admission | `207476213889668e17b2ea6f64eb80c735632f46` |
| #9129 | P127 유휴 polling | `720c161b9b961b862f6834ab045f6ff599279de3` |
| #9130 | P128 owner-thread trim | `c1d53287a6991c6195840c71927502dbc22f8907` |
| #9131 | P129 memory pressure | `3798dd236733ed41cf547e15c73b622ff44a5b04` |
| #9132 | P130 stamped metrics | `4379aa080b01a5da26f03f154287057ff6fb8269` |

원 11개 head의 최신 CI/check/status에 실패·진행 중 항목은 없다. 원격 녹색과 로컬·실제 OS 검증은 구분한다. 11개 각각의 base 병합 시뮬레이션은 exit 0이며, 마지막 head가 원 head 11개를 모두 포함한다. 원 commit을 재작성하거나 cherry-pick으로 SHA를 대체하지 않고 원 head 위에 최신 devel을 clean merge했다. 원 fork는 수정하지 않는다.

## 선행 해결과 검증 범위

- 부모 #536·#8701과 선행 수용 #9037 → `2c7644c53171e21c27b8a532ce4c53f01c40ddff`의 CMake junction·실행 파일 identity·messenger 종료 보정을 추적했다. 현재 caller와 실제 Windows 종료에서도 유지되는지 확인한다. 부모 로드맵 전체의 완료를 주장하지 않는다.
- resize·EventChannel·close → cache ownership/limits → admission → idle polling → trim → memory pressure → metrics 순서로 각 단계의 차이와 실제 소비 지점을 검토한다. 선행 원 head가 최종 head에 들어 있다는 사실을 동작 보존의 증거로 대신하지 않는다.
- Flutter SDK의 strict analyze·format·전체 test, Linux/Windows 공통 Python·portable C++·sanitizer, 실제 Native GL/D3D feature compile과 참조앱 표시·resize·복구·문서/창 종료를 검사한다. fake/mock/portable 결과를 실제 GPU 완료·메모리 측정으로 표시하지 않는다.
- Native 공통 cache·frame lifetime·admission의 집중 회귀와 필요한 Rust lint·suite 정책을 확인한다. current-base에서 바뀐 의존성과 제품 기능도 검증 범위에 포함한다. 광범위 회귀 재사용은 정본 3.2.2의 조건을 실제 만족할 때만 적용한다.
- 실제 최종 소비 화면과 CPU 독립 replay를 대조하며, 변경된 GPU 경로의 자원 상한·stale/timeout·unknown completion·retirement·공개 front 보존을 확인한다. 정식 문서 대조가 필요한 범위는 기존 입력/독립 Print·Native/fresh WASM 전쪽 TSV와 직접 판독을 준비한다. 문서 조판 변경 유무와 GPU 소비 검증을 구분한다.
- macOS 환경을 제공할 수 없어 기여자의 주장을 수용한다는 기존 작업지시자 결정을 적용한다. Apple SDK·Metal 실기를 실행한 것으로 기록하지 않으며 portable/코드 검토와 실제 검증 범위를 구분한다.

검사 결과와 문제별 근거는 원 PR 번호의 review 문서에 분리한다. 필수 미해결·미검증을 완료로 표시하거나 다른 PR의 승인으로 대신하지 않는다. 원격 후속 처리는 실제 후보·검증·본문·댓글을 준비한 뒤 별도 승인 범위를 확인한다.

## 로컬 검토 완료 후 원격 경로 확정

2026-11-16 KST, 제품 보정 없이 원 head 11개를 보존한 최종 스택 검토를 완료했다. 원 #9132 자체가 exact code candidate이며 새 수용 PR을 만들지 않는다. 로컬 current-base 병합은 실제 기능/출력 공존 검증용이다. `pr_review_workflow.md` 3.2.2의 원 head CI 재사용과 focused·실제 consumer 검증을 적용하고, 선택적으로 시작한 중복 broad workspace 빌드는 중지 사실을 보존했다. 자세한 실행/비해당 판정은 [원장](../assets/seo-rii-20261116/README.md), 개별 PR 판정은 [최종 검토](seo_rii_20261116_review.md), 아직 실행하지 않은 원격 조치는 [처리안](../assets/seo-rii-20261116/remote-action-plan.md)에 연결한다.
