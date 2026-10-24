# Stage82: 독립 Print JPEG 전달의 공통 통합 후보

Issue: #7445. 부모 HEAD `b21f9e897d`와 base `d9fd85ed5bc3c4d9ce6b2981422e48bf9bb90867` 위에서
기존 C354 대조군의 실제 원인을 보정한다. 새 HWP 보정 대상·기준 PDF·비교 문턱은 추가하지 않았다.

## 독립 근거와 원인 계보

원 해결 PR #8064의 근사 bitmap 전달·cubic 계약에서 이어진 범위다. `ubuntu-ted2`의 별도
#8160 작업 HEAD `871b470437f39479ad8ba97506d328bd8f3921e4`와 독립 Print 전달 계약을 조사했다.
그 작업의 전쪽 수치를 이 브랜치의 통과로 사용하지 않고, 동결 Native `803046519`로 기존
C354 입력을 실제 실행해 100%를 확인했다. 같은 엔진의 C352 91.58691%, C356 33.33810%도
기록해 전체 브랜치 적용으로 현재 개선을 대체하지 않았다.

기존 C354는 RGB 색상 strip 때문에 한컴 Print에서 JPEG로 전달된다. 원본 PNG의 독립
품질 55/4:2:0 인코딩은 PDF JPEG와 디코딩 픽셀이 완전히 같았다. 원격은 독립 17/18/24/31색과
32/256색, 여러 크기·배율·출력 세대의 대조로 팔레트 분기와 폭 비례 품질을 확인했다.
그 source와 근거의 SHA·원본 라이선스는 ignored 조사 자료에 보존했다.

## 공통 생산·소비

`원본 불투명 BMP/PNG/GIF → Print LayerBuilder의 실제 DPI/배치 폭 → print_jpeg_payload →
ResolvedImagePayload::OutputDevice → 기존 Native/WebCanvas/CanvasKit 그림 소비`를 연결한다.
편집 원본·저장 BinData는 바꾸지 않는다. 효과·색 조정·투명도·실제 crop·명시적 PDF 보간 선언은
기존 경로를 유지한다. JPEG가 아닌 32색 미만 무손실 그림도 기존 면적 축소를 유지한다.
품질은 문서명이나 점수가 아니라 원본 폭과 실제 인쇄 폭으로 계산한다.

#8160의 JPEG 인코더 보정본은 파일별 SHA를 확인해 그대로 가져왔다. 원본 MIT/Apache-2.0/IJG
라이선스·저작권·upstream checksum·패치 고지와 구분된 내부 이름을 보존했다. GIF decode feature,
Cargo workspace 제외·정확 버전·lock, source 사전 검사 후보 그래프와 제삼자 고지도 연결했다.
Cargo metadata는 명시적인 의존성 변경 때문에 한 차례 lock을 생성했으며 추가 package는 이
로컬 fork 한 개다. 이후 Native 빌드는 `--locked -j 8`로 실행했다. registry 게시는 하지 않았다.

## 실제 Native 집중 검증

새 source의 native-skia CLI 빌드는 exit 0, 171초이며 동결 binary와 source hash를 기록했다.
독립 Print를 현재 canonical 96DPI/2px/90% 조건으로 비교한 결과다.

| 기존 대조군/쪽 | 보정 전 | 통합 후보 Native |
| --- | ---: | ---: |
| C352 | 100.00000% | 100.00000% |
| C354 | 94.57727% | 99.99600% |
| C356 | 42.89522% | 42.89522% |
| 로드맵 153쪽 | 이전 증거와 구분 | 100.00000% |
| 로드맵 295쪽 | 이전 증거와 구분 | 98.96693% |
| sample11 2쪽 | 이전 증거와 구분 | 99.83734% |
| sample11 50쪽 | 이전 증거와 구분 | 93.14605% |

세 대조군은 각 1쪽, 로드맵 302쪽·sample11 151쪽의 실제 전체 쪽수도 PDF와 같았다.
해당 두 문서는 위 네 쪽만 캡처했으며 전쪽 완료로 확대하지 않는다. C354 review/overlay를
직접 열어 세 격자·색상 strip의 보존과 오른쪽 경계의 남은 미세 차이를 확인했다.
C354는 현재 고정 devel 98.85995%보다 높아 이전 4.28268%p 하락을 이 Native 후보에서는 복구했다.
이 결과를 fresh WASM·CanvasKit·최종 PR head의 전쪽 수용으로 사용하지 않는다.

## 잔여와 증거 위치

C356은 여전히 미달이며 사용자가 거절한 예외를 적용하지 않았다. JPEG와 별개인 무손실 그림의
정수 종이 viewport/물리 프레임 소비를 이어서 보정한다. 새 source의 fresh WASM, 전체 nextest·
lint/build/Skia/manifest/unit tiers·source preflight와 기존 50조건/실제 소비 검증은 아직 남아 있다.
이 commit은 로컬 구현 체크포인트이며 PR 준비 완료가 아니다.

source·원격 archive/hash·Native producer·빌드 로그·입력/PDF·전체 집중 TSV·review/overlay는
ignored `rebase-d9fd85ed-20261023/print-bitmap-common-transfer/`에 보존했다. 초기에 제삼자 고지의
기존 행을 잘못 검색한 StopIteration은 같은 원격 고지의 정확한 행을 확인해 보완했다.
lock 생성·빌드 성공과 구분해 기록하며 이 오류로 빠진 고지나 부분 그래프는 남기지 않았다.
