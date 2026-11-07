# #8733 한컴 표 배경 저장 수정

- 기준: devel b8dd67c5001dfb0410e9e192528676396cdd6a4e
- 작업 브랜치: task/8733-solid-fill-hancom
- 확인일: 2026-11-07 UTC
- 대상: https://github.com/edwardkim/rhwp/issues/8733
- 비교 자료: https://github.com/edwardkim/rhwp/issues/8733#issuecomment-6702261909

## 원인과 수정

표/셀 배경 대화상자와 공통 BorderFill 생성기가 새 단색 배경의 무늬 없음에 0을 사용했다. 첨부 HWP와 신고자의 한컴 출력에서는 이 값이 가로 무늬로 나타난다. 새 단색의 무늬 없음은 -1로 저장한다.

- 두 표 배경 대화상자의 무늬 없음 선택값을 -1로 변경했다.
- 원본에서 읽은 0은 숨김 보존 선택지로 유지한다. 테두리만 바꾸거나 확인할 때 NaN 또는 -1로 치환하지 않는다. 미리보기에는 가로 무늬를 표시한다.
- 배경이 없는 대상으로 대화상자를 재사용하면 무늬 선택을 -1로 초기화한다.
- 공통 생성기는 부분 편집에서 생략한 기존 배경색, 무늬색, 무늬 종류를 유지한다. 기존 배경이 없는 새 단색만 흰색/검정색/-1을 기본값으로 쓴다.
- HWP 바이너리 writer와 양수 무늬 매핑은 변경하지 않았다. 이 변경은 모든 무늬 형식의 전면적인 매핑 수정이나 기존 손상 문서의 자동 복구가 아니다.

## 너비와 간격

첨부 HWP의 일곱 표에 저장된 공통 FLAGS 0x04200317은 #7606의 이전 Table.attr 전체 덮어쓰기 결과와 일치한다. 이 값에는 PAPER 크기 기준과 affect_line_spacing 비트가 포함된다. #7608은 현재 devel에 이미 반영되어 있으므로 compositor나 크기 계산을 중복 변경하지 않는다.

https://github.com/edwardkim/rhwp/pull/7608

기존 #7606 회귀에 새 TAC 표의 줄 간격 영향 비트가 꺼져 있고 빈 속성/위치 편집 후에도 유지된다는 검사를 추가했다. 오래된 손상 문서의 기존 FLAGS를 임의로 정규화하지 않는다.

## 검증

- UI 수정 전: 새 회귀 9건 모두 실패.
- UI 수정 후: 새 회귀 9건 모두 통과.
- Rust 수정 전: 새 저장 회귀 3건 모두 실패. 새 단색의 0/-1 불일치와 부분 편집의 무늬색 소실을 재현했다.
- 고정 base 대비 Rust suite manifest 및 unit-tier 정책: 통과.
- fresh WASM dev 진단 빌드 및 최종 optimized release WASM 빌드: 성공. wasm-pack 0.15.0, Rust 1.93.1, locked wrapper, wasm-opt -O, exit 0.
- 실제 Chrome 155 + 최종 optimized release WASM: 셀과 표 대화상자 모두 새 단색 -1, 취소/재열기, HWP/HWPX 저장/재열기, 원본 0 보존, 명시적 무늬 제거, undo/redo 통과. browser-results.json의 success=true 및 브라우저 오류 0건을 확인했다.
- Native root Clippy: 통과. TypeScript noEmit: 통과.
- 중단 복구 시 이전 focused Native 실행이 잘못된 파생 shard를 선택해 0건만 실행했음을 확인했다. 이를 통과 근거로 사용하지 않았으며 현재 파생 shard에서 #8733과 #7606을 각각 3건씩 재실행해 통과했다.
- 전체 Studio 최종: 설치된 Node 24.22.1에서 2,756건 중 2,754 통과, 2 skip, 실패 0건(108.42초, exit 0). 기존 #8102 fixture의 URL.pathname이 Windows에서 /C:/ 경로를 생성하는 문제를 fileURLToPath로 최소 보정했다. 해당 fixture의 12건과 TypeScript도 별도 통과했다. 제품 코드나 테스트 기대값은 바꾸지 않았다. 초기 Node 22 및 Windows 경로 실패 기록은 진단 로그로 보존한다.
- WASM Clippy: 통과(exit 0, 3분 33초). #8733 focused Native: 3건 통과(exit 0), raw HWP -1 저장 및 HWP/HWPX roundtrip 확인.
- #7606 focused Native: 3건 통과. 빈 속성과 위치 편집 후 절대 크기 기준 및 줄 간격 플래그 보존을 확인했다.
- workspace build: 통과(exit 0, 4분 20초). workspace all-target Clippy: 통과(exit 0, 3분 20초).
- 전체 release-test: 55개 바이너리, 12,767건 통과, 45건 skip, exit 0. 명령 전체 40분 33초(컴파일 15분 12초 포함).

새 Rust 검사는 HWP DocInfo의 BorderFill 원시 바이트와 HWP/HWPX 재열기 결과를 검증한다. 기존 가로 무늬 0 및 양수 무늬의 HWP 저장 보존, 배경색만 바꾸는 편집, 무늬만 없애는 편집을 포함한다. 한컴 2020(11.0.0.9166)은 설치되어 있으나 지원 도구의 앱 실행 승인이 시간 초과되어 문서를 열거나 직접 인쇄하지 못했다. 승인 우회나 원격 업로드는 하지 않았다. 전체 양수 무늬 형식 간 대응도 이번 수정 범위가 아니다.

## 제출 후보와 증적

사용자가 남은 검증과 PR 제출을 승인한 후 최종 optimized WASM 및 Studio 전체 검증을 완료했다. 코드 commit은 `cf88d4c70`, Windows 테스트 경로 보정은 `e551fcc76`이며, Rust 전체 검증 이후 제품·Rust 테스트 소스는 동일하다. 이후 추가되는 파일은 보고서와 검증 입력뿐이다.

- 공개 가능한 합성 HWP/HWPX, 대화상자 PNG, 파일 해시와 검증 집계: [검증 입력](../pr/assets/issue_8733/README.md).
- 한컴 직접 인쇄는 앱 실행 승인 시간 초과로 미검증이다. 이를 통과했다고 표시하지 않으며, 양수 무늬 번호 전체 매핑이나 기존 손상 문서 자동 복구도 주장하지 않는다.
- 생성된 `rhwp-studio/public/` WASM glue와 WASM 바이너리는 제출에서 제외한다.
- 이 보고서는 제출 후보의 로컬 검증 기록이다. 원격 CI 결과는 PR의 정확한 head에서 별도로 확인하며 merge는 사용자 승인 없이 수행하지 않는다.
