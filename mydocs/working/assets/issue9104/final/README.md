# #9104 최종 로컬 검증 증거

제품 source는 `1da7afbd64bbc59d9338a187ebcd21860ddc0842`, 최종 Rust 검사 head는
`13bb8191967f7e2efc9c182deaf0e7e075576205`, 기준 devel은
`4f44355f5ff234abd1289d340cb4a3fe13297982`다. 검사 뒤 제품·Rust 검사 source는 바꾸지 않았다.
이 원장은 로컬 검증 완료 시점의 증거다. 이후 원격 코드 CI는
[PR #9147 검토 기록](../../../../pr/archives/pr_9147_review.md)과
[CI 영수증](../../../../pr/assets/pr9147/code-ci.json)에서 구분한다.

## 결과

| 검증 | 실제 결과 | 증거 |
| --- | --- | --- |
| 최소 공개 글꼴 전체 Rust 회귀 | 13,710 PASS, 0 FAIL, 45 skip | [최종 검사](final-gates.json) |
| Native 공식 3종 | 4,073 PASS | [source/범위 동등성](native-scope-equivalence.json) |
| 실제 Native 진단 집중 검사 | 12 PASS | [집중 검사](native-observation-focused.json) |
| fmt·Native/WASM/workspace Clippy·workspace build·suite/unit 정책 | 모두 PASS | [최종 검사](final-gates.json) |
| 정본 보호 | 각 1,771쪽, 총 3,542행, 하락·누락 0 | [전쪽 검증](visual-validation.json), [행별 대조](visual-validation.tsv) |
| Native/fresh WASM raw 최저 | 98.86043% / 98.86472% | [Native TSV](protected/native), [WASM TSV](protected/wasm) |
| 정확한 devel 전쪽 대조 | 하락 0, PNG 3,536쌍 동일, 변경 6쌍 직접 확인 | [전체 비교](devel-whole-comparison.json) |
| 새 devel 7개 PR의 66개 문서·74쪽 | 양 backend 각각 전쪽 PNG 동일 | [Native](incoming-native-base-comparison.json), [WASM](incoming-wasm-base-comparison.json) |
| 편람 필수 gate | 실제 Native HWPX 383쪽 PASS | [gate](handbook-8153-gate.json) |
| 새 공개 입력의 보안 검사 | 3종 clean, Git blob 동일 | [보안 검사](public-input-security.json) |

중첩 표 10쪽은 양 backend 모두 raw 100.00%다. 두 형식 편람의 60·63쪽도 양쪽
100.00%이며, 원문 입력란과 밑줄의 소유·연속성·후속 본문을 확대 대조했다.
[선행 1,566행](visual-prerequisites.json), [직접 판독](direct-review.json),
[실제 변경 영역](handbook60-changed-body-detail.png)을 함께 확인한다.
회색 서식 영역의 기존 Canvas 색 차이는 같은 devel의 해당 영역과 픽셀이 동일하다.
실루엣 100%를 전체 색상·전체 픽셀 동일이라는 의미로 사용하지 않는다.

## 실제 진단과 음성 대조

- [원 재현 문서](../inputs/offline-font-families.hwpx)는 공개 API로 만든 10문단 진단 입력이다.
  한컴 Print 좌표 정본으로 사용하지 않는다.
- [오프라인 실제 결과](cjk/render.json): 설치된 CJK face의 대체 선택 8건을 관측했고,
  `missingRequests`는 비어 있다. [실제 PNG](cjk/offline-font-families.png)에서 10문단을 확인했다.
- [실제 네트워크 실패](cjk/actual-network-failure-validation.json)는 네트워크 namespace를
  차단하고 cache를 비운 실행이다. 오프라인 환경 플래그를 설정하지 않았으며 다운로드 실패와
  실제 로컬 대체 8건을 각각 확인했다. [JSON](cjk/actual-network-failure.json)을 남긴다.
- [일반/굵게/기울임](cjk/styled-cli-validation.json): 실제 공개 API로 서식을 바꾼 입력에서
  요청 스타일을 구분했고 진단 on/off PNG가 바이트 단위로 같았다.
- [metadata-only](cjk/metadata-only.json)는 `notRendered`·0 records이며 PNG를 만들지 않는다.
- [정확한 devel 음성 대조](cjk/base-negative-control.json)는 PNG가 후보와 같지만 실제
  대체 진단이 없어 새 요구를 검출한다.
- 말미 공백 관계 검사의 수정 전 결과는 [10 PASS / 1 FAIL](tail-negative-control.log),
  후보의 집중 결과는 [11 PASS](tail-positive-diagnostic.log)다. 최종 전체 회귀에서도
  네 새 검사와 provenance 검사가 실제 PASS했음을 [최종 검사](final-gates.json)에서 확인했다.

## 재현 조건과 자료 무결성

[도구 버전](tool-versions.json), [최소 글꼴 노출](minimum-font-exposure.log),
[실제 글꼴 프로그램 해시](actual-font-program-audit.json),
[기존에 커밋된 font environment의 신원](font-environment-identities.json)을 기록했다.
글꼴 프로그램은 커밋하지 않았다. 공개 테스트 자원 또는 사용 권한이 있는 로컬 원본을 준비하고,
[pool 재구성](font-pool-reconstruction.json)의 SHA와 파일명을 검증해 ignored 검증 폴더에 공급한다.
1,002개 항목 모두 지속 원본이나 고정 source에서 재구성할 수 있다. 변환 프로그램 11개는
FontTools의 시각 갱신을 끄고 원래 테이블 순서를 보존하여 바이트 단위로 같은 결과를 확인했다.

[글꼴 조건](font-conditions.json)의 dataset별 공급과 기존 font environment를 적용한다.
실행 당시 절대 경로는 출처이며 새 환경의 경로로 옮겨 해시를 다시 확인한다.
전쪽 검증 JSON의 입력/PDF 신원은 제품 commit의 Git blob과 대조했다.
763쪽 문서의 WASM 외부 그림 3개도 [원본 신원](linked-image-identities.json)으로 고정했다.
같은 dataset의 전쪽은 한 조건의 생산본이며 페이지별 최댓값을 합치지 않았다.

[검사 참조 57개 점검](target-assertion-audit.json)에서 기존 관계 검사와 독립 근거가 있는
완료 페이지의 좌표 검사를 유지했다. 기존 핀 교체·fixture 제거는 0건이며 새 관계 검사 4건을
추가했다. 다른 원본의 검사를 같은 파일에 있다는 이유로 재작성하지 않았다.
[공통 계약 검토](implementation-review.md)와 [실제 SVG 출력 5건](svg-consumption.json)을 남긴다.
SVG의 내장 글꼴 프로그램은 배포하지 않고 실제 출력의 manifest·해시만 보존했다.

보호 하한과 consumer 해시의 후속 갱신은 [보호 정본](../../../../pr/assets/visual-protected-baseline/README.md)과
[상승 2행 영수증](floor-update.json)에 연결한다. 원격 CI와 병합 상태는 로컬 실행 결과와 구분한다.

[하한 갱신 후 연결 검사](final-baseline-consumer-validation.json)에서 consumer는 표 해시만
바뀌고 Rust source·검사는 같음을 확인했다. 실제 완료된 편람 383쪽의 canonical compare도
새 해시에서 PASS다. [관련 Python 계약 267건](baseline-python-contracts.json)과
[새 정본/실제 측정 3,542행 동일](updated-baseline-matches-actual.json)을 확인했다.

#9143에 수용된 #9022·#9105·#9107·#9109·#9127·#9137·#9141의
[기능 보존](incoming-feature-preservation.json)은 같은 최신 base/후보 출력과 최종 source의
실제 동작 검사 38건으로 확인했다. 과거 생산자의 검사 결과를 현재 실행으로 대체하지 않았다.
