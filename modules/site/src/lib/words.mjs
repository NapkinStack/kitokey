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
  changed: { en: "Last changed:", fr: "Dernière modification :", keeps: "the policy's first line, the day it last changed (docs/privacy.md)" },
  other: { en: "Français", fr: "English", keeps: "the page in the other language (D114)" },
  theme: { en: ["Switch to the dark theme", "Switch to the light theme"], fr: ["Passer en thème sombre", "Passer en thème clair"], keeps: "public/theme.js, the moon and the sun (the decider's round 9, 2026-11-25)" },
  credit: { en: "Tux: Larry Ewing (lewing@isc.tamu.edu) and The GIMP.", fr: "Tux : Larry Ewing (lewing@isc.tamu.edu) et The GIMP.", keeps: "Tux's own terms, its author credited where it is shown (BRANDS.md, Wikimedia Commons)" },
  footer: { en: "Footer", fr: "Pied de page", keeps: "the footer's links (site v7)" },
  lost: {
    en: ["Not found", "This page doesn’t exist, or no longer does.", "Back to the home page"],
    fr: ["Page introuvable", "Cette page n’existe pas, ou plus.", "Revenir à l’accueil"],
    keeps: "/404.html, the page Cloudflare Pages answers an unknown address with (tests/notfound.test.mjs)",
  },
  legal: { en: "Legal notice", fr: "Mentions légales", keeps: "/<lang>/legal/, the publisher and the host (the decider, 2026-11-27; src/lib/legal.mjs)" },
  legalFields: {
    en: ["Publisher", "Name", "Legal form", "Share capital", "Registration", "VAT number", "Registered office", "Contact", "Director of the publication", "Host", "Address", "Phone"],
    fr: ["Éditeur", "Dénomination", "Forme", "Capital", "Immatriculation", "Numéro de TVA", "Siège", "Contact", "Directeur de la publication", "Hébergeur", "Adresse", "Téléphone"],
    keeps: "the fields of the legal notice the decider asked for on 2026-11-27, each from the build's environment or the host's own terms",
  },
  marks: {
    en: ["Fedora is a trademark or registered trademark of Red Hat, Inc., or its subsidiaries in the United States and other countries.", "Kitokey is not affiliated with Debian. Debian is a registered trademark owned by Software in the Public Interest, Inc.", "Debian Open Use Logo © 1999 Software in the Public Interest, Inc., CC BY-SA 3.0."],
    fr: ["Fedora is a trademark or registered trademark of Red Hat, Inc., or its subsidiaries in the United States and other countries.", "Kitokey is not affiliated with Debian. Debian is a registered trademark owned by Software in the Public Interest, Inc.", "Debian Open Use Logo © 1999 Software in the Public Interest, Inc., CC BY-SA 3.0."],
    keeps: "Fedora's trademark guidelines, Proper Trademark Use (read 2026-11-27); Debian's trademark policy 2.0 and its Open Use Logo's licence (BRANDS.md), said in English as their owners write them",
  },
  // « Me prévenir de la sortie » (ADR-0021): the form in the price, and the three pages /notify answers with.
  notify: {
    en: ["Your e-mail address", "One message, when Kitokey is out in the Microsoft Store, then we delete your address. You confirm it first, by a link we send you.", "What we do with it", "Tell me"],
    fr: ["Votre adresse e-mail", "Un seul message, à la sortie de Kitokey dans le Microsoft Store, puis nous effaçons votre adresse. Vous la confirmez d’abord, par un lien que nous vous envoyons.", "Ce que nous en faisons", "Me prévenir"],
    keeps: "functions/notify.js and Brevo's double opt-in (ADR-0021); docs/privacy.md, This website: one message at the release, then the list deleted",
  },
  notified: {
    en: {
      sent: ["Check your inbox", "We sent you a link to confirm your address. Until you click it, we won’t write to you."],
      failed: ["That didn’t go through", "The address wasn’t recognised, or the service didn’t answer. Check it and try again.", "Try again"],
      confirmed: ["You’re on the list", "Your address is confirmed. We’ll write to you once, when Kitokey is out, then delete it."],
    },
    fr: {
      sent: ["Vérifiez votre boîte", "Nous vous avons envoyé un lien pour confirmer votre adresse. Tant que vous n’avez pas cliqué, nous ne vous écrivons pas."],
      failed: ["Votre demande n’est pas passée", "L’adresse n’a pas été reconnue, ou le service n’a pas répondu. Vérifiez-la et réessayez.", "Réessayer"],
      confirmed: ["C’est noté", "Votre adresse est confirmée. Nous vous écrirons une fois, à la sortie de Kitokey, puis nous l’effacerons."],
    },
    keeps: "functions/notify.js's three answers (tests/notify.test.mjs); Brevo's double opt-in, its confirmation link landing on confirmed (ADR-0021)",
  },
  sections: { en: "Sections", fr: "Sections", keeps: "the header's links to the page's sections (site v7)" },
  skip: { en: "Skip to content", fr: "Aller au contenu", keeps: "each page's main, #main (tests/frame.test.mjs)" },
  publisher: { en: "Kitokey, by C12C.", fr: "Kitokey, par C12C.", keeps: "the publisher's name, proposal D's footer, validated by the decider on 2026-11-25" },
  source: { en: "Source code (GPL v3)", fr: "Code source (GPL v3)", keeps: "the repository's LICENSE, the GNU GPL version 3" },
};

export const say = (key, lang) => words[key][lang];
