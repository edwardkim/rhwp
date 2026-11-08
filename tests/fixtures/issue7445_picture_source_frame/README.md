# 그림의 원본 프레임·줄 점유·외곽선 소유

이 fixture는 #7445 리베이스 후 그림 보정의 최종 제출용 입력이다. 임시 진단 스크립트·로그·
JSON과 미채택 입력은 ignored output에 보존하고 여기에 포함하지 않는다.

## 입력과 독립 기준

| 입력 | SHA-256 | 독립 한컴 Print PDF |
|---|---|---|
| `public-picture-outlines.hwpx` | `b352bae1c818f19c0a44fec71495d8c5dc560a243f3d73c69b430a20c64d4227` | [2020 Print](../../../pdf/issue7445-public-picture-outlines-20261021-2020.pdf) |
| `nested-picture-border.hwpx` | `1a9b4dcd251dff00f391cb47901d041339ef435da6b335335fa4386f110ef53c` | [2020 Print](../../../pdf/issue7445-nested-picture-border-20261021-2020.pdf) |

첫 입력은 기존 `samples/issue_265.hwp`의 JPEG를 공개 API로 빈 문서에 삽입한 9개 그림이다.
실제 이미지100×44px, 본체4500×1980HU와 사방2000HU 바깥여백을 사용한다. 없음·가는 실선·
굵은 실선·점선·파선·30도 회전·36pt 외곽선·FLAT·SQUARE를 구별한다. 공개 setter에 없는
테두리 종류 enum만 IR에 지정하며 좌표·줄 높이는 수동으로 맞추지 않는다.

두 번째 입력은 기존 `samples/pic-in-table-with-toggle.hwp`의 문서·표·그림 위치를 유지하고,
테두리 비트만 정규화하여 첫 셀 그림 하나에 실선75HU를 지정한 HWPX 저장본이다.
기존 일반 셀 그림 경로를 실제로 호출하므로 두 번의 테두리 발행을 검출한다.

독립 기준은 각 입력 자체의 Hancom2020 Print, timeout1800/PrintMethod0이다.
일반 PDF 변환이나 다른 입력의 PDF로 대체하지 않았다. Print 작업은 각각
`d696eebd-b7c0-4a57-9e99-e4de52b92a6d`와 `0fd921fd-f573-4427-b3d9-a7a64d72bd70`이다.

## 정식 회귀 추가의 시각 선행 근거

최종 Native713와 locked `web/pkg/no-opt` fresh WASM714의 동일 소스를 사용했다.
공통 스크립트 `visual_sweep.py`, PDFium smooth_rgb_v1,192DPI/compat2022,
색상 경계 보정 전2px 실루엣90% 기준이다. Docker 최적화 결과가 아니다.

| 문서 | 쪽수 Native/PDF/WASM | Native 최저 | fresh WASM 최저 |
|---|---:|---:|---:|
| 공개 그림9종 | 1/1/1 | 99.90570% | 99.90577% |
| 셀 그림 테두리 | 1/1/1 | 100.00000% | 100.00000% |

Native693/701 및 WASM697의 review와 overlay를 직접 판독했고, 최종715의 대응 PNG는
697과102/102개 모두 바이트 동일하다. 입력·글꼴·producer·hash 비교 원장으로 판독 재사용을
연결했다. 회전 그림 본체의 크기·중심,
로고 보존, 굵은 선의 바깥 점유, 둥근 모서리와 FLAT/SQUARE, 셀 그림·표 경계·후속 그림을
확인했다. 점선·파선의 위상 및 가는 획의 작은 래스터 차이를 완전 일치라고 쓰지 않는다.
회전 전 크기45×19.8pt의96DPI CSS60×26.4px는 양쪽98% 이상인 완료 페이지의
독립 관측이므로 ±2CSSpx로 보호한다. 선택은 절대 영역이 아니라 원문 표지의 줄 소유를 따른다.

바깥여백 setter의 줄 메트릭 계약은 같은4500×1980HU 그림의 독립 Hancom 재저장666에서
높이5980HU·기준선5083HU·다음 줄 시작6580HU로 확인했다. 여백 제거 시 같은4000HU를
반환하는 관계를 검사한다. 줄 메트릭을 구현 helper의 반환값으로 재인용하지 않는다.
회전 행렬은 원PR1279/메인터너658868407c의 HWP 저장 계약과 독립 Print의 회전 모양을
기준으로 HWP/HWPX 양쪽 저장에서 비교한다.

## 수정 전후

`issue_7445_picture_source_frame`의 처음5개 정식 함수는 frozen Native656에서
0 PASS/5 FAIL, Native689에서5 PASS/0 FAIL이다. 원본 HWP3 테두리 비트와 저장 유지·첫 본문
Print 상태 검사까지 보강한 최종6개는 Native713에서6 PASS다. 같은6개를 Native656에서
실행하면5 FAIL/1 PASS이며, 아직 테두리 파싱을 고치기 전Native607에서 여섯 번째 함수의
동일 본문을 실행하면1 FAIL이다. Header/Footer를 한 match arm으로 묶었던 최초 컴파일
오류는 정정했고 제품의 RED로 세지 않았다. 단순 helper 존재나 전체 SVG 해시가 아니라
실제 줄·그림·셀의 소유와 전체/흐름 이미지 JSON의 최종 프레임을 검사한다.
자동 suite에서 관련5모듈39건 모두 실제 PASS했으며 이 중 신규6함수가 포함된다.
최종 source·빌드·범위와 남은 작업은 [단계19 기록](../../../mydocs/working/task_m100_7445_rebase_repair_stage19.md)에 연결한다.

최종715는 원본16쪽·현재 저장 HWP16쪽/HWPX16쪽·그림9종1쪽·셀 그림1쪽·기존90도 회전1쪽을
Native/fresh WASM 각각51쪽 검증했다. 이전697의 각18조건243쪽은 schema1.28·Studio 표면
보정 전 source의 결과로 구분한다. 실제 Studio의12개 초기 JS/WASM 응답 일치와 인쇄
backend 비교4 PASS를 확인했으나 글꼴 중복 캐시 응답 증적 및 화면 비교의 잔여가 있어
actual36 완료는 아니다. 전체 PR 게이트와 이 fixture의 선행 조건을 혼동하지 않는다.
