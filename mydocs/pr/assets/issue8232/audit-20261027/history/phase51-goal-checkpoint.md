---
kind: investigation
status: historical
canonical: mydocs/pr/assets/issue8232/audit-20261027/history/phase51-goal-checkpoint.md
last_verified: 2026-10-27
---

> 이 문서는 해당 회차 당시의 조사 기록입니다. 이후 정정·리베이스·범위 제외가 있을 수 있습니다. 현재 집계와 완료 여부는 [현재 조사 보고서](<../../../../../working/task_m100_8232_stage1.md>)를 따릅니다.

> 원 실행 기록: `output/issue8232-audit/phase51-goal-checkpoint.md` · 원문 SHA-256: `297f8e0867099eb92d634b949c08e1e021ed611baa9695557db5f5c7ffe64715`. 로컬 전용 증적 경로는 원문 문자열로 남겼으며, 원문 전체는 `history-original-texts.jsonl`에 보존합니다.

# 51차 감사 체크포인트

목표 ACTIVE/미완료. 전수조사→#8232정리→부적합 검사/필요 구현보정→실제보정파일/함수만issue8232명명·helper/module/suite/generated/script/E2E소비갱신/옛wrapper중복금지. 정상검사의모든수치±2변환/일괄개명/삭제는하지않음. 독립근거의검증목적은보존하고잘못된기대값은교체. PR나중/새승인전push·PR없음. 에이전트·다른task메시지·예약·메모리수정없음. 메모리registry8232/native_viewport검색0hit로관련메모리사용없음.

R=/Users/tsjang/rhwp,branch audit/8232-px-regressions-20261025,HEAD238de6de41a208648f4ade6b494fe433a81c1fce. 시작/종료gitstatus clean. 제품/검사보정0/개명0.
원본/PDF/sharedtarget보존. 직전50턴은99신규직접판정/67PASS/원장과이슈업데이트까지실제진척이었다. 옛각단계checkpoint/receipt는당시snapshot이고현행globalcount는51이다.

이번3파일모든41함수직접읽음: native_viewport12함수/8tests,native_fonts13/8,hyperlink_pdf16/9. 첫combined도구출력에서fonts중간이잘렸기때문에font파일3423tokens전체를별도로다시읽은뒤기록했다. 신규direct41(25tests16helpers),숫자후보밖20원장연결/기존미분류21해소. integrationbatch63/helper55. 새px부적합원인0. 실물잠정배치핀과authored합성기하/현재호출동등/상태·bytes·자원계약을구분.

run_phase51_existing_tests.py exec79283 terminal0. native-skia활성현재3모듈24실등록/24PASS0FAIL/summary4.036s/834필터제외,wrapper5.1535s. Nativefonts의비활성1검사는여기에없으므로run_phase51_no_skia.py exec91668로별도기본feature구성실행:1등록1PASS0FAIL/0.015s/279필터제외,wrapper1.6764s,list/build150.3388s(실Cargo2m27s). 스크립트둘다terminal/exit확인,중복실행안함. Cargo순차/jobs8/testthreads8,같은공유target사용. defaultbuild가rootCLI를다른feature로덮을수있으므로51nativeproducer는불필요한rootCLI실행파일을고정하지않고고유suite deps바이너리hash만고정했다. 현재두feature의서로다른고유binary/source전후hash같음. 24+1은고유25테스트이며default구성PDF8개까지실행했다고확대하지않음.

viewport: 합성full-page와region의장치격자crop을exactRGBA로대조한다(AA/clip/group/cache/effects/negative/offpage/padding). 원점1,2/크기10,8은입력1.3,2.2,7.4,5.1×1.25의floor/ceil산술.32×24×4는byte수,±2^24는정확격자범위,16384/16Mpixel/128MiB는자원예산. current renderer.rs663–672의SkiaRegionPlan::new?이render_pixels_in_window앞이라는소비경로를읽었다. 오류assert는allocator실측아니며working+output예산은전체RSS아님. 이를실물PDF98%자격이필요한잠정좌표핀이나±2로완화할검사로오인하지않음. 두렌더링이다같이잘못그리는경우까지잡는Hancomfidelityoracle도아님.

fonts: MIT독창합성fixture Regular/Italic/TTC3faces,A/가/😀의정의·cmap·hmtx를FontTools로독립대조. generator_source전체와LICENSE읽음(생성기실행으로원본변경안함). 글리프 source는직사각형/평행사변형테스트이며실제한국어대체font정합아님. blake3/byte_length/face_count,source순서/환경선언/0–3font_generation/revision0,실패원자성/파일삭제뒤bytes소유/스냅샷현재bitmap동등/TTCoffset16–20두번째face손상/파일·메모리한계를검사. native비활성source는파일읽기블록밖에서즉시FeatureDisabled. 제한테스트64face/1MiB초과는retainedcopyguard순서이며64정상faces전체지원주장아님. 같은bitmap정합은현재동작불변이지역사적실물SVGgolden이아님.

PDF: authoredWi😀X/scalar경계0,24,28,51,65와clip교차/논리링크조각·URI허용scheme·선택/실패인덱스계약. coordinate7.5,52.5,30,60pt는100²CSSpage/10,20,30,10CSSrect의72/96·Y반전독립산술. PDFium별도CAPI로실생산8PDF/11쪽의URI/Rect/페이지귀속확인; skipped는first/third,멀티재배열3쪽counts33/41/33이며1/3annotation동일,DirectSkiaURI유지. 현재writer문자열testhelperpage_uris는일반압축PDF/문자열escape전체파서아님을분리. 실제source Textmail/LHfragment는URI존재만assert하며임시pagebbox핀없음. 원task6963stage3기록도writer좌표정합과독립한컴배치차이6.5/29.2pt/수동ink검증을구분함. 현재같은SVG의annotation유무두PDF쌍을PDFium96dpi재래스터해픽셀차이0. 기존Rusttest는beforePDF를산출할뿐ink동등assert는없으므로추가독립대조를기존test의잉크검출력으로쓰지않음. 이생산PDF11쪽은전체실물VisualSweep/90·98%수용이아님. 새canonicalcapture0/catalog179조건2719쪽/backend유지(기존미달포함목록).

독립sourceURI: Textmail원시CFB/BodyTextSection0의tag67표시문자와tag71CTRL_HEADER(1357offset,raw6b6c6825)에escapedhttp\://www.hancom.co.kr명령을확인. 초기도우미는encode('utf16le')오타로LookupError→utf-16-le정정,이어unescapedfullURL이있다고가정해assert실패→원시Command가escaped인것을실제스캔해정정. 두초기실패로그보존/제품실패로세지않음. HWPX초기host전체16일치진단은별도broad원문보존후정확index.htmlCommand/Path2개(HYPERLINKfieldBegin id1733147618)에좁혀소유확인. 독립한컴PDFTextmail1쪽/LH29쪽중8쪽의URL텍스트·annotation1개/sourcehash도대조. 원본/PDF/제품/기준변경없음. 생성기JSON주석에잘못섞인비한국어문구2개는메타데이터만한국어정정했고같은독립관측값유지.

기원: Native도입823137/공통viewport보정5ccc/fonts a119는실7969merge04f8c2bd,PDF6b8e890은실6984merge11860a9. GitHubcommitassociation에nativecommits미통합PR23건도따라붙지만실merged1개만사용. 현재3source는actualmerge내용과byte동일/조상확인. docs536integrationstage5/6963stage3의원목적·경로차이읽음. 닫힌상태/커밋제목만으로merge판정안함. 원문 metadata/association/sourcehashphase51-origin-chain/pr-metadata파일보존.

현재direct7997/818files,ledger11877/2138,adjud7506/pending491/unreviewed3555/semantic113/filecontext212,분류미확정3880. 모든행fullconformancefalse/callclosurepending유지. validate_phase51.py실제PASS/producer/고유binary/원본URI/PDF/font/산술/원장연결. 원장분류는제품/테스트수정이아님.

#8232 updated2026-10-26T05:43:57Z,body64950Pythonchars(JS추가5). 직전50본문fresh byte동일확인후,자기가쓴50차869chars만원문phase51-prior-50-unabridged.md/이전fullbody/receipt보존후586chars요약·byte복구검증. top원장11877/미분류3555/미확정3880만갱신+51차추가,외부/사용자내용변경0. 게시직전fresh본문비교/게시후EOFAPIbyte동일. body한도65536임박하므로다음차수추가전자기옛상세만원문보존/복구검증후요약하고외부내용은유지. 현재목표·기준·보정범위축소금지.

다음: 남은미분류3555+메모325/독립pending491의sourcecfg·tests/helper·동적corpus/generator/script/E2E독립근거와소비경로를계속판정. 7902completed11pending/originalframecontent3/sourcepending38(HeightCursor12포함)/다른실물독립PDF계약도아직남음. 모든실제부적합보정/새명명/호출등록갱신/필수재검증/후속PR미완료. 완료한24+1테스트/독립PDF·폰트대조는이유없이반복실행하지않음. ACTIVE유지.
