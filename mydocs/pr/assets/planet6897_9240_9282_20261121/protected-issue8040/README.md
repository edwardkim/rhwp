# #8040 보호 문서의 신그래픽체 공급 검증

삽입본과 삽입·삭제본에서 한컴 2020 Print, 96 DPI의 2px 실루엣을 대조했다. Native와 fresh WASM
각각 두 파일의 전 4쪽을 출력했다. 네 실행 모두 **1~3쪽 100.00%, 4쪽 99.99%**로 보호 TSV의 쪽별
기준값을 충족했다. 측정에 사용한 원본, Print와 렌더러 해시는 [검증 영수증](validation.json)에 있다.

삽입본의 HWP `DocInfo`에서 큰 제목의 한글·영문 글꼴 레코드가 모두 `신명 신그래픽`임을 확인했다.
제공받은 `신그래픽체.ttf`의 실제 family·PostScript 이름은 `SinGraphic`이었다. 해당 글꼴 레코드에만
연결하고 기존 글자 폭과 다른 글꼴의 자동 선택을 유지했다. 측정에서는 파일을 명시적으로 공급했고,
측정 뒤 같은 바이트를 검증 호스트의 사용자 글꼴로 등록했다. 환경 선언은 글꼴 신원 해시를 사용한다.

- Native [글꼴 환경](native-font-environment.json) · [삽입](issue8040-insert-native.tsv) · [삽입·삭제](issue8040-insert-delete-native.tsv)
- fresh WASM [글꼴 환경](wasm-font-environment.json) · [삽입](issue8040-insert-wasm.tsv) · [삽입·삭제](issue8040-insert-delete-wasm.tsv)

최종 제품 후보와 고정 `devel`에 동일한 신그래픽체 파일과 검증 설정을 적용해, 삽입본·삽입 삭제본의 Native와 fresh WASM **4조건·16쪽**을 대조했다. 각 후보 TSV는 고정 기준 TSV 및 위에 연결한 기존 TSV와 바이트 단위로 일치한다. [조건·생산물·TSV 해시 대조](paired-v5.json)
