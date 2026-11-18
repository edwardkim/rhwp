# 원본 HFT 글머리표 전진과 명시 공급자의 Native 소유

화학 원본 72쪽의 다섯 `•`는 SYMBOL 슬롯의 한양중고딕 HFT다. 독립 한컴 2020 Hancom PDF Print의 T11은 U+2022를 1000/1000em으로 기록한다. 기존 출력은 표시용 H2GTRM의 cmap 결측에서 450HU 반각을 추정했다. 원본의 9pt(900HU) 한 칸을 잃어 뒤 본문이 당겨졌다. 원본 Print와 입력은 변경하지 않는다.

기존 원본 글리프 overlay는 한양신명조·명조 기호의 전진과 실제 TTF 폭을 구분한다. #9025의 `31a4225757`은 명조 U+2022의 원본 전각을 보존했고, 현재 브랜치의 `058acd32a1`도 한양신명조 U+257A의 같은 cmap 결측 경계를 보정했다. 한양중고딕 U+2022의 일반 HFT만 같은 독립 근거로 보완하며 동명 TTF·미검증 bold/italic에 옮기지 않는다.

표시 프로그램도 독립적으로 확인했다. 일반 fallback HCRDotum의 U+2022는 원본과 윤곽·전진이 다르다. 기존 로컬 CompleteSymbols의 U+2022는 PDF의 원본 pen 명령과 수치가 정확히 같고 hmtx도 1000/1000em이다. 비공개 프로그램을 새로 만들거나 Git에 넣지 않으며 읽기 전용 공급자로 검증한다. 원본 font container의 전체 파일 동일성과 특정 글리프의 동일성은 구분한다.

검증한 공급자를 glyphFallbacks에 선언하면 준비된 메트릭은 900HU가 되지만 Native paint는 여전히 HCRDotum을 선택했다. Canvas는 #9025에서 명시 glyph 공급자를 OS 별칭보다 먼저 소비하도록 보정됐으나 Native는 `installed_render_font_aliases`가 끼워 넣은 일반 대체를 먼저 사용했다. 실제 style의 `glyph_fallback_advance_explicit`을 Native paint와 관측이 함께 소비하도록 연결한다. 일반 환경의 기존 별칭 순서와 주 face의 실제 glyph hit는 보존한다. 문서·쪽 번호 분기는 추가하지 않는다.

새 관계형 검사는 tests/cases에 두고 MIT 합성 프로그램만 메모리에 적재한다. 수정 전 최종 실행은 3건 중 1 PASS·2 FAIL(0.213초)이며 일반 환경 대조군은 통과했다. 초기 helper 수집 누락·합성 프로그램의 표시 이름 착오는 검사 준비 오류로 별도 보존하며 제품 결함의 실패 증거로 사용하지 않는다. 수정 후 새 검사 3건 모두 PASS(0.347초)다. 관련 Native/Canvas/PUA/시스템 글꼴 소유 검사 47건도 PASS(1.180초)이고 suite 수집·Cargo 격리 계약 26건이 통과했다. 모두 최소 글꼴 sandbox에서 회귀 검사를 실행했으며 개인 글꼴이 없어도 수행된다. 실물 Native/fresh WASM·전체 회귀·보호 검증은 진행 중이다.

## 실제 집중 출력

코드 `120483b82f`의 Native 72쪽은 99.91401%(미대응76), fresh WASM은 99.95110%(44)이다. 같은 원본 기호 환경의 수정 전 Native는99.86650%(118)이고 수정 전 WASM은99.95110%(44)다. 예전 자동 환경99.38114%/99.43167%와 공급 조건이 다르므로 그 차이 전부를 코드 개선으로 주장하지 않는다. 다섯 글머리표는 원본900HU 전진과 검증된 원본 glyph를 소비함을 실제 Native 결정으로 확인했다. 두 경로 같은 쪽 review·overlay를 직접 판독했다. 남은 본문·따옴표 차이를 그림 차이로 부르거나100%로 반올림하지 않는다.

Mac fresh WASM은 locked wrapper --no-opt 로컬 대체 빌드이고 루트 pkg·Studio public의 JS/WASM SHA가 각각 일치한다. 이 집중 결과는 새 소스의 전쪽·전체 회귀·정본 보호·실제 Studio 소비 검증 완료가 아니다.
