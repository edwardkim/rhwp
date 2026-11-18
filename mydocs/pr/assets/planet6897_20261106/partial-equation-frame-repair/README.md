# 부분 수식 재조판의 점유와 후속 줄 소유 복구

원 해결 #7640과 실제 통합 #8754의 독립 한컴2020 Print·고정 devel을 대조했다.
`section0/p318/endnote0/p10`의 저장 앞줄은 현재 수식이 점유하는 높이와 기준선을 소비하지 않았고,
다시 채운 뒤 줄은 첫 줄로 처리돼 내어쓰기를 잃었다. paint에서도 이미 줄별로 소속을 확정한
제어문 목록을 새 문단으로 다시 채웠다. 이 세 지점이 17쪽 이후 내용 이동의 원인이다.

- 저장 앞줄은 현재 인라인 개체의 높이·기준선과 줄 간격 참여 속성을 함께 소비한다.
- 부분 채움의 시작 문자가 문단 처음이 아니면 후속 줄의 들여쓰기 플래그를 게시한다.
- 현재 줄 소유자가 확정된 수식은 같은 글줄 원점·폭을 paint까지 사용한다.
- 문서/쪽별 예외·임의 오프셋·판정 완화 없이 공통 producer와 소비자를 수정했다.

| 독립 입력 | Native 전21쪽 최저 | fresh WASM 전21쪽 최저 | 17쪽 Native / WASM |
| --- | ---: | ---: | ---: |
| 원본 | 98.78583% | 98.77947% | 99.15706% / 99.22150% |
| 재저장본 | 99.03786% | 99.02612% | 99.15980% / 99.22122% |

두 입력·두 backend 모두 전21쪽 번호를 확인했고 90% 미달·누락·중복은 없다.
Native 원본 12·17·18·19쪽, 재저장본 17·18·19쪽의 큰 하락은 고정 devel 표시값으로 복원됐다.
재저장본 2쪽은 base99.68% 대비99.66%로 남으며 이 보정 전후 값은 같다.
이 단계의 검증을 전체 통합 PR 완료로 확대하지 않는다.

새 관계형 검사는 네 수식의 소유·점유·내어쓰기를 보호한다. 고정형 검사는 두 경로의 17쪽
raw98% 이상과 독립 Print를 확인한 뒤 수식 시작점·본체 기준선을 ±2 CSSpx로 보호한다.
원본 수식의 정수 퍼센트 기준선 해상도와 실제 glyph 기준선은 구분한다.
수정 전 새 두 검사가 의도한 원점 오류로 실패했고, 수정 후 standalone 두 검사 모두 통과했다.
기존7건과 새2건을 자동 수집한 정식 nextest 모듈은9 PASS/0 FAIL이다(309건은 선택 밖).
같은 컴파일된 suite 바이너리를 개인 글꼴 파일과 Font Registry 접근을 막은 프로세스에서
실행해도9 PASS/0 FAIL이며,
이를 전체 CoreText 격리 환경 검증으로 주장하지 않는다. 기존 자동 수집 모듈에 추가하며 CI 호출은 추가하지 않는다.

fresh WASM은 Mac의 locked wrapper `--target web --out-dir pkg --no-opt` 로컬 대체 빌드다.
루트 pkg·Studio public·검증용 frozen package의 JS/WASM SHA가 각각 같다.
기존 실제 Canvas 묶음은17 PASS/0 FAIL이다. 전체 Studio 소비 검증의 대체 증거는 아니다.

[입력·producer·페이지별 결과·로그 hash](validation.json) ·
[Native 수정 전 review](before-native-p017-review.png) · [overlay](before-native-p017-overlay.png) ·
[Native 수정 후 review](after-native-p017-review.png) · [overlay](after-native-p017-overlay.png) ·
[WASM 수정 후 review](after-wasm-p017-review.png) · [overlay](after-wasm-p017-overlay.png).

각 `*-21-silhouette.tsv`는 같은 이름의 입력·backend·단계 전21쪽 실제 결과다.
고정 base producer의 실제 source는4aa8이며0618과 제품 소스가 같은 배포 문서 전용 차임을 별도로 확인했다.
정확한 producer·빌드·입력 SHA와 정밀값은 validation.json을 따른다.
