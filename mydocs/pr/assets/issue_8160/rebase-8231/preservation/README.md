# #8231 리베이스 후 기능 보존

새 base `238de6de41a208648f4ade6b494fe433a81c1fce`와 보정 producer
`e01e7540fd4c8a67ce5d1d1e5802170973acb998`를 같은 입력·독립 한컴 Print PDF·글꼴 공급으로
Native·표준 최적화 fresh WASM으로 실제 실행한 대조다. sample16 세 목표 파일의 각각 64쪽 raw100
증거는 [상위 기록](../README.md)에 있다.
편람 HWP/HWPX 두 파일은 이 검토에서 제외한다.

| 검증 | 결과 | 증거 |
| --- | --- | --- |
| Native 전쪽 대조 | 35조건·2,041쪽, 최저 91.66238%, 최대 하락 0.08523%p | [페이지별 대조](native/paired-pages.json), [입력·PDF·producer·명령·전체 TSV](native/proof.json) |
| WASM 전쪽 대조 | 35조건·2,041쪽, 최저 91.70207%, 최대 하락 0.09506%p | [페이지별 대조](wasm/paired-pages.json), [입력·PDF·producer·명령·전체 TSV](wasm/proof.json) |
| 본문 소유 | Native·WASM 각각 전 35조건의 페이지·가용 소유키별 본문 문자 순서 동일 | [Native](native/paint-owners.json), [WASM](wasm/paint-owners.json) |
| 주요 영역 직접 판독 | 기호·앵커 표·반복 제목·교육과정 셀·닫는 선 등 backend별 17쪽 확인 | [Native 이미지·판독](native/inspection.json), [WASM 이미지·판독](wasm/inspection.json) |
| Studio 소스 계약 | base·후보 각각 203 PASS | [실제 비시각 소비 검사](native/nonvisual.json) |
| 글상자 편집·저장 | base·후보 각각 3 PASS, 생성·붙이기·제거·HWP 저장/다시 열기 | [실제 비시각 소비 검사](native/nonvisual.json) |
| Windows 체크아웃 설정 | Linux Git에서 `core.autocrlf=true`, `core.symlinks=false`로 12경로씩 24건 확인 | [바이트·설정 대조](native/nonvisual.json) |
| 관련 Rust 검사 | #8160 기본 85건과 새 devel 보존 51건, 합계 136 PASS | [선택 실행 범위](native/nonvisual.json) |

소유 대조는 공백을 제외한 paint 문자와 실제 제공된 페이지·소유키의 대조다. 그림 내부 문자나
IR 전체 동일성으로 확대하지 않는다. Windows 설정 대조는 실제 Windows에서의 실행을 뜻하지 않는다.
관련 Rust 136건은 필터 선택 실행이며 전체 nextest 완료가 아니다.
전쪽 대조는 양쪽 backend 모두 0.1%p 이상 하락·90% 미만·누락·쪽수 불일치가 없다.
WASM은 모든 PNG의 실제 export receipt 해시와 전쪽 render tree 수를 확인했고, 소유 대조의
각 조건에 실제 본문 소유키가 존재함을 확인했다. 빈 대조를 동일성으로 판정하지 않았다.

온새미로 변환본은 기존 #7881 검증에서 독립 Print로 확인한
[명시적 글꼴 환경](../../../issue7881/current-body-frames-20261025/print-reference-environment.json)을
양쪽에 적용한다. 최초 Native 대조 명령에서 이 설정을 누락해 기본 환경의 46쪽이 양쪽 87.14432%였다.
그 실패 출력은 별도로 보존했다. 기존 설정을 복원한 전 46쪽 재실행은 base 최저 94.80229%,
후보 최저 94.80197%이며 46쪽 TSV는 양쪽 100.00000%다. 문서·PDF·문턱과 제품 코드는 바꾸지 않았다.
WASM도 같은 환경에서 양쪽 최저 94.60320%, 46쪽은 양쪽 100.00000%다.

415쪽 교육과정 문서는 canonical exporter의 지원 옵션 `protocolTimeoutMs=3600000`을 양쪽에
적용했다. 기본 10분 CDP timeout 실패와, 임시 호출 wrapper에 `__main__` 보호가 빠져
multiprocessing 자식이 재진입한 실패는 ignored output에 보존하고 통과 자료에서 제외했다.
wrapper를 바로잡은 뒤 실제 2쪽 multiprocessing 실행의 exit 0을 확인하고 양쪽 415쪽을 다시
산출했다. 나머지 조건의 timeout은 600000ms다. 제품·픽셀 문턱·비교기는 바꾸지 않았다.
[base 빌드](wasm/base-build.json)와 [후보 빌드](wasm/head-build.json)는 서로 다른 실제 producer를 기록한다.

실제 Studio·Canvas·CanvasKit 소비는 [별도 기록](../consumer/README.md)에 있다.
[최종 로컬 검증](../validation.json)은 전체 Rust 11,573 PASS·45 skip, Native Skia 라이브러리
4,040 PASS·12 ignore 및 공식 누락 그림·직접 PDF·#8160 Native 검사 19 PASS다.
CI·원격 제출·머지 완료로 표시하지 않는다.
