# axe-core 4.14.0, vendored for the shell's e2e (D106)

- **What**: `axe.min.js` of [axe-core](https://github.com/dequelabs/axe-core) 4.14.0 (2026-10-05), Deque's
  accessibility engine, with its `LICENSE` (MPL-2.0) and `LICENSE-3RD-PARTY.txt`, unchanged.
- **From**: `https://registry.npmjs.org/axe-core/-/axe-core-4.14.0.tgz`, its integrity checked against the
  registry's: `sha512-9WTZxEjsZ7b13TH8JPmbV2z8CHbl80/2hm3XPEG4JgNdQLK81IBRXmSxHfMAOkSqQeRxT/0dwNDz2GOm3zzpcQ==`.
- **Pinned**: `axe.min.js`'s SHA-256 is
  `20c09fe157a8a34a30e241aaa1fcdade657734f08ab379ecfbeb7d45cc46e878`; the e2e checks it before injecting the
  file and refuses one that differs.
- **Used**: by `scripts/e2e_window.py` alone, injected into the page under test. Never built into the app,
  never shipped in a package.
- **Updated**: by replacing the three files from a newer release and its hash in the e2e, in one pull request.
