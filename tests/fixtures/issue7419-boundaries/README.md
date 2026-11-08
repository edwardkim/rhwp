# #7419 저장 줄 혼재 입력

`mixed-child-storage.hwpx`와 `no-child-storage.hwpx`는 저장 줄의 유무가 다른 인라인
형제도 하나의 부모 프레임에서 소유·예약·이월되어야 한다는 경계를 검증한다.
기존의 다른 입력은 수동 XML 반례이며, 이 두 입력은 실제 한컴 저장 줄을 보존한 변형이다.

## 생성과 독립 기준

1. 같은 폴더의 `partial-nested-width-wrap.hwpx`를 한컴2024 13.0.0.3901에서 HWP로
   저장했다. job `94075ab2-d378-4fd3-9eee-0050640e0f9b`.
2. 그 HWP를 HWPX로 저장했다. job `7a141687-fbad-416c-8ae5-a5cee8d63e2d`.
   실제 저장 HWPX의 SHA-256은
   `3884350593f3322e0aa9bb699864db407e2c1e81e171e757ab497ba7eb2368c0`다.
3. `Contents/section0.xml`의 부모 표 `1000000009`에서 두 자식을 품은 문단
   `1000000011`의 `hp:linesegarray`를 제거했다. `mixed-child-storage`는 첫 자식의
   문단 `1000000006`, `1000000007`에서도 제거하고, 둘째 자식의 실제 저장 줄은 유지했다.
   `no-child-storage`는 추가로 둘째 자식의 `1000100006`, `1000100007`에서도 제거했다.
   표 크기·셀 속성·문단/글자 속성과 나머지 ZIP 항목은 보존했다.
4. 각 최종 입력을 한컴2024의 PrintMethod0, `frame_print_to_pdf_ex_one_up`으로 직접 출력했다.
   PDF는 `pdf/issue7419/<입력>-print-2024.pdf`에 있다. Print job은 각각
   `d98accb7-7e02-4387-90c7-06ff11afa887`, `ba87fd38-33b3-4a3d-9c95-c447e1bb74da`다.
   두 PDF는 각각2쪽이며 페이지별 raster가 동일하다.

| 파일 | 바이트 | SHA-256 |
| --- | ---: | --- |
| mixed-child-storage.hwpx | 23177 | `8758c38c662d09ccca949a4551a645aeef31a639e8f3c09b78ae2321513d53a4` |
| no-child-storage.hwpx | 23119 | `d323de426683eb052e5715015f11d41597d22308a23ef8e739e64db5893f2018` |
| mixed-child-storage-print-2024.pdf | 14039 | `5516c3b313cf2b9e78563e7ec2620cc498ecbfcb3cfc6eeb37f777fddb454668` |
| no-child-storage-print-2024.pdf | 14038 | `fc6966e6066793f917808cdae4f990ca461e7ae9cbe1fd0c77c28c054d7c202c` |

## 선행 검증과 검사 계약

소스 `dca310c06036c7f3fa7ca30cce74c1704f1169f2`의 Native/fresh WASM에서 두 입력
각각2쪽 모두2px 관용 실루엣100%, 독립 Print 쪽수 일치를 확인하고 직접 대조했다.
첫 자식은1쪽, 둘째 자식과 후속 본문은2쪽이다. 글리프 raster의 엄격 픽셀 일치는 주장하지 않는다.
관계 검사는 전체 내용·순서·누락·중복·셀 포함·쪽 소속·후속 본문 비겹침을 확인한다.
완료4쪽의 핵심 제목·후속 본문은 독립 PDF span.origin을96dpi로 환산한±2px 검사도 유지한다.

[Stage4 결과와 실행 명령](../../../mydocs/working/task_m100_7419_stage4.md)에 연결한다.
원시 SHA·변환 job·TSV·review/overlay PNG는 ignored
`output/pr-review/pr7433-current-20261016/{regression-fix/mixed-storage,mixed-storage-fix}/`에 있다.
