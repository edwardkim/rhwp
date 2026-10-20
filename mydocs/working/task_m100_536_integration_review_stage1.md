# seo-rii 20개 PR 체리픽 통합 1단계

## 적용 범위와 신원

기준 devel `fe5725dc384689b45d60ef8ea4c83138bc2701d8`, 최종 원 head `b4003f89fceb8df6e64652dfdf9ec1cfcd642cf1`.
비 draft 20개를 기존 author와 원 SHA를 보존해 순차 적용했다. draft Qt #7930의 5개 commit과 #7932는 제외했다.
총 고유 165개 비 merge commit 중 Qt 5개를 제외한 160개를 처리했다. 결과: {'applied': 153, 'empty-preserved-by-current-tree': 7}.
원 PR별 review/implementation 접수 기록과 전체 계획을 같은 branch에 두었다. 검토·검증은 아직 진행 중이다.

## 원 merge 해소 결과 대조

최신 devel + 최종 source의 자동 병합은 충돌 없이 tree `223917d97e375522df9f12e880fdb1fd57e09ca6`를 만들었다.
merge commit을 직접 체리픽하지 않는 규칙에 따라, 비 merge 적용 결과와 이 tree 사이 17파일·1,137줄 diff를 직접 읽었다.
차이는 다음 통합 해소의 누락 및 이 세션 충돌 보정의 표현 차이였다. 최종 source delta와 최신 devel 보존 결과로 정렬했다.

- glyph SVG의 실제 fontEmBox 목적지 수용 조건과 Rust/TS 반례를 함께 복구했다.
- Native font catalog의 SFNT family alias·CSS3 style 순서·첫 공급 경로 소유권·명시적 alias API를 유지했다. HFT 소비자의 matching helper도 같은 catalog를 쓴다.
- Native shape shader와 direct PDF는 기존 공통 source-center·axis·expanded stop 계약을 소비한다. 독립 f32 영역/stop 상한과 관련 기존 회귀를 보존했다.
- 정확한 기존 Skia glyph 검사에는 새 필수 fontSize/fontEmBox 필드와 0 크기 fallback 반례를 반영했다. 기존 검사 전체 삭제를 수용하지 않았다.
- Native edit의 실제 replacement span guard를 유지하고, 잘못 중복한 현재 selection 선행 검사는 제거해 명시적 대상 selection 및 동일 내용 no-op 계약을 보존했다.
- e2e 각주 기준선 동작 보호와 #7688 폰트 검사를 유지했다. scripts/선언의 단순 위치 차이는 source 최종 tree에 정렬했다.

조판 결과나 테스트를 아직 통과했다고 주장하지 않는다. 다음 단계는 코드 소비 경로 검토·작은 집중 검사·후보 출력과 독립 Print 비교다.

## 기준 출력 준비

- Native Skia 기준 빌드: 386.895초, exit 0.
- fresh WASM 기준 빌드: host no-opt 396.304초, exit 0. Docker 실행 파일이 없어 지침의 대체 진단 경로를 썼으며 최적화 배포 빌드 검증으로 표시하지 않는다.
- 공급 폰트는 `/opt/hnc/hoffice11/Shared/TTF/{Install,Hwp,All}`의 실제 디렉터리이며 파일 hash는 실행 manifest에 보존한다.
- lseg-01-basic 1쪽 기준 Native 99.74540%, WASM 99.93175%; lseg-02-mixed 1쪽 기준 Native 99.80580%, WASM 99.84066%.
- 추가 기준 Native: pic-crop-01 99.98500%, shape-group-02 90.04403%, form-01 0%, equation-lim 88.42676%. 미달 두 문서는 실제 출력과 기준 대응을 판독한 뒤 후보에서 재검증·필요 보정한다.
- form-01 Print PDF는 빈 쪽이며 Native에는 폼 개체가 그려지는 차이를 확인했다. 숨김 처리로 덮지 않고 원본 인쇄 계약·옵션과 producer/consumer 경로를 조사한다.
- 모든 입력/PDF는 기존 Git 파일이다. 최종 수용 시 실행 파일/commit byte 및 provenance를 개별 review에 연결한다.

## 로그와 다음 gate

현재 로그/TSV/비교 PNG는 ignored `/home/tsjang/rhwp/output/pr-review/seo-rii-20261017/`에 보존한다.
종료 시 필요한 증거를 output 밖으로 해시 대조 보존하고 루트 output 전체를 정리한다. 사용자 Studio JS와 공용 target은 보존한다.
[전체 실행 계획](../plans/pr_review_seo_rii_20261017.md)을 따른다. PR 생성·머지·원 PR close는 최종 필수 gate 이후다.
