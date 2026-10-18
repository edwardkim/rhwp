# #7998 활성 내용 클립의 병합·분리 대조군

## 생성 방식과 독립 기대 구조

`generate_sources.rs`는 공개 `native-borders/print-source-inputs/solid-parent.hwpx`에서
정상 create/insert/merge/border API로 2×2 표를 만든다. 수동 LineSeg 수정은 없다.
`print-source-inputs/*.hwpx`는 그 생성본이다. 세 원본은 engine 2020 서비스의
PrintMethod 0·`hancom2020_pdf_driver_one_up`으로 인쇄했다(11.0.0.9136).
`colspan.pdf`는 위쪽 두 칸 병합, `rowspan.pdf`는 왼쪽 두 칸 병합으로 각각 실제 셀
3개다. `hidden-separate.pdf`는 위쪽 공유 선만 숨긴 **별도 셀 4개**다. 원 생성 구조와
PDF에서 선택할 수 있는 편집 IR을 대조하며, 임의 PDF의 작성 객체를 유일하게 안다고 주장하지 않는다.

`generate_controls.py --print-dir pdf/issue7998/cell-regions --output <dir>`는
독립 Print의 geometry/content를 보존한 채 활성 클립의 Q 복원·누락·부분 경계·비사각 bevel·
경합·실제 선과의 충돌·비어 있는 구역과, 변환된 Form/같은 F1 자원 이름 재사용을 구성한다.
Form은 자체 Arial 자원을 사용하고 caller의 F1은 Times-Roman이다. 원본 Print와
PDF 연산을 직접 바꾼 계약 입력은 서로 구분한다. 생성 시 PDF container ID가 바뀔 수 있으므로
아래는 검증한 입력 바이트의 hash다.

일반 클립 모양만으로는 셀 소유를 정하지 않는다. 실제 text-show의 invocation path와
활성 clip chain, 공통 glyph 관측, 완전한 지역 tiling 및 실제 분리선과의 비모순이 필요하다.
근거 부족인 빈 구역은 보류한다. 후속 [지역 축·빈 셀 계약](local-regions.md)은 사방 선이
직접 확인되는 빈 단일 tile만 별도로 수용한다. 원래 선의 부재와 의미상 셀 분리를 구분하고,
없는 공유 선을 새 실선으로 그리지 않는다.

## 입력과 선행 시각 검증

Native/fresh Docker WASM에서 각 입력의 전쪽 PDF 직접 열기·HWP/HWPX 저장 재열기를
검증했다(각 backend 36건·36쪽). 직접 확대 판독에서 선의 존재/부재, 문자 간격과 셀 소유를
대조했다. 아래는 입력별 **세 형식의 최저 raw 2px 실루엣 일치율**이며 평균이 아니다.
정식 검사에는 절대 px·전체 SVG hash 대신 셀 주소/span·내용 순서/보존·셀 포함·실제 잉크의
소유와 범위를 사용한다.

| 입력 | bytes | SHA-256 | Native 최저 | fresh WASM 최저 |
|---|---:|---|---:|---:|
| `colspan.pdf` | 18062 | `9d300df62e8ea1ccf9b3e7e4def4f21bd23e442e0e80e698a3db72e3990ed01c` | 98.65443% | 98.76543% |
| `competing-scopes.pdf` | 17611 | `7cb3990bbb857da9d8544ba037d7dd8f21d96ca729bcedc0c088f7643f6563f4` | 99.97921% | 99.97974% |
| `form-shadowed-font.pdf` | 17885 | `7fb91e65d73024ca7ab3f8b1c500033930e34e79b3509f72ca09fb236f44b7c6` | 98.08227% | 98.27632% |
| `form-transformed.pdf` | 17812 | `e166bd071a2c9977df49cb728aad01a1efc3dc039487ed23ba67d668ada77cc2` | 98.08227% | 98.27632% |
| `hidden-separate.pdf` | 17218 | `38110b3e53010b36a1b31f223454aa5dfeb3e6349807c2c8b68e6721cd4e358e` | 98.28549% | 98.42490% |
| `missing-clip.pdf` | 17591 | `42b5a749b0bcd0dfb8052782b648b0c9d81a0d5d53776c3b0f44e20ef30e5ae2` | 99.97921% | 99.97974% |
| `nonrectangular-clip.pdf` | 17595 | `4a6c13e12c5909add7244a49b81acefa332fdc04c762098759b3372472026482` | 99.97921% | 99.97974% |
| `painted-separator-conflict.pdf` | 17592 | `da3fb3a27feebe0e1067ee59c55cbd966b249336bd6a36024fdd4dc0d6a10e3e` | 99.97921% | 99.97974% |
| `partial-clip.pdf` | 17593 | `02f4b40ece76721bbd5327a7fdc089cd3362b747ffd2ed0d985e1ee9b56de330` | 99.97921% | 99.97974% |
| `restored-clip.pdf` | 17590 | `a5aaba8570fb1684b574135cbf580ce8765b59eaf0cd6b4948b6382a335a2ce5` | 99.97921% | 99.97974% |
| `rowspan.pdf` | 18047 | `ee2cbcf5ce04b57e4c82d0e8e0acee4bd31ddaf57048a8bed0421dd4cf3a006b` | 98.64803% | 98.69238% |
| `unwitnessed-cell.pdf` | 17570 | `0985d7d711b75cefaa261897070c3dbf0c7de63e42672dc26445becb08a3f59e` | 99.97692% | 99.97750% |


원래 삼각 clip 진단본은 실제 글자를 자르는 별개 입력이며 기존 text-clip 미지원 차이가
남아 로컬 진단에 보존하고 회귀 등록을 보류했다. 여기의 bevel 반례는 글자를 가리지 않는다.
그 결과를 삼각 진단본의 출력 일치로 바꾸어 보고하지 않는다.

## 실질적인 편집·저장과 글꼴 계약

가로/세로 병합과 숨긴 공유 선은 실제 Table/Cell IR이다. 문자 삽입/삭제와 병합 범위를
지나는 행열 삽입/삭제를 실행하고, 양 저장 형식에서 내용 순서·중복/누락·소유를 보호한다.
동일하게 편집한 한컴 Print와 실제 WASM 저장본의 전쪽 비교도 수행했다. 세로 실선 일부는
BorderFill 대신 셀 소유 원본 도형으로 보존된다. raw BorderFill 종류만으로 실제 선의 존재를
판정하지 않으며, 숨긴 위쪽 선의 부재와 아래쪽 선의 셀 포함·전체 길이를 최종 트리에서 확인한다.

Arial 프로그램의 OpenType nameID 1=`Arial`, nameID 6=`ArialMT`를 독립 대조했다.
검증한 파일 SHA-256은 `b3658eadae55e682b5f69eb64c439c1ecc8f196c0bb8d4756d145d13bc86476a`이며
폰트 바이너리는 포함하지 않는다. PDF BaseFont의 정확한 PostScript→family 쌍만 IR로 연결한다.
임의 suffix 제거는 하지 않으며, 원래 subset 이름은 source 관측에 남긴다. Form은 font가
활성일 때의 BaseFont를 보존하여 caller의 같은 자원 이름으로 다시 해석되지 않는다.

실제 producer·명령·원본 TSV·대표 PNG·수정 전 FAIL/후 PASS 및 판정 상태는
[stage8](../../mydocs/working/task_m100_7998_stage8.md)에 연결했다. 정상 원본 세 개의 Print와
저장·편집 Print 12개 응답은 로컬 장부에서 출력 계약까지 확인했다. 2026-10-19 메인테이너가
checkpoint `7e34a0140`의 병합 3셀·별도 4셀을 **“시각·편집 구조 통과”**로 수용했다.
제품 변경 없는 판정 기록이며 위 검증 결과를 재사용한다.

정상 paint 격자가 이미 성립하는 범위의 내용 기반 선택만 검증했다. paint 축 자체가 부족한
지역 후보의 일반 선택, L자/부분 격자, 빈 구역의 의미 복원, 임의 source paint의 일반 스타일
교체·새 행열 상속, 비사각 text paint 복원과 P5 최종 게이트는 남는다.

후속 P4 절편에서 `unwitnessed-cell.pdf`의 닫힌 빈 단일 tile는 정상 3셀로 수용하고 양 저장을
재검증했다. 이 입력의 과거 보류 상태는 현재 기대값이 아니며, 자세한 근거는
[local-regions](local-regions.md)·stage9를 따른다.
