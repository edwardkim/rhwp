// font_substitution.js
// Legacy editor mirror of rhwp-studio/src/core/font-substitution.ts.
// Keep the font substitution table and fallback chain aligned with the studio
// renderer so canvas2d/canvaskit/legacy canvas resolve families consistently.

(function() {
    'use strict';

    const REGISTERED_FONTS = new Set([
        '함초롬돋움',
        '함초롬바탕',
        '함초롱돋움',
        '함초롱바탕',
        '한컴돋움',
        '한컴바탕',
        '새돋움',
        '새바탕',
        'HY헤드라인M',
        'HYHeadLine M',
        'HYHeadLine Medium',
        'HY견고딕',
        'HYGothic-Extra',
        'HY그래픽',
        'HYGraphic-Medium',
        'HY그래픽M',
        'HY견명조',
        'HYMyeongJo-Extra',
        'HY신명조',
        'HY중고딕',
        '양재튼튼체B',
        'Malgun Gothic',
        '맑은 고딕',
        'Apple SD Gothic Neo',
        '돋움',
        '돋움체',
        '굴림',
        'GulimChe',
        '굴림체',
        '새굴림',
        'Batang',
        '바탕',
        '바탕체',
        'AppleMyungjo',
        '궁서',
        '궁서체',
        '새궁서',
        'NanumGothic',
        '나눔고딕',
        'NanumMyeongjo',
        '나눔명조',
        'NanumGothicCoding',
        '나눔고딕코딩',
        'Palatino Linotype',
        'Noto Sans CJK KR',
        'Noto Sans KR',
        'Noto Serif CJK KR',
        'Noto Serif KR',
        'Pretendard',
        'Pretendard Thin',
        'Pretendard ExtraLight',
        'Pretendard Light',
        'Pretendard Medium',
        'Pretendard SemiBold',
        'Pretendard Bold',
        'Pretendard ExtraBold',
        'Pretendard Black',
        'D2Coding',
        '해피니스 산스 레귤러',
        'Happiness Sans Regular',
        '해피니스 산스 볼드',
        'Happiness Sans Bold',
        '해피니스 산스 타이틀',
        'Happiness Sans Title',
        '해피니스 산스 VF',
        'Happiness Sans VF',
        'Cafe24 Ssurround Bold',
        '카페24 슈퍼매직',
        'Cafe24 Supermagic',
        'Latin Modern Math',
        'SpoqaHanSans',
        '고운바탕',
        '고운돋움',
    ]);

    const SUBST_TABLES = [
        [
            ['휴먼명조',2,'휴먼명조',1],['휴먼명조',1,'HY신명조',1],
            ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
            ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
            ['한양견명조',2,'HY견명조',1],['신명 태명조',2,'HY신명조',1],
            ['신명 견고딕',2,'HY견고딕',1],['신명 견명조',2,'HY견명조',1],
            ['신명 태그래픽',2,'HY그래픽',1],['신명 중고딕',2,'HY중고딕',1],
            ['태 가는 헤드라인T',2,'HY헤드라인M',1],['양재 튼튼B',2,'양재튼튼체B',1],
            ['태 가는 헤드라인D',2,'HY헤드라인M',1],['한양견고딕',2,'HY견고딕',1],
            ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
            ['Malgun Gothic',1,'맑은 고딕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],
            ['궁서',1,'새궁서',1],
            ['백묵 굴림',1,'굴림',1],['백묵 돋움',1,'돋움',1],
            ['백묵 바탕',1,'바탕',1],['백묵 헤드라인',1,'돋움',1],
            ['가는안상수체',1,'함초롬돋움',1],['중간안상수체',1,'함초롬돋움',1],
            ['굵은안상수체',1,'함초롬돋움',1],['HY그래픽M',1,'HY그래픽',1],
            ['명조',2,'바탕',1],['고딕',2,'돋움',1],
            ['샘물',2,'고딕',2],['필기',2,'명조',2],['시스템',2,'고딕',2],
            ['HY둥근고딕',2,'시스템',2],['옛한글',2,'명조',2],
            ['가는공한',2,'명조',2],['중간공한',2,'명조',2],['굵은공한',2,'명조',2],
            ['가는한',2,'샘물',2],['중간한',2,'샘물',2],['굵은한',2,'샘물',2],
            ['휴먼명조',2,'옛한글',2],['휴먼고딕',2,'고딕',2],
            ['가는안상수체',2,'가는한',2],['중간안상수체',2,'중간한',2],['굵은안상수체',2,'굵은한',2],
            ['휴먼가는샘체',2,'가는한',2],['휴먼중간샘체',2,'중간한',2],['휴먼굵은샘체',2,'굵은한',2],
            ['휴먼가는팸체',2,'휴먼가는샘체',2],['휴먼중간팸체',2,'휴먼중간샘체',2],['휴먼굵은팸체',2,'휴먼굵은샘체',2],
            ['휴먼옛체',2,'휴먼고딕',2],
            ['한양신명조',2,'휴먼명조',2],['한양견명조',2,'휴먼명조',2],
            ['한양중고딕',2,'휴먼고딕',2],['한양견고딕',2,'휴먼고딕',2],
            ['한양그래픽',2,'굴림',1],['한양궁서',2,'궁서',1],
            ['문화바탕',2,'휴먼명조',2],['문화바탕제목',2,'휴먼명조',2],
            ['문화돋움',2,'휴먼고딕',2],['문화돋움제목',2,'휴먼고딕',2],
            ['문화쓰기',2,'휴먼명조',2],['문화쓰기흘림',2,'휴먼명조',2],
            ['펜흘림',2,'휴먼명조',2],['복숭아',2,'휴먼중간팸체',2],
            ['옥수수',2,'휴먼옛체',2],['오이',2,'필기',2],['가지',2,'필기',2],
            ['강낭콩',2,'한양그래픽',2],['딸기',2,'휴먼옛체',2],['타이프',2,'굵은공한',2],
            ['태 나무',2,'휴먼고딕',2],
            ['태 헤드라인D',2,'신명 견명조',2],['태 가는 헤드라인D',2,'태 헤드라인D',2],
            ['태 헤드라인T',2,'신명 견고딕',2],['태 가는 헤드라인T',2,'태 헤드라인T',2],
            ['양재 다운명조M',2,'휴먼명조',2],['양재 본목각M',2,'옥수수',2],
            ['양재 소슬',2,'태 나무',2],['양재 튼튼B',2,'태 가는 헤드라인T',2],
            ['양재 참숯B',2,'한양견고딕',2],['양재 둘기',2,'가지',2],
            ['양재 매화',2,'옥수수',2],['양재 샤넬',2,'태 나무',2],
            ['양재 와당',2,'양재 참숯B',2],['양재 이니셜',2,'양재 참숯B',2],
            ['신명 세명조',2,'휴먼명조',2],['신명 신명조',2,'휴먼명조',2],
            ['신명 신신명조',2,'휴먼명조',2],['신명 중명조',2,'휴먼명조',2],
            ['신명 태명조',2,'휴먼명조',2],['신명 견명조',2,'휴먼명조',2],
            ['신명 신문명조',2,'휴먼명조',2],['신명 순명조',2,'휴먼명조',2],
            ['신명 세고딕',2,'휴먼고딕',2],['신명 중고딕',2,'휴먼고딕',2],
            ['신명 태고딕',2,'휴먼고딕',2],['신명 견고딕',2,'휴먼고딕',2],
            ['신명 세나루',2,'휴먼고딕',2],['신명 디나루',2,'휴먼고딕',2],
            ['신명 신그래픽',2,'한양그래픽',2],['신명 태그래픽',2,'한양그래픽',2],
            ['신명 궁서',2,'한양궁서',2],['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
            ['명조',2,'HY견명조',1],['HCI Poppy',2,'Palatino Linotype',1],
            ['신명 태고딕',2,'HY중고딕',1],['산세리프',2,'Calibri',1],
            ['한양견명조',2,'HY견명조',1],['신명 태명조',2,'HY신명조',1],
            ['신명 견고딕',2,'HY견고딕',1],['신명 견명조',2,'HY견명조',1],
            ['신명 태그래픽',2,'HY그래픽',1],['신명 중고딕',2,'HY중고딕',1],
            ['양재 튼튼B',2,'양재튼튼체B',1],['한양견고딕',2,'HY견고딕',1],
            ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
            ['Malgun Gothic',1,'맑은 고딕',1],
            ['Tahoma',1,'함초롬돋움',1],['MS Sans Serif',1,'함초롬돋움',1],
            ['Times New Roman',1,'함초롬바탕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['백묵 굴림',1,'굴림',1],['백묵 돋움',1,'돋움',1],
            ['백묵 바탕',1,'바탕',1],['백묵 헤드라인',1,'돋움',1],
            ['HY그래픽M',1,'HY그래픽',1],
            ['명조',2,'바탕',1],['고딕',2,'돋움',1],
            ['산세리프',2,'고딕',2],['필기',2,'명조',2],
            ['한양신명조',2,'명조',2],['한양중고딕',2,'고딕',2],
            ['시스템',2,'한양중고딕',2],['HY둥근고딕',2,'시스템',2],
            ['한양견명조',2,'한양신명조',2],['한양견고딕',2,'한양중고딕',2],
            ['한양그래픽',2,'굴림',1],['한양궁서',2,'궁서',1],
            ['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
            ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
            ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['명조',2,'바탕',1],['한양신명조',2,'명조',2],['한양중고딕',2,'돋움',1],
            ['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
            ['명조',2,'HY견명조',1],['신명 태고딕',2,'HY중고딕',1],
            ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['명조',2,'바탕',1],['고딕',2,'돋움',1],
            ['한양신명조',2,'명조',2],['한양중고딕',2,'고딕',2],
            ['시스템',2,'굴림',1],['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양신명조',2,'HY신명조',1],['명조',2,'HY견명조',1],
            ['Gulim',1,'굴림',1],['Malgun Gothic',1,'맑은 고딕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['명조',2,'바탕',1],['한양신명조',2,'명조',2],
            ['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양중고딕',2,'HY중고딕',1],['한양신명조',2,'HY신명조',1],
            ['명조',2,'HY견명조',1],['신명 견고딕',2,'HY견고딕',1],
            ['신명 견명조',2,'HY견명조',1],['신명 태그래픽',2,'HY그래픽',1],
            ['Gulim',1,'굴림',1],['HYHeadLine Medium',1,'HY헤드라인M',1],
            ['Malgun Gothic',1,'맑은 고딕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['명조',2,'바탕',1],['시스템',2,'명조',2],
            ['한양신명조',2,'명조',2],['한양중고딕',2,'한양신명조',2],
            ['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
        [
            ['한양신명조',2,'HY신명조',1],['명조',2,'HY견명조',1],
            ['Gulimche',1,'굴림체',1],['Gulim',1,'굴림',1],
            ['Malgun Gothic',1,'맑은 고딕',1],
            ['함초롬돋움',1,'함초롬바탕',1],
            ['한컴바탕',1,'함초롬바탕',1],['한컴돋움',1,'함초롬돋움',1],
            ['새바탕',1,'한컴바탕',1],['새돋움',1,'한컴돋움',1],
            ['바탕',1,'새바탕',1],['돋움',1,'새돋움',1],
            ['새굴림',1,'돋움',1],['굴림',1,'새굴림',1],
            ['새궁서',1,'바탕',1],['궁서',1,'새궁서',1],
            ['명조',2,'바탕',1],['한글 풀어쓰기',2,'명조',2],
            ['SPOQAHANSANS',1,'SpoqaHanSans',1],
        ],
    ];

    const substMaps = SUBST_TABLES.map(function(langTable) {
        const map = new Map();
        for (const entry of langTable) {
            const key = entry[0] + '\0' + entry[1];
            if (!map.has(key)) {
                map.set(key, { face: entry[2], type: entry[3] });
            }
        }
        return map;
    });

    const resolveCache = new Map();

    function resolveFont(fontName, altType, langId) {
        if (!fontName) return fontName;
        if (REGISTERED_FONTS.has(fontName)) return fontName;

        const cacheKey = langId + '\0' + fontName + '\0' + altType;
        if (resolveCache.has(cacheKey)) return resolveCache.get(cacheKey);

        const langIdx = langId >= 0 && langId <= 6 ? langId : 0;
        const substMap = substMaps[langIdx];

        let name = fontName;
        let type = altType || 0;

        if (type === 0) {
            if (substMap.has(name + '\x001')) {
                type = 1;
            } else if (substMap.has(name + '\x002')) {
                type = 2;
            } else {
                resolveCache.set(cacheKey, fontName);
                return fontName;
            }
        }

        const visited = new Set();
        for (let i = 0; i < 15; i++) {
            if (REGISTERED_FONTS.has(name)) break;

            const key = name + '\0' + type;
            if (visited.has(key)) break;
            visited.add(key);

            const subst = substMap.get(key);
            if (!subst) break;

            name = subst.face;
            type = subst.type;
        }

        resolveCache.set(cacheKey, name);
        return name;
    }

    function fontFamilyWithFallback(fontName) {
        if (fontName === 'serif' || fontName === 'sans-serif' || fontName === 'monospace') {
            return fontName;
        }
        if (/굴림체|바탕체|gulimche|batangche|coding|courier/i.test(fontName)) {
            return '"' + fontName + '", "GulimChe", "D2Coding", "NanumGothicCoding", "나눔고딕코딩", "Noto Sans Mono", monospace';
        }
        if (/[바탕명조궁서]|hymjre|times|palatino|georgia|batang|gungsuh/i.test(fontName)) {
            return '"' + fontName + '", "Batang", "AppleMyungjo", "Noto Serif KR", "Noto Serif CJK KR", "NanumMyeongjo", "나눔명조", serif';
        }
        return '"' + fontName + '", "Malgun Gothic", "Apple SD Gothic Neo", "Noto Sans KR", "Noto Sans CJK KR", "NanumGothic", "나눔고딕", "Pretendard", sans-serif';
    }

    globalThis.FontSubstitution = {
        resolveFont: resolveFont,
        fontFamilyWithFallback: fontFamilyWithFallback,
        REGISTERED_FONTS: REGISTERED_FONTS,
    };
})();
