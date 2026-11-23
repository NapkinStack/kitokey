// Every sentence the site says, in English and in French, with what keeps its promise (D114,
// criterion 3): a merged deliverable or a test, as docs/store/listing.md does. A sentence without
// `keeps` fails the test; a sentence promising what the product does not do has no place here.
export const words = {
  description: {
    en: "Kitokey records your call, transcribes it and writes its summary, on your computer. No bot joins your meeting, no account is asked for.",
    fr: "Kitokey enregistre votre appel, le transcrit et en écrit le compte rendu, sur votre ordinateur. Aucun robot ne rejoint votre réunion, aucun compte n’est demandé.",
    keeps: "docs/store/listing.md, its description (D103), each line held by the tests it names",
  },
  local: {
    en: "Your meetings stay on your computer. A service you trust stays your choice, and a meeting you say confidential never leaves.",
    fr: "Vos réunions restent sur votre ordinateur. Un service de confiance reste votre choix, et une réunion dite confidentielle ne part jamais.",
    keeps: "docs/privacy.md; the shell's tests of the confidential meeting (D110, PDR-0007)",
  },
  privacy: { en: "Privacy policy", fr: "Politique de confidentialité", keeps: "docs/privacy.md (D103), this site's /privacy/ (D114)" },
  other: { en: "Français", fr: "English", keeps: "the page in the other language (D114)" },
};

export const say = (key, lang) => words[key][lang];
