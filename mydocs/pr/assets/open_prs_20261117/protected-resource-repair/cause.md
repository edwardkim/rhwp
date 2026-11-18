# sample16의 WASM 메타파일 글꼴 공급 복원

초기 좁은 WASM 공급 묶음의 HWPX 18·35쪽은 77.59066%·82.46843%였다.
같은 원문·독립 PDF·DPI·compat·2px raw에서 고정 devel과 후보가 같은 미달을 보였다.
software Chrome 전환과 Arial 공급만으로는 복원되지 않았다. 이 실패는 output에 보존한다.

원본 HWPX 내부 WMF의 META_CREATEFONTINDIRECT(LOGFONT16)를 직접 읽어 `System`·`굴림체`
요청을 확인했다. 재구성한 묶음에는 원 수용 기록의 굴림 TTC가 빠져 있었다. Native는 Mac 설치
글꼴을 자동으로 읽지만 WASM은 명시적으로 전달한 프로그램 목록을 쓰므로 두 공급 경로를 구분한다.

Mac에 이미 설치된 굴림 컬렉션을 같은 family의 원본 자원으로 추가 공급하자 고정 base와 후보의
18쪽은 99.99065%, 35쪽은 100.00000%로 복원됐다. glyph coverage가 확보돼 공통 WMF 인쇄
재생 경로를 사용할 수 있게 된 관계는 `src/paint/metafile_print.rs`의 coverage 전제와 함께 확인했다.
글꼴을 설치·변경하거나 재배포하지 않았으며 소스 코드·원문·PDF·비교 반경·점수 기준도 변경하지 않았다.

공급 파일의 실제 SHA는 [집중 검증](focused-validation.json)에 기록했다. 원 수용의 Windows TTC와
동일 파일이라고 주장하지 않는다. 같은 요청 family를 복원하고 독립 Print와 실제 출력을 대조했다.
[18쪽 review](review-018.png)·[overlay](overlay-018.png),
[35쪽 review](review-035.png)·[overlay](overlay-035.png)를 직접 판독했다.
이 집중 최고값을 이전 전쪽 TSV에 섞지 않으며 base·후보의 세 형식 전 64쪽을 새 조건으로 다시 산출한다.
