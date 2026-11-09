# 의존성·원 CI 원인 검토

기준 devel은 `024d610633ede166ebcbb915f9c90e7a9d0649d8`, 결합한 제품·의존성·workflow 검증 source는 `2320351d2325a41d900e91f4e07f559d4a924a7d`다. 원 PR #8811–#8829의 원 head를 각각 보존했다. 이 문서는 조사 결과이며 최종 검증 완료·원격 통합 결과와 구분한다.

## Rust 10건

| PR | 실제 소비 범위 | 확인한 변경·검증 목적 |
| --- | --- | --- |
| #8811 zip 9.4.0 | HWPX 읽기·쓰기, 암호 HWPX, lineage bundle | 새 병렬 API를 활성화하지 않는다. 기존 deflate·크기 한도·저장 재열기 계약을 확인한다. |
| #8814 log 0.4.35 | 공통 진단 기록 | logger 설정·level·로그 인자·비밀 값 처리 코드는 바꾸지 않는다. |
| #8817 cfb 0.16.0 | HWP5·BinData OLE 컨테이너 | FAT/DIFAT를 sector 단위로 읽도록 바뀐다. MiniFAT는 chain 바이트와 u32 vector가 잠시 함께 존재한다. 원 bounds·sector 순환 검사와 RHWP 자원 한도를 완화하지 않으며 CFB·손상 입력·저장 재열기를 검증한다. 성능 향상이나 메모리 사용 불변을 추정하지 않는다. |
| #8819 unicode-properties 0.1.5 | 공통 paint의 글리프 category 판정 | Unicode 분류 소비가 글꼴 fallback에 연결되므로 Native/fresh WASM 출력과 편람을 확인한다. |
| #8822 ml-kem 0.3.4 | security_trailer의 ML-KEM-768 공개키 봉인 | 공개 컨테이너 호환·왕복·잘못된 개인키·변조 거부 계약을 확인한다. |
| #8823 ml-dsa 0.1.2 | pq_sign의 ML-DSA-65·hybrid 서명 | 서명·검증·잘못된 키·변조 거부를 확인한다. upstream의 선택적 zeroize 보강을 RHWP의 모든 feature에서 활성화한 것으로 표현하지 않는다. |
| #8824 svgtypes 0.16.2 | EMF/WMF의 path·transform·font-family | CSS 색 문법·비 ASCII 식별자 지원과 잘못된 transform-origin의 panic 방지 변경이다. 기존 그림·글꼴·색 출력이 유지되는지 대조한다. |
| #8825 crc32fast 1.5.4 | 컨테이너 checksum·암호 HWP3 | 기존 checksum·암호 보호·해제와 잘못된 입력 거부를 확인한다. |
| #8826 ureq 3.5.1 | Native 공개 웹폰트 다운로드 | 요청의 15초 global timeout·redirect 0·응답 크기 한도는 유지한다. 비어 있는 별도 cache에서 기존/후보 CLI의 실제 HTTPS 다운로드 영수증도 대조한다. |
| #8827 encoding_rs 0.8.44 | HWP3·legacy 문자열·WMF 문자 변환 | 한국어·legacy 인코딩 및 실제 문서의 내용 보존을 전체 회귀와 출력으로 확인한다. |

직접 갱신 10종에 `ctutils`·`module-lattice`·`ureq-proto`의 전이 버전 교체가 더해져 Cargo.lock delta는 13종이다. 공식 crates.io archive의 SHA-256을 lockfile checksum과 대조했다. 선언된 MSRV는 프로젝트 Rust 1.93.1 이하이며 선언이 없는 crate도 실제 빌드·lint 대상에 포함한다. 정확 버전 13종의 OSV 응답에 advisory ID가 없었다. 이는 전체 의존성의 완전한 보안 감사를 의미하지 않는다.

[서드파티 목록](../../../../THIRD_PARTY_LICENSES.md)은 이번 갱신 대상의 버전·라이선스를 현행화했다. `unicode-properties` registry archive의 Unicode 고지는 기존 `assets/licenses/Unicode-3.0.txt`와 SHA-256 `e7a93b009565cfce55919a381437ac4db883e9da2126fa28b91d12732bc53d96`로 동일하다. SDK의 실제 라이선스 Apache-2.0도 기록했다. 이 목록은 Rust에 컴파일되는 입력과 분리된 배포 고지 문서다.

CFB 변경은 [공식 0.15.0…0.16.0 비교](https://github.com/mdsteele/rust-cfb/compare/v0.15.0...v0.16.0)와 대조했다. ML-KEM·ML-DSA의 비밀 값 Debug 출력·상수 시간 처리·zeroization 관련 upstream 보강은 실제 선택된 feature와 구분한다.

## npm 5건

Studio의 SDK 1.3.0→1.6.0과 Vite 8.3.5→8.3.6, Chrome·Firefox의 같은 Vite 갱신, VS Code의 webpack-cli 7.2.3→7.3.1을 검토했다. `dependency-audit`의 lockfile 전후 비교는 dev 의존성을 포함한 실제 변경 graph를 대상으로 했다.

- Studio: 변경 node 20개, 추가·제거 0개, flagged 0개. SDK 외 Vite·rolldown·platform binding·PostCSS·OXC의 전이 변경도 포함한다.
- Chrome·Firefox: 각각 변경 node 19개. 각 node의 변경 전후 전체 metadata가 이미 감사한 Studio graph의 해당 node와 같음을 대조한 뒤 결과를 재사용했다.
- VS Code: 변경 node 2개, 제거 22개, 추가 0개, flagged 0개. webpack-cli와 rechoir 변경이다.
- 새 설치 스크립트·출처 교체·identity mismatch·보고된 취약점이 검출되지 않았다. 기존 graph 전체나 실행 중인 계정의 API 권한을 감사한 결과로 확대하지 않는다.
- 실제 Node 24.15.0에서 네 디렉터리를 설치했다. Studio는 ignore-scripts 설치 뒤 별도 rebuild의 종료 코드까지 확인했다.
- SDK의 Bearer 인증과 `anthropic-version: 2026-10-21` 소비 계약을 대조했다. 실제 API 키나 유료 요청을 쓰지 않고 기존 AI 설정·SSE·도구 호출·오류·편집·Undo E2E를 적용한다.

## GitHub Actions 4건과 원 CI 실패

| 원 PR | 적용 Action | 공식 tag의 full SHA |
| --- | --- | --- |
| #8813 | softprops/action-gh-release v3.0.4 | `8a0cd651e5b2365bd640854a3fa5dfc70e4875f3` |
| #8818·#8820 | github/codeql-action v4.39.3 | `33d7b85da1a04171e211befe2c739303b9a4c066` |
| #8821 | taiki-e/install-action v2.90.2 | `3a3daafd3b7f23b8f65d9f5ba2db203fb59b79e2` |

공식 action.yml 전후에서 Node 24/composite runtime, 실제 호출 입력, 삭제된 입력·변경된 기본값을 대조했다. 입력 제거·기본값 변경은 없으며 이 PR은 job 권한·조건·명령·검사 범위를 바꾸지 않는다. 실제 릴리즈 게시는 로컬 검증으로 실행하지 않는다.

#8818은 analyze만, #8820은 init만 바뀌어 서로 다른 버전의 CodeQL 설정을 읽는다. 두 원 head의 네 언어 Analyze와 lint 실패는 실제 로그에서 버전 불일치로 확인했다. 기존 해결 #7851·#7854와 현재 Dependabot codeql-action 그룹도 확인했다. 그룹이 있는데도 두 PR로 나뉜 원인은 미확정이며 설정을 임의로 바꾸지 않았다. 두 업데이트를 함께 적용하고 기존 버전을 가리키던 주석 5곳을 정렬한 것이 이번 보정의 전부다. 보안 탐지·검사 기대값·언어 수는 유지한다.

#8814·#8817·#8819·#8820의 Native 취소는 GitHub annotation의 `maximum execution time of 30m0s`와 실제 테스트 성공·후속 컴파일 로그로 확인했다. assertion·컴파일 오류로 판정하지 않는다. 원 CI의 실패·취소는 지우지 않으며 임의로 재실행하지 않았다. 결합 source의 새 head CI가 별도로 필요하다.

원격 처리 승인을 받기 전까지 push·새 PR·공개 댓글·merge·close를 수행하지 않는다. workflow 변경을 포함하므로 merge 뒤 운영 기록은 옵션 2의 별도 기록 PR로 반영한다.
