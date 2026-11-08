# Mac 최소 글꼴 환경의 전체 회귀·Native 검사

제품 code head `def44c54c6370c0cd463bb859f6b810e2ee4f0bb`의 실제 실행 결과다.
개인 글꼴을 삭제·비활성화하지 않고 프로세스 한정 sandbox를 적용했다.

- release-test 전체 nextest: **13,072 PASS / 0 FAIL / 45 skip**, 실행346.703초.
- Native Skia root lib: **3,852 PASS / 0 FAIL / 12 ignored**, 다른 library182 PASS.
- Native PNG2 PASS / PDF35 PASS. 정본의 필수3종 범위를 순차 실행했다.
- Native 글꼴·프로그램·소수 원점·수식 소유 집중6모듈: **37 PASS / 0 FAIL**.
  1,931건은 선택 밖이며 전체 Native integration 재실행으로 확대하지 않는다.

CoreText에서528 faces를 열거했고 전부 OS 경로였다. system_profiler에서도237개 OS 경로만
확인했다. 고유 OS 프로그램208개의 파일 hash를 기록했다. AppleSDGothicNeo-Regular로
한글·영문·숫자6글리프의 실제 비어 있지 않은 윤곽을 확인해 LastResort 대체를 제외했다.
일반 프로세스의1,875 faces 목록은 전후 동일하다. 개인·공유 추가 글꼴 파일과 Font Registry의
접근만 검사 프로세스에서 차단했으며, 라이선스 확인 fixture·OS 기본 글꼴은 보존했다.

[명령·원래 producer·요약·로그 hash](validation.json), [실제 격리 프로필](full-minimal-font.sb),
[CoreText 목록](full-minimal-coretext-qualified.json), [기본 한글 윤곽 확인](full-minimal-korean-glyphs.json),
[OS 프로그램 hash·system_profiler 확인](full-minimal-os-font-files.json)을 연결한다.
로그와 시스템 글꼴 binary는 Git에 넣지 않는다. 이 결과는 다른 OS의 최신 head 실행이나
보호 TSV 전쪽·전체 lint/build·실제 소비 경로·PR 준비 완료를 의미하지 않는다.
