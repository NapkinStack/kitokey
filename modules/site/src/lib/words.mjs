// Every sentence the site says, in English and in French, with what keeps its promise (D114,
// criterion 3): a merged deliverable or a test, as docs/store/listing.md does. A sentence without
// `keeps` fails the test; a sentence promising what the product does not do has no place here.
export const words = {
  title: { en: "Kitokey — be in the meeting", fr: "Kitokey — soyez dans la réunion", keeps: "the product's name and its listing's short description (D103)" },
  description: {
    en: "Kitokey records your call, transcribes it and writes its summary, on your computer. No bot joins your meeting, no account is asked for.",
    fr: "Kitokey enregistre votre appel, le transcrit et en écrit le compte rendu, sur votre ordinateur. Aucun robot ne rejoint votre réunion, aucun compte n’est demandé.",
    keeps: "docs/store/listing.md, its description (D103), each line held by the tests it names",
  },
  promise: {
    en: "Be in the meeting. The summary writes itself, on your computer.",
    fr: "Soyez dans la réunion. Le compte rendu s’écrit tout seul, sur votre ordinateur.",
    keeps: "docs/store/listing.md, its short description (D103)",
  },
  local: {
    en: "Your meetings stay on your computer. A service you trust stays your choice, and a meeting you say confidential never leaves.",
    fr: "Vos réunions restent sur votre ordinateur. Un service de confiance reste votre choix, et une réunion dite confidentielle ne part jamais.",
    keeps: "docs/privacy.md; the shell's tests of the confidential meeting (D110, PDR-0007)",
  },
  price: { en: "Price: coming soon.", fr: "Prix : bientôt annoncé.", keeps: "no price until the decider announces it (D102)" },
  windows: { en: "Microsoft Store: coming soon.", fr: "Microsoft Store : bientôt.", keeps: "the submission, once D102 and D103 are merged" },
  linux: {
    en: "Free on Fedora, from Copr.",
    fr: "Gratuit sur Fedora, depuis Copr.",
    keeps: "PDR-0001; the install module's README, « On Fedora, from Copr » (D78)",
  },
  privacy: { en: "Privacy policy", fr: "Politique de confidentialité", keeps: "docs/privacy.md (D103), this site's /privacy/ (D114)" },
  other: { en: "Français", fr: "English", keeps: "the page in the other language (D114)" },
};

export const say = (key, lang) => words[key][lang];
