# PR #8870 검토 — feat(flutter): compose bounded GPU tile groups (P112)

## 최종 판정

**메인터너 보정 후 수용 가능.** 원 기여 head를 그대로 승인하는 판정이 아니다. 원 head 6개를 보존한 통합 후보에서 Linux 인수 확인 누락을 보정하고, 승인한 Mac 제외 범위를 적용했다. 최종 제품 검증 source는 `857594293fc7d8e8aec83fcc13a0b40a32a8f037`이다. 승인된 원격 처리로 [통합 PR #8907](https://github.com/edwardkim/rhwp/pull/8907)의 제출 head `6e2a37ea860aba30ecc80b14758f25ebb534683a`에서 CI·self-review를 완료하고 `0f91d29ca81f00be3d5adb91857875b40ec5e355`로 머지했다. 이 원 PR도 GitHub에서 MERGED로 자동 반영됐다.

## 접수와 검토 기준

- 작성자 seo-rii, base devel, non-draft. reviewer edwardkim을 지정하고 `2026-11-10T14:48:02.751194+09:00`에 API로 확인했다.
- 원 head `20eb3d8d902f03586db6a946e44b7b9b1308f0c3`, fork `seo-rii/rhwp`, branch `task/8701-p112-gpu-tiles`.
- 접수 기준 devel `5f4e95c187f64116f48f710c3c048162786c1c14`. 원 PR 규모 +3992/−247, 29개 파일은 선행 기능을 포함하는 누적 diff다.
- 개별 병합 시뮬레이션 exit 0, 원 head는 기준 devel에 포함되지 않았다. 이 사실만으로 기능·시각 검증 통과를 선언하지 않는다.
- [누적 검토·검증 계획](seo_rii_20261110_review_impl.md)에 따라 원 기능·독립 실행·미실행을 구분한다.

## 고유 변경·보정과 실제 소비 경로

범위: **Linux·Windows 두 타일 기반과 Mac 후속 분리**.

공통 unique snapshot union·shared lease와 Linux/Windows의 두 member admission을 확인했다. 준비·검사·할당 뒤 두 metadata 상태를 함께 확정하며 새 controller가 이전 자원 한도를 초기화하지 않는다. 승인에 따라 새 Mac native 타일 두 파일을 제외하고 Mac 폴더 전체를 최신 devel과 같게 유지했다.

최종 Linux GTK에서 초기·편집·resize 3회, seam·CPU/독립 2배 source 대조·종료를 통과했다. Windows는 portable·production 오류 35조건과 실제 MSVC/SDK compile/link를 확인했다. Windows GPU 화면 수집 완료를 주장하지 않는다.

[누적 검증 원장](../assets/seo-rii-20261110/README.md)에 실제 source·명령·종료 코드·수치와 직접
판독한 PNG를 보존했다. 초기 단일 페이지 두 타일 검사를 여러 페이지 표시의 증거로 바꾸지
않았고, 실제 실행과 mock/portable·SDK compile/link를 구분했다.

## 최종 로컬 검증과 기본 출력 보존

- 최신 base `411d4a91fd`와 충돌 없이 통합했으며 원 head 6개를 모두 Git 조상으로 보존했다.
- Flutter 계약/위젯 269개와 최소 글꼴의 실제 Native 연동 6개가 통과했다. skip 6개를 통과로 세지 않고 별도 Native 실행에서 해소했다.
- Linux 공통 C++ 18 cases/292 checks·tile 4/25, 실제 SDK/GLES·GTK 검사와 Windows portable·실제 SDK 검증이 통과했다. Python bundle 11개·tile qualification 4개와 실제 Dart FFI fixture 3경로도 통과했다.
- 새 base의 #8863 정식 집중 회귀 8개와 실제 WASM의 411문단·잔여 10,456문자 서식 및 HWP/HWPX 저장 재열기가 통과했다.
- 최종 Native/fresh WASM은 각각 원본 전체 7쪽, 독립 Print PDF와 쪽수가 같고 전쪽 raw 최저 100.00%다. 전체 TSV 원본과 같은 배율 직접 판독을 원장에 연결했다.
- 기본 생산자 입력은 최신 base와 같다. 새 배치·줄 구성·LineSeg/높이 추정이나 문서별 보정, baseline 완화·회귀 제거는 없다. 편람 조판 gate는 공통 조판 생산자 변경이 없어 비해당이다.
- 이번 새 diff는 Flutter/Dart/C++/Python 소비자다. Rust source/test를 새로 수정한 PR의 전체 lint/회귀를 이번 로컬 실행으로 보고하지 않는다. 새 통합 head의 [GitHub 필수 CI](https://github.com/edwardkim/rhwp/actions/runs/42786011478)와 [self-review](https://github.com/edwardkim/rhwp/pull/8907#pullrequestreview-5839510481)가 성공했다.

## 원 PR CI와 운영 범위

#8869·#8872의 원 head에는 Adapter preflight 취소가 있었다. 실제 로그에서 checkout이 끝나기 전
기존 3분 job 한도를 소진했으며 제품 검사는 시작되지 않았다. 기여자 코드의 assertion 실패로
분류하거나 취소 check를 성공으로 쓰지 않는다. 나머지 네 원 head에는 조회한 실패 check가 없다.
workflow·시간 상한·탐지 기준을 수정하거나 원격 재실행하지 않았다.

새 Mac 타일은 #8701 후속이며 기존 Mac native 경로를 보존한다. #536·#8701 전체 로드맵은
OPEN을 유지한다. 기존 fork branch와 다른 작업공간은 정리 대상이 아니다.

## 조판 원칙 준수 검토

실제 수정은 Flutter 소비와 인수·자원 소유 계약이며 Rust의 측정·조판·pagination 생산자를
바꾸지 않는다. 좌표는 기존 page metadata와 viewport 배치를 사용하며 근거 없는 값 조정·clamp로
화면을 맞추지 않았다. 수용한 기본 renderer의 같은 Print 입력을 Native/Canvas에서 직접
출력하고 변경된 GTK 소비는 CPU·compositor 실제 결과로 확인했다. 경계·문자 기준선·간격의
직접 판독과 기존 AA 차이도 원장에 기록했다. metadata/lease 합성 assertion을 한컴 출력 일치
증거로 승격하지 않는다. 이번 렌더링 범위에 새 절대 px·고정 쪽수 golden은 추가하지 않았다.

## Merge 후 contributor PR comment 계획

원격 승인과 정확한 head CI·self-review·merge를 완료한 뒤 이 PR의 고유 변경과 보정·한계를
명시한 감사 댓글을 게시한다. 실제 원본 7쪽의 Native/fresh WASM 최저 100.00%, 아래 고유 PNG와
[원장](../assets/seo-rii-20261110/README.md)의 Print 비교·전체 TSV를 연결한다.

- 고유 직접 확인 이미지: `mydocs/pr/assets/seo-rii-20261110/tile-resize.png`.
- Print 직접 표시: `radial-pages-1-3.png`, `bitmap-native-review.png`, `bitmap-wasm-review.png`.
- 확정 merge SHA의 raw URL: `https://raw.githubusercontent.com/edwardkim/rhwp/<merge-commit-sha>/mydocs/pr/assets/seo-rii-20261110/tile-resize.png`.
- 게시 전 asset의 merge 반영·PNG 형식·SHA-256, 게시 뒤 API의 본문·한국어·정확한 원 PR URL과
  이미지 표시를 확인한다. `--body-file`을 사용하고 아직 발생하지 않은 merge·댓글을 완료로 기록하지 않는다.
- 독립 비교 방법: [Visual Sweep 정본](../../manual/verification/visual_sweep_guide.md#github-merge-comment).

## 병합 후 확인

- 통합 merge `0f91d29ca81f00be3d5adb91857875b40ec5e355`, 2026-11-10 18:23:46 KST. 원 기여 head와 fork branch를 보존했다.
- [최종 CI](https://github.com/edwardkim/rhwp/actions/runs/42786011478)·[CodeQL](https://github.com/edwardkim/rhwp/actions/runs/42786011727)·Adapter·Proptest·[최종 정책 감사](https://github.com/edwardkim/rhwp/actions/runs/42789964690) 성공. 31개 check 성공·정상 skip 5개이며 재실행·보호 설정 우회는 없다.
- [공개 self-review](https://github.com/edwardkim/rhwp/pull/8907#pullrequestreview-5839510481)와 병합 직전 exact head/base·MERGEABLE/CLEAN·실제 병합 tree 일치를 확인했다. 원 PR의 간접 MERGED 상태를 API로 확인했다.
- 부모 [#536](https://github.com/edwardkim/rhwp/issues/536)·[#8701](https://github.com/edwardkim/rhwp/issues/8701)은 OPEN을 유지한다. 새 Mac native 타일 구현은 후속이다.
- 운영 기록 반영·devel 동기화 뒤 위 계획의 감사 댓글과 종료 정리를 수행한다. 최종 댓글 URL·정리 결과는 `target/pr-review/cleanup-seo-rii-20261110.json`에 기록한다.
