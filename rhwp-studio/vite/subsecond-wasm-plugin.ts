import { createReadStream } from 'node:fs';
import { join } from 'node:path';
import type { Plugin } from 'vite';

const SUBSECOND_WASM = /^(?:rhwp-subsecond_bg|librhwp-subsecond-patch-\d+)\.wasm$/;

/** Serve the base and patch modules from dx's local output, independent of the dx process. */
export function subsecondWasmPlugin(wasmDir: string): Plugin {
  return {
    name: 'serve-subsecond-wasm',
    configureServer(server) {
      server.middlewares.use('/wasm', (req, res, next) => {
        let fileName: string;
        try {
          fileName = decodeURIComponent(req.url?.split('?')[0] ?? '').replace(/^\/+/, '');
        } catch {
          res.statusCode = 400;
          return res.end();
        }
        if (!SUBSECOND_WASM.test(fileName)) return next();

        const stream = createReadStream(join(wasmDir, fileName));
        res.once('close', () => stream.destroy());
        stream.once('error', () => {
          if (!res.headersSent) {
            res.statusCode = 404;
            res.end();
          } else {
            res.destroy();
          }
        });
        stream.once('open', () => {
          res.setHeader('Content-Type', 'application/wasm');
          res.setHeader('Cache-Control', 'no-store');
          stream.pipe(res);
        });
      });
    },
  };
}
