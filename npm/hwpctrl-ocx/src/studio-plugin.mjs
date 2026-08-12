/**
 * rhwp-studio 플러그인 — HwpCtrl API 로 **studio 가 들고 있는 그 문서**를 조작한다.
 *
 * 이 파일은 `rhwp-studio` 를 import 하지 않는다. `host` 의 모양만 알고, 그래서 패키지는
 * studio 없이도 단독으로 배포·테스트된다(standalone 모드는 `index.mjs` 그대로다).
 *
 * 세 가지 계약이 여기서 지켜진다.
 *  1. **문서는 한 벌** — `host.borrowDocument()` 로 빌린 핸들을 그대로 쓴다. 복사도 재파싱도 없다.
 *  2. **뮤테이션은 트랜잭션 경유** — 배치 1건 = undo 1스텝 = 재조판 1회.
 *  3. **교체는 위임** — `Open`/`Clear` 는 host 를 태운다. 아니면 두 문서가 조용히 갈라진다.
 *
 * 계획: `mydocs/plans/rhwp_studio_hwpctrl_plugin_impl.md` §5
 */
import { createHwpCtrl } from './index.mjs';
import { createAdoptDocument, isMutating } from './adapter.mjs';
import { listToStudio, studioToList, listDepth } from './cursor-map.mjs';

const PLUGIN_ID = 'hwpctrl';

/** 배치 안에 하나라도 문서를 바꾸는 호출이 있으면 트랜잭션으로 감싼다. */
function batchNeedsTransaction(ops) {
  return ops.some((op) => isMutating(op && op.m));
}

function callOn(ctrl, method, args) {
  const fn = ctrl[method];
  if (typeof fn !== 'function') {
    const error = new Error(`HwpCtrl 에 없는 메서드입니다: ${method}`);
    error.code = 'UNKNOWN_METHOD';
    throw error;
  }
  return fn.apply(ctrl, args || []);
}

export const hwpctrlStudioPlugin = {
  id: PLUGIN_ID,
  apiVersion: 1,

  activate(host) {
    const lease = host.borrowDocument();

    const ctrl = createHwpCtrl({
      // wasm 네임스페이스는 주지 않는다 — 문서를 따로 만들 수 있는 길을 열지 않기 위해서다.
      // 이 모드에서 문서 생성은 전부 `adoptDocument` 위임으로만 일어난다.
      wasm: null,
      doc: lease ? lease.handle : null,
      adoptDocument: createAdoptDocument(host),
      onSave: (bytes, fileName) => ({ bytes, fileName }),
    });

    // studio 가 문서를 갈아끼우면 빌린 핸들을 다시 받는다. 안 받으면 이 층이 옛 문서를 만진다.
    host.onDocumentSwap((next) => { ctrl.setDocument?.(next.handle); });

    /** 한 번의 호출. 읽기는 히스토리를 건드리지 않는다. */
    const invoke = (method, args) => {
      // 되돌리기는 studio 히스토리가 단일 진실이다 — 이 층의 자체 스냅샷을 쓰면 사용자가 보는
      // undo 스택과 두 갈래가 된다.
      if (method === 'Undo') return host.automation.execute('edit:undo');
      if (method === 'Redo') return host.automation.execute('edit:redo');
      if (!isMutating(method)) return host.read(() => callOn(ctrl, method, args));
      return host.transaction(`hwpctrl:${method}`, (tx) => {
        tx.deferPagination();
        return callOn(ctrl, method, args);
      });
    };

    /**
     * 여러 호출을 한 트랜잭션으로. 100번 써도 undo 1스텝, 조판 1회다.
     * 읽기만 모인 배치는 트랜잭션 없이 지나간다.
     */
    const batch = (ops) => {
      const list = Array.isArray(ops) ? ops : [];
      if (!batchNeedsTransaction(list)) {
        return host.read(() => list.map((op) => callOn(ctrl, op.m, op.a)));
      }
      return host.transaction(`hwpctrl:batch(${list.length})`, (tx) => {
        tx.deferPagination();
        return list.map((op) => callOn(ctrl, op.m, op.a));
      });
    };

    return {
      invoke: (method, args) => invoke(method, args),
      batch: (ops) => batch(ops),

      /** 현재 문서를 바이트로. 저장은 호스트(브리지)가 받아 내려받기를 처리한다. */
      exportBytes: (format) => host.read((doc) => {
        if (format === 'hwpx') return doc.exportHwpx();
        if (format === 'hml') return doc.exportHml();
        return doc.exportHwp();
      }),

      /**
       * undo 는 studio 히스토리를 쓴다.
       *
       * 이 층에도 자체 스냅샷 되돌리기가 있지만(standalone 용), plugin 모드에서 그것을 쓰면
       * 사용자가 보는 undo 스택과 두 갈래가 된다. 사용자 눈에 보이는 것 하나만 남긴다.
       */
      undo: () => host.automation.execute('edit:undo'),
      redo: () => host.automation.execute('edit:redo'),

      // ── 좌표 변환 (§9.D 실증 규칙) ────────────────────────
      toStudioPosition: (listId, cellParaIndex) =>
        host.read((doc) => listToStudio(JSON.parse(doc.getCursorModel()), listId, cellParaIndex)),
      toHwpList: (position) =>
        host.read((doc) => studioToList(JSON.parse(doc.getCursorModel()), position)),
      listDepthOf: (listId) =>
        host.read((doc) => listDepth(JSON.parse(doc.getCursorModel()), listId)),

      /** 분류 확인용 — 어떤 이름이 트랜잭션을 타는지 밖에서 볼 수 있어야 한다. */
      isMutating: (method) => isMutating(method),
    };
  },
};

export default hwpctrlStudioPlugin;
