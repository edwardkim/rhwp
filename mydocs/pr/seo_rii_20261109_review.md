# seo-rii PR 9건 최종 로컬 검토

## 판정

**9건 모두 수용 권고 — 로컬 필수 검증 완료, 원격 처리 승인 대기.**
추가 제품 보정은 필요하지 않았다. 메인테이너는 두 Rust 회귀 파일의 측정 근거 주석과 독립 검증
증거를 보완했으며 검사 본문·fixture·기준값은 바꾸지 않았다. GitHub 공개 승인 review·push·새 PR·
merge·close는 수행하지 않았다.

| 원 PR | 검토 범위 | 개별 판정·근거 |
| --- | --- | --- |
| #8749 | 명시적 Canvas2D strict-outline preview | [수용 권고](pr_8749_review.md) |
| #8761 | P103: 안전한 worker 저장·복구 | [수용 권고](pr_8761_review.md) |
| #8764 | P104: 문서 lifecycle | [수용 권고](pr_8764_review.md) |
| #8765 | P105: bounded 다중 페이지 CPU viewport | [수용 권고](pr_8765_review.md) |
| #8767 | P106: 실제 Windows texture consumer | [수용 권고](pr_8767_review.md) |
| #8769 | P107: 실제 Apple SDK Metal producer | [수용 권고](pr_8769_review.md) |
| #8777 | P108: 실제 Linux consumer·연속 GL | [수용 권고](pr_8777_review.md) |
| #8779 | P109: Windows immutable ownership transfer | [수용 권고](pr_8779_review.md) |
| #8781 | P110: AppKit Metal 연속 표시 | [수용 권고](pr_8781_review.md) |

## 통합 소스

- base: `upstream/devel@06181802e574dbd91e62375536751dea8c270fcd`.
- 작업 브랜치: `review/integration-536-seo-rii-preview-flutter`.
- 작업공간: `/home/edward/mygithub/rhwp-worktrees/review-integration-536-seo-rii-preview-flutter`.
- 통합 제품 소스: `be778bae9b3803050146dcc47a896940587a612e`.
- 측정 근거 주석 보완 및 최종 Rust 검증 소스: `5bccdb7b90c4f447f523fc7ec4fa0931e761c3a7`.
  이후 추가한 변경은 `mydocs/`의 검토 기록·증거뿐이다.

#8781에 P103–P110의 원 head 8개가 모두 포함된다. 독립 #8749와 함께 실제 로컬 merge해
[원 head 9개를 모두 보존](assets/seo-rii-20261109/original-head-preservation.json)했다.
최신 devel과 충돌이 없으며 원 PR별 reviewer edwardkim을 지정·확인했다.
기존 #8739의 통합·보정 이력을 추적하고 [기존 해결 여부](assets/seo-rii-20261109/already-resolved-check.json)를
대조했다. 초기·최종 source 조회와 CI 원장은 [공통 증거](assets/seo-rii-20261109/README.md)에 연결한다.

## 실제 검증 결과

| 검증 | 결과 |
| --- | --- |
| fmt·root/WASM32/workspace all-target Clippy·workspace build·고정 base suite 정책 | 모두 통과 |
| 최소 글꼴 전체 release-test nextest | 13,005건 통과, 기존 skip 45건 |
| 최소 글꼴 Native 공식 3종 | lib workspace 4,040건 + missing-picture 2건 + Direct PDF 35건 = 4,077건 통과 |
| Rust 집중·Chromium 실제 wasm32 | 8건·12건 통과; Chromium은 글꼴 파일 시스템 격리에서도 통과 |
| 최적화 WASM | release + wasm-opt 완료; 실제 공개 Screen/Print API와 기본 API 보존 확인 |
| Flutter | unit/widget 183건, 실제 Native·owner 연동 최소 글꼴 6건 통과(skip 0); analyze 문제 0건 |
| Python·C++·Linux SDK/EGL | Python 최종 182건, portable C++·실제 public header compile/link·EGL handoff 통과 |
| 실제 desktop 표시 | 메인테이너 Linux GTK 32회 실행; 실제 Windows/Apple SDK artifact 각각 32회 대조·직접 판독 |
| 독립 Print Native/fresh WASM | 각 7쪽, 전쪽 raw 최저 100.00%, 쪽수 일치; 같은 backend의 base/후보 PNG 14쌍 동일 |

[실제 명령·종료 코드·단계별 환경·전체 결과](assets/seo-rii-20261109/final-local-matrix.json)와
[시각·플랫폼 검증 원장](assets/seo-rii-20261109/README.md)을 보존했다. 최초 Node 경로와 Native
다운로드 전 DNS 경로 누락은 로컬 실행 환경 보완으로 해결했으며 통과한 검사는 반복하지 않았다.
최초 시도는 제품 검사 시작 전의 실행 환경 설정 실패로 기록했다.

새 source·test header를 포함한 통합 후보이므로 원 PR의 녹색 CI를 새 후보의 전체 검증으로
대신하지 않고 위 묶음을 한 차례 완료했다. 최종 제출 기록 commit은 `mydocs/`만 추가한다.
Native/fresh WASM은 같은 제품 소스의 실제 산출물을 사용했고 기본 출력과 새 opt-in API 검증을 구분했다.

## 수용 범위

원본 LineSeg·측정·조판·pagination 생산자는 바꾸지 않는다. 같은 page-layer tree와 Native geometry를
renderer/Flutter 참조 host가 소비한다. 따라서 조판 변경의 편람 383쪽 gate는 비해당이며 실제
렌더링·기본 출력 전쪽 비교·새 API·플랫폼 consumer 경로의 검증은 완료했다.
문서별 수치 맞추기, 기존 기대값 완화, 새 ignore 또는 baseline 갱신은 없다.

Windows/macOS는 실제 SDK 실행 source·런타임 입력·제공된 Native 라이브러리 source와 해시를
대조했다. 새 통합 head에서 두 OS의 Native 자체를 재빌드했다고 주장하지 않는다. 제공된 Native
source와 현재 devel의 제품 차이는 JPEG description 메타데이터뿐이며 ABI header는 같다.
현재 변경은 Canvas2D/WASM과 Flutter 참조 host이고 OS별 Native producer·ABI는 보존한다.

연속 표시는 `consumerGpuCompletionVerified:false`, `hardwareAccelerationVerified:false`와
Windows `targetReuse:false`의 명시적 계약이다. 실물 GPU 성능·완료 fence·장시간 OS IME/접근성
전체 완료로 확대하지 않는다. 이전 #8726 예외를 SDK 검증 면제로 옮겨 쓰지 않았다.
**#536·#8701 전체 로드맵은 OPEN을 유지한다.**

## 원격 처리안

[원격 처리안](assets/seo-rii-20261109/remote-action-plan.md)의 승인 범위는 통합 branch push →
Open PR → 해당 head CI·self-review → 원 기여를 보존하는 merge → 원 PR 9건 완료 확인·감사 댓글 →
운영 기록·devel 동기화·종료 정리다. 원 fork branch와 다른 작업공간·설치 도구·로컬 소비 검증 환경은 보존한다.
CI나 보호 설정·보안 분류 변경을 이 승인에 자동으로 포함하지 않는다.
