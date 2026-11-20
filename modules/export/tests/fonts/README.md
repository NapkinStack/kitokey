# The PDF tests' fonts

Static subsets of Noto, made for `tests/pdf.rs` alone: each holds only the characters the tests write, so
that the tests read no font from the system they run on. None of them ships with Kitokey: the program
gives typst the system's fonts (D108, the coordinator's call by delegation of the decider, 2026-11-20).

| File | From | Copyright |
|---|---|---|
| `NotoSans-Regular-subset.ttf`, `NotoSans-Bold-subset.ttf` | Fedora's `NotoSans[wght].ttf` 2.015, instanced at `wght` 400 and 700; Basic Latin, Latin-1, Latin Extended-A, Cyrillic, the punctuation the tests use | Copyright 2022 The Noto Project Authors |
| `NotoSansSC-Regular-subset.otf`, `NotoSansSC-Bold-subset.otf` | Fedora's `NotoSansCJK-VF.ttc` 2.004, its « Noto Sans CJK SC » face, instanced at `wght` 400 and 700; the CJK characters of the tests | © 2014-2021 Adobe, with Reserved Font Name 'Source' — not used by these files, named « Noto Sans CJK SC » |
| `NotoColorEmoji-subset.ttf` | Fedora's `Noto-COLRv1.ttf` 2.057; 👍 and 🎉 | Copyright 2022 Google Inc. |

All under the SIL Open Font License 1.1: [`OFL.txt`](./OFL.txt). Made with fontTools 4.68
(`fontTools.varLib.instancer.instantiateVariableFont`, then `fontTools.subset` with every layout feature
and name kept). A test that writes a new character adds it to the subset first.
