# PR #9472 검토

[seo-rii 님의 원 PR](https://github.com/edwardkim/rhwp/pull/9472)

## 검토 버전

- 원 PR head `e36933ed4062fbf5f5965d3077514f75a114c811`
- 기준 devel `3a6b7d2b318287361f443b0814702783eb973533`
- 통합 코드 후보 `8550d0698f54fda3d66bee84232e753bb4599185`
- 로컬 회귀 실행 head `b45dfda9a3de46438d2dd14ea473bd96ec6e6c81`
- 검토 브랜치 `review/integration-8506-native-layout`

| 원본 커밋 | 체리픽 커밋 |
| --- | --- |
| `e36933ed4062fbf5f5965d3077514f75a114c811` | `4613e5ad44ed0d62dd554fe4030dd8e8314463ca` |

## 검토 범위

공개 육아기 근로시간 단축 신청서를 대상으로 C#에서 통합 후보의 Native Skia DLL을 직접 불러오는 콘솔 스모크입니다. 원 PR에 누적된 선행 14개 커밋은 #9473에 반영된 체리픽 SHA와 조상 관계를 확인했습니다. 이번에는 C# 검사와 실행 문서 커밋 1개를 적용했습니다. [이전 통합 #9473](https://github.com/edwardkim/rhwp/pull/9473)

### Windows x64 실행

win10-ted의 Windows x64 PowerShell에서 .NET SDK 10.0.400과 MSVC Native Skia 빌드를 사용했습니다. 스모크와 미저장 Print PDF가 성공했습니다. DLL SHA-256은 `5e64b810e172f6d0d97647d14bc2567724374784de490ca6a64329de6904a3f5`입니다. 기존 명령의 .NET 10 복원 오류 NETSDK1005를 Windows에서 재현했습니다. `e83eb5774feb652a0732dcf7aca8c0525615359f`에서 빌드와 DLL 실행 명령을 명시했고, 보정한 명령을 Windows에서 다시 실행해 통과했습니다.

신청인 이름의 편집·실행 취소·다시 실행, 생년월일 IME 확정, 자녀 생년월일 보존, 미저장 첫 쪽 Print PDF를 검사합니다. HWP·HWPX 각각 두 번 저장·재열기, 이전 문서 ID 거절, 취소 또는 이미 완료된 요청의 일관성, worker 해제 후 접근 거절도 확인합니다. 이 PR의 실행 범위는 .NET 콘솔과 Native C ABI입니다. [스모크 README](../../../bindings/csharp/tests/README.md) · [공개 신청서](../../../samples/issue5599_oracle/3191107_leave_request_form.hwpx)

## 병합 후 원 PR 안내

Windows x64의 DLL·.NET SDK와 실제 검사 범위를 안내합니다. 이 스모크는 #8701과 #536에서 추적합니다.

[통합 검증 결과](../assets/integration-8506-20261128/local-verification.json)
