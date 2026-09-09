## 문제와 수정

다운로드로 자동 열린 문서를 저장하면 전체 로컬 경로가 파일명 앞에 붙었다.
Chrome/Firefox의 viewer URL 조립 경계에서 basename만 추출해 한글·공백·괄호·밑줄을 보존한다.
다운로드 상태 머신과 Studio 저장 로직은 변경하지 않는다.

Closes #6961

## 검증

- shared/sw·두 adapter: 171/171, 두 확장 production build, dist 계약 3/3.
- 실제 Chrome 다운로드 4/4: 파일명 및 탭 수 검증.
- 실제 Firefox 155.0.1: 저장/다른 이름으로 저장 정상 basename, 저장본 재파싱으로 편집 보존 확인.
- Firefox의 기존 탭 중복/자체 저장 재오픈은 수정 전 기준본과 대조하고 #6964로 별도 분리했다.
- Rust/WASM source 무변경, 기존 WASM 재사용.

배포 전 #6964와 결합한 탭 수 검증이 필요하다.
