# #7642 자동 줄 캐시의 문자 경계와 명시적 글꼴 요청 구분

Issue: #7642.

## 확인된 전제 오류

`b86d75469824f30ab796abfca58fb6dfccf3a18b`의 Native에서 온새미로 HWPX는
전쪽 90% 이상이다. HWP는 6쪽이 95.72581%로 개선됐으나 15·36쪽은 남았다.
같은 1.216 문단은 HWP/HWPX 모두 52자·같은 본문 폭·번호·글자 크기와
장평이지만 저장 마지막 줄의 시작은 48/50으로 다르다. 독립 Print는 같은
질문을 한 줄로 출력한다. 마지막 어절의 중간에서 끊겼는지를 캐시 유효성의
대용으로 삼으면 완전한 어절로 넘어간 HWP만 같은 재검토에서 빠진다.

또한 명시적 PostScript 식별자 지원 뒤 일반 fallback family도 시스템의
전체 PostScript 색인을 초기화할 수 있다. 원본이 명시한 프로그램과 서체
분류로 생성한 fallback family는 같은 요청 종류가 아니다. 없는 이름을 OS에
직접 조회하는 macOS 위험을 되살리는 방법은 쓰지 않는다.

## 구현 범위와 독립 근거

- 렌더 전용 본문 정규화는 입증한 동일 물리 폭·연속 프레임에서 명시 개행이
  없는 자동 줄을 실제 frame으로 재검토한다. 마지막 어절 조각 여부로 원본
  두 형식을 구분하지 않는다. 빈 문단·마지막 빈 예약 행은 이 재결합 대상으로
  삼지 않고 기존 점유를 보존한다. 줄·공백·후속 원점을 함께 발행한다.
- 지역 셀/그림 경로는 기존 어절 꼬리 계약을 유지한다. 이 경로는 본문 전체
  후속 축의 소유자가 아니므로 본문 정규화의 새 적용 범위를 복제하지 않는다.
- 명시적 원본 family와 그 설치 별칭을 생성 fallback과 구별해 전달한다.
  알려진 family의 정상 style 조회는 그대로며, 별도 PostScript 색인은
  명시적 식별자에만 사용한다. 원본이 직접 같은 이름을 요청하면 여전히 찾는다.
- [OpenType 이름 표](https://learn.microsoft.com/en-us/typography/opentype/spec/name#name-ids)의
  이름 ID 6 문자 제약에 맞지 않는 이름은 PostScript 색인을 열지 않는다.
  일반 family 이름 수용에는 이 제약을 적용하지 않는다. 길이의 과거 63자 제한은
  새로운 권고와 다를 수 있으므로 이 변경에서 길이 상한을 추가하지 않는다.

## 소비 경로와 검증

본문은 `project_stored_soft_wraps → render_normalization → 파생 paragraphs/composed →
pagination와 paint`를 공유한다. 글꼴은 선언 체인과 서체 fallback을 모두 유지하고
같은 `typeface_candidates_for_families → 문자별 선택 → 실제 paint/관측`을 사용한다.
명시 ID 6, family/full-face 구분, 같은 family의 문자 fallback, 설치 별칭,
수식과 폼 caption의 경로를 확인한다. spec 기반 이름 거절의 작은 계약 검사는
실물 한컴 배치·픽셀 회귀와 구분한다.

현재 빌드·검증 source는 고정한 worktree에서 계속 실행한다. 두 보정을 모은
source에서 영향을 받는 경계·전체 102개 문서 및 #7969·#7970의 양쪽 경로,
필수 Rust·lint·UI를 검증한다. 원본·기준 PDF·점수 임계값을 바꾸지 않는다.
전쪽 90%·확대 판독 전에 새 실물 회귀·baseline을 추가하지 않는다.
