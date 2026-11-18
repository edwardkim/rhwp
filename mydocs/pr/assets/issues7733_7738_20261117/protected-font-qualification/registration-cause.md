# 보호 대조군의 원본 글꼴 공급 조건 복구

현재 Mac에 없는 과거 보호 생산자의 전체 파일을 동일한 파일로 간주하지 않는다. 독립 한컴 Print PDF의 사용 글리프 윤곽·hmtx·글리프 명령·전역 힌트 프로그램을 별도로 확인하고, 현재 고정 base와 후보를 같은 조건으로 다시 비교한다. 정본 보호값은 변경하지 않았다.

SEO 스타일 대조의 PDF 내장 Arial 부분 TTF에는 `name`·`OS/2`·`post` 등록 테이블이 없었다. Unicode cmap만 복원한 파일은 완전한 시스템 글꼴이 아니므로 로더가 사용하지 못했고 기울임체에 다른 face가 공급되었다. PDF와 실제 사용 글리프가 동일한 정식 Arial의 같은 스타일에서 등록용 이름·스타일 테이블만 보충했다. 실제 PDF 글꼴의 윤곽·hmtx·글리프 명령·`fpgm`·`prep`·`cvt `와 기존 테이블은 바꾸지 않고 검증했다. `head`는 SFNT 체크섬만 재계산했다.

이 글꼴은 검증 전용 로컬 풀에만 있으며 시스템 설치나 재배포를 하지 않았다. 기존 전체 파일 해시가 같다는 주장도 하지 않는다. 일반 사용자의 웹폰트 fallback이나 제품의 기본 글꼴 우선순위는 변경하지 않았다.

등록 보완 전 Native는 base/후보 각각 99.26%였고, 보완 뒤 두 조건 모두 100.00%다. 독립 원본 프로그램을 선택한 결과이며, 문턱·PDF·보호 TSV를 바꾸어 통과시킨 것이 아니다. 다른 보호 문서 전체의 같은 조건 재검증은 진행 중이다.

[이전 review](before-native-review-001.png) · [이전 overlay](before-native-overlay-001.png) · [보완 review](after-native-review-001.png) · [보완 overlay](after-native-overlay-001.png)
