# #8160: #8231 리베이스 후 세 문서 전쪽 raw100

보정 producer `e01e7540fd4c8a67ce5d1d1e5802170973acb998`의 Native와 표준 최적화 fresh WASM을
각각 새로 실행했다. 아래 세 파일은 각각 64쪽이며, 여섯 실행의 총 384개 PNG에서 양방향 정수
미대응이 모두 0이다. [전쪽 PNG 해시·입력/PDF·실행 신원과 정수 비교](raw100-proof.json)에 연결한다.

| 원본 | Native 전 64쪽 | 최적화 fresh WASM 전 64쪽 |
| --- | --- | --- |
| `hwp3-sample16-hwp5.hwpx` | [100%](native/hwpx.tsv) | [100%](wasm/hwpx.tsv) |
| `hwp3-sample16-hwp5-2024.hwp` | [100%](native/hwp2024.tsv) | [100%](wasm/hwp2024.tsv) |
| `hwp3-sample16-hwp5-2022.hwp` | [100%](native/hwp2022.tsv) | [100%](wasm/hwp2022.tsv) |

96 CSSpx Print, 잉크 문턱 232, 양방향 Chebyshev 반경 2px, 흰색 패딩의 raw 실루엣 계약이다.
RGB 완전 동일성을 뜻하지 않는다. 점수의 반올림만 확인하지 않고
[`verify_issue_8160_raw100.py`](../../../../../scripts/verify_issue_8160_raw100.py)가 실제 PNG를 다시 읽어
정수 미대응 0/0, 페이지 누락·추가 없음, 입력과 독립 Print SHA를 확인했다.

[WASM 빌드 기록](wasm/build.json)은 clean source·release·standard·`wasm-opt -O` 완료와
실제 패키지 해시를 기록한다. WASM SHA-256은
`c767cd8a3d1550d1310130bf5a5abcc2cefe8714e257896a7b94b8053b12b45c`다.
Native 신원은 [별도 기록](native/proof.json)에 있다. 후속 커밋의 검사·문서 변경을 렌더 producer로
바꾸어 표시하지 않는다.

전쪽 100%를 확인한 뒤 [상위 WMF 클립 회귀](../../../../../tests/cases/issue_8160_wmf_clipped_print.rs)를
정식 등록했다. 불투명 원본 비트맵의 두 blit과 선택적 장치 클립을 공개 DocumentCore API로 소비한다.
클립 없는 정상 대조군과 클립이 있는 경우의 인쇄 재생·공유 image 소유·원본 클립 영역을 보호한다.
문서명·특정 쪽 좌표로 제품 동작을 분기하지 않는다. #8160은 41개 모듈·87개 검사이며
기본 86개와 `native-skia` 전용 1개다. 최종 본문은 같은 내용의 수정 전 실패·수정 후 통과를 확인했다.

새 devel의 [Native·WASM 각각 35조건·2,041쪽 보존 및 관련 비시각 검사](preservation/README.md)는 통과했다.
실제 Studio·Canvas·CanvasKit 소비와 최종 WMF 검사 본문의 수정 전/후 재현은
[별도 증거](consumer/README.md)로 확인했다.

[최종 로컬 검증](validation.json)은 검사 원본 `949195dba653cdef8d8fa1506131f5db2f669e78`의
14개 필수 단계 모두 exit 0을 기록한다. 전체 nextest는 11,573 PASS·45 skip이고 #8160 기본 86건도
모두 포함한다. Native Skia 라이브러리는 4,040 PASS·12 ignore, 공식 누락 그림 2건·직접 PDF 출력
14건·#8160 문단/이중 테두리 3건도 모두 통과했다. 마지막 3건 중 1건이 #8160 Native 전용 검사다.
선택 실행에서 제외된 검사를 통과 수에 합산하지 않는다.

`076b19d36`까지의 후속 커밋은 검증 증거·문서만 추가했다. 이후 [Native 전용 CI 연결 보정](native-ci-registration.json)은 기존 job의 논리 case 호출과 소유 목록을 연결한다. 렌더 producer와 Rust 검사 본문·기대값·입력/PDF는 바꾸지 않았다.

[게시용 Native/WASM review·standalone overlay](publication/panels.json)는 검증을 마친 같은 최종 PNG에서 만들었다. [원본·Print 75개 commit 바이트 확인](inputs-in-commit.json)과 함께 원격 검토에 사용한다.
이 자료를 CI·원격 제출·머지 완료로 표시하지 않는다. 편람 HWP/HWPX 두 파일과 문서 정오표는
이번 검토 범위에서 제외한다.
