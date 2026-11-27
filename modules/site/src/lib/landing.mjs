// The landing's words (D114, proposal D validated by the decider on 2026-11-25), in English and in French,
// each with what keeps its promise, as words.mjs asks. A list's entries are tuples in the order the page
// reads them. French takes its narrow no-break spaces here, once (typo below).
const REPO = "https://github.com/NapkinStack/kitokey";

const raw = {
  title: {
    en: "Kitokey, meeting minutes written on your own computer",
    fr: "Kitokey, des comptes rendus de réunion écrits sur votre ordinateur",
    keeps: "docs/store/listing.md, its description (D103)",
  },
  description: {
    en: "Kitokey records your calls, transcribes them and writes the minutes on your computer. No bot in the meeting, no account, no cloud by default. Free on Linux.",
    fr: "Kitokey enregistre vos appels, les transcrit et en écrit le compte rendu sur votre ordinateur. Sans robot dans la réunion, sans compte, sans cloud par défaut. Gratuit sur Linux.",
    keeps: "docs/store/listing.md (D103); PDR-0001, free on Linux",
  },
  nav: {
    en: [["how", "How it works"], ["privacy", "Privacy"], ["who", "Who it’s for"], ["systems", "Systems & price"], ["faq", "FAQ"], ["install", "Install"]],
    fr: [["parcours", "Comment ça marche"], ["confidentialite", "Confidentialité"], ["pour-qui", "Pour qui"], ["systemes", "Systèmes et prix"], ["faq", "FAQ"], ["installer", "Installer"]],
    keeps: "the page's own sections",
  },
  // Site v7, the hero (validated by the decider on 2026-11-26): PDR-0009's promise, the price, the Store's
  // button, and a drawn scene of the call, the bar and the minutes. French first.
  hero: {
    fr: ["Vos rendez-vous clients ont leur compte rendu.", "Rien ne quitte votre ordinateur.", "Kitokey enregistre l’appel, le transcrit et en écrit le compte rendu, sur votre ordinateur. Il marche avec Zoom, Meet, Teams ou tout autre logiciel d’appel."],
    en: ["Every client call gets its minutes.", "Nothing leaves your computer.", "Kitokey records the call, transcribes it and writes its minutes, on your computer. It works with Zoom, Meet, Teams or any other calling app."],
    keeps: "PDR-0009, the promise; docs/store/listing.md: the minutes written on the computer, any calling app, no bot in the call (D103)",
  },
  buy: {
    fr: ["Essai gratuit, puis 69 € une fois.", "Bientôt dans le Microsoft Store", "Linux, gratuit", "Bientôt sur Mac, au même prix."],
    en: ["Free trial, then €69 once.", "Coming soon to the Microsoft Store", "Linux, free", "Coming to the Mac, at the same price."],
    keeps: "PDR-0009, 69 € once on macOS and Windows after a free trial, the Store's button once its listing exists (until then these words, no badge); PDR-0001, free on Linux",
  },
  assure: {
    fr: ["Sans compte.", "Sans abonnement.", "Aucun robot dans l’appel."],
    en: ["No account.", "No subscription.", "No bot in the call."],
    keeps: "PDR-0001 and PDR-0003, no account and no subscription; modules/capture records the computer's sound, no participant joins the call",
  },
  scene: {
    fr: ["Un appel vidéo en cours. Au-dessus, la barre de Kitokey enregistre depuis 31 minutes. À côté, le compte rendu fini de l’appel, avec ses points saillants et trois actions datées.",
      "Appel vidéo", "Sarah (vous)", "Aujourd’hui, 14:00", "Confidentielle", "Atelier Lumen, cadrage du projet", "31 min · Sarah · Camille, Hugo et Léa",
      "Écrit sur cet ordinateur. Relisez avant de partager.", "Points saillants",
      "Le devis est validé à 18 400 €. Le lancement est fixé au lundi 7 décembre, avec un premier point d’étape à mi-parcours.", "Actions",
      [["Sarah", "Envoyer le planning", "Vendredi"], ["Camille", "Signer le bon de commande", "Lundi"], ["Hugo", "Ouvrir les accès au site", "3 déc."]]],
    en: ["A video call in progress. Above it, Kitokey’s bar has been recording for 31 minutes. Beside it, the call’s finished minutes, with their highlights and three dated actions.",
      "Video call", "Sarah (you)", "Today, 2:00 pm", "Confidential", "Atelier Lumen, project kick-off", "31 min · Sarah · Camille, Hugo and Léa",
      "Written on this computer. Read it before you share it.", "Highlights",
      "The quote is approved at €18,400. The launch is set for Monday 7 December, with a first checkpoint halfway.", "Actions",
      [["Sarah", "Send the schedule", "Friday"], ["Camille", "Sign the purchase order", "Monday"], ["Hugo", "Open the site’s access", "3 Dec"]]],
    keeps: "a drawn scene, made-up names and figures: the compact bar as the app draws it (D125's mock-up, PDR-0008's two voices), the meeting page's minutes (D121); the portraits generated, of people who do not exist (2026-11-25)",
  },
  // Site v7, the walkthrough: one drawn scene in four steps, record, confidential, the two voices, the minutes;
  // four static cards on a narrow screen and under reduced motion. Made-up meeting, French first.
  walk: {
    fr: {
      kicker: "Comment ça marche", h: "Vous écoutez votre client. Kitokey prend les notes.", steps: "Les quatre étapes",
      tabs: [["Enregistrer", "Un clic au début de l’appel. Une petite barre reste à l’écran, rien n’apparaît chez votre client."],
        ["Confidentielle", "Un geste sur le cadenas, pendant ou après l’appel. La réunion n’est alors jamais confiée à un service en ligne."],
        ["Les deux voix se transcrivent", "Votre micro et le son de l’appel restent séparés. Chaque réplique a son heure."],
        ["Le compte rendu s’écrit", "Tout seul à la fin de l’appel, sur votre ordinateur : les points saillants, les actions, qui s’en charge et pour quand."]],
      cards: ["Un clic au début de l’appel. Une petite barre reste à l’écran, rien n’apparaît chez votre client.",
        "Un geste, pendant ou après l’appel. La réunion n’est alors jamais confiée à un service en ligne.",
        "Votre voix et celle de l’appel restent séparées. Chaque réplique a son heure, et vous nommez chaque côté.",
        "Il s’écrit tout seul après l’appel : les points saillants, les actions, qui s’en charge et pour quand."],
      lockOn: "Confidentielle, activée", cloud: ["Service en ligne", "Jamais pour cette réunion"],
      lanes: [["Vous", "votre micro"], ["Les autres", "le son de l’appel"]], note: "Rien à installer dans Zoom, Meet ou Teams.",
      banner: "Cette réunion est confidentielle. Aucune IA en ligne ne la lit, et le partage demande votre confirmation.",
      share: "Partager", tabsPage: ["Compte rendu", "Transcription"], meta: "31 min · Sarah · Camille, Hugo et Léa · Écrit sur cet ordinateur",
      lines: [["00:03", 0, "On reprend le devis ligne par ligne."], ["00:09", 1, "La formation peut passer à deux jours."],
        ["00:16", 0, "Alors on reste à 18 400 € pour l’ensemble ?"], ["00:21", 1, "D’accord, et on lance le lundi 7 décembre."]],
      writing: ["Rédaction du compte rendu", "sur cet ordinateur"],
      text: "Le devis est validé à 18 400 €. Le lancement est fixé au lundi 7 décembre.",
      mini: ["Devis validé. Lancement le 7 décembre.", [["Sarah", "Planning détaillé", "Vendredi"], ["Camille", "Bon de commande", "Lundi"]]],
    },
    en: {
      kicker: "How it works", h: "You listen to your client. Kitokey takes the notes.", steps: "The four steps",
      tabs: [["Record", "One click as the call starts. A small bar stays on screen, and nothing shows on your client’s side."],
        ["Confidential", "One gesture on the lock, during or after the call. The meeting is then never given to an online service."],
        ["Both voices are transcribed", "Your microphone and the call’s sound stay apart. Each line has its time."],
        ["The minutes are written", "On their own when the call ends, on your computer: the highlights, the actions, who takes them and by when."]],
      cards: ["One click as the call starts. A small bar stays on screen, and nothing shows on your client’s side.",
        "One gesture, during or after the call. The meeting is then never given to an online service.",
        "Your voice and the call’s stay apart. Each line has its time, and you name each side.",
        "They are written on their own after the call: the highlights, the actions, who takes them and by when."],
      lockOn: "Confidential, on", cloud: ["Online service", "Never for this meeting"],
      lanes: [["You", "your microphone"], ["Others", "the call’s sound"]], note: "Nothing to install in Zoom, Meet or Teams.",
      banner: "This meeting is confidential. No online AI reads it, and sharing it asks for your confirmation.",
      share: "Share", tabsPage: ["Minutes", "Transcript"], meta: "31 min · Sarah · Camille, Hugo and Léa · Written on this computer",
      lines: [["00:03", 0, "Let’s go through the quote line by line."], ["00:09", 1, "The training can go down to two days."],
        ["00:16", 0, "So we stay at €18,400 for the whole?"], ["00:21", 1, "Agreed, and we start on Monday 7 December."]],
      writing: ["Writing the minutes", "on this computer"],
      text: "The quote is approved at €18,400. The launch is set for Monday 7 December.",
      mini: ["Quote approved. Launch on 7 December.", [["Sarah", "Detailed schedule", "Friday"], ["Camille", "Purchase order", "Monday"]]],
    },
    keeps: "a drawn scene, made-up meeting: modules/capture records the microphone and the call's sound apart, no bot (D103); the bar's lock makes a meeting confidential, never given to an online service (D110, D118, D119); each line timed, each side named (#1205); the minutes written after the recording, on the computer (#1276), highlights and dated actions (minutes/v5)",
  },
  // Site v7, what stays and what leaves, who it is for, and what is ready after the call. Each claim with what
  // holds it in « keeps »; « Bientôt » on what main does not hold yet. French first.
  sovereign: {
    fr: {
      kicker: "Confidentialité", h: "Vous restez souverain.", lead: "Pas de serveur Kitokey, pas de compte. Ce qui reste sur votre ordinateur, et ce qui n’en sort que si vous le décidez.",
      here: ["Reste sur votre ordinateur", "Toujours, dans le dossier de votre choix.", "Sur votre ordinateur"],
      stays: [["mic", "L’audio de l’appel", "Votre micro et le son de l’appel, en deux pistes."], ["file", "La transcription", "Écrite par un moteur d’écoute installé chez vous."], ["sparkles", "Le compte rendu", "Rédigé sur cet ordinateur, par défaut."]],
      leaves: ["Ne sort que si vous le choisissez", "Rien de ceci n’est activé au départ.", "Désactivé"],
      chosen: [["file", "Un service en ligne, si vous le choisissez", "Avec votre clé, pour aller plus vite. Il reçoit le texte pour rédiger, ou le son pour transcrire, selon ce que vous lui confiez."], ["share", "Un compte rendu que vous envoyez", "Vers Slack ou Discord, d’un geste, quand vous le décidez."]],
      proof: [["lock", "Confidentielle, vraiment", "Une réunion confidentielle n’est jamais confiée à un moteur en ligne."], ["download", "Téléchargé une fois", "Les moteurs se téléchargent une fois, au premier usage."], ["code", "Vérifiable", "Le code source est public, sous GPL v3. Chacun peut lire ce que fait Kitokey."], ["eyeoff", "Ni cookie ni traceur", "Ce site ne vous suit pas, et Kitokey non plus."]],
      policy: "Lire la politique de confidentialité",
      shot: ["Capture de l’app", "La page d’une réunion, filmée dans l’app après sa refonte."],
    },
    en: {
      kicker: "Privacy", h: "You stay in control.", lead: "No Kitokey server, no account. What stays on your computer, and what leaves it only if you decide so.",
      here: ["Stays on your computer", "Always, in the folder you choose.", "On your computer"],
      stays: [["mic", "The call’s audio", "Your microphone and the call’s sound, in two tracks."], ["file", "The transcript", "Written by a listening engine installed on your computer."], ["sparkles", "The minutes", "Written on this computer, by default."]],
      leaves: ["Leaves only if you choose", "None of this is on at first.", "Off"],
      chosen: [["file", "An online service, if you choose one", "With your key, to go faster. It receives the text to write, or the sound to transcribe, as you entrust it."], ["share", "Minutes you send", "To Slack or Discord, in one gesture, when you decide."]],
      proof: [["lock", "Confidential, really", "A confidential meeting is never given to an online engine."], ["download", "Downloaded once", "The engines are downloaded once, on first use."], ["code", "Checkable", "The source code is public, under GPL v3. Anyone can read what Kitokey does."], ["eyeoff", "No cookie, no tracker", "This site does not follow you, and neither does Kitokey."]],
      policy: "Read the privacy policy",
      shot: ["App capture", "A meeting’s page, filmed in the app after its redesign."],
    },
    keeps: "PDR-0001 and PDR-0003, no server and no account, no telemetry; session-audio, two tracks (modules/capture); the transcription engine (D40) and the summary's « Auto », on this computer by default (D122); D96 and D110, an online service only with the user's key and choice; D116, a send on a gesture; D60 and D118, a confidential meeting never online; registry/models.json, the engines downloaded once; LICENSE, GPL v3; _headers, no third-party request on the site. The capture is a placeholder until D121 is filmed",
  },
  who: {
    fr: {
      kicker: "Pour qui", h: "Pour les conversations qui ne regardent que vous.",
      cases: [["briefcase", "Conseil, avocats, experts-comptables, coachs", "Le rendez-vous client", "Vous écoutez votre client au lieu de prendre des notes. Le compte rendu est prêt à relire et à lui remettre.", ["Atelier Lumen, cadrage du projet", "Mar. 17 nov. · 31 min", true]],
        ["usercheck", "Ressources humaines, managers", "L’entretien de recrutement", "Vous restez attentif au candidat. Ses réponses sont notées, et vous comparez les entretiens sur pièces.", ["Entretien, cheffe de projet", "Mer. 11 nov. · 40 min", true]],
        ["building", "Direction, finance", "La réunion interne sensible", "Budget, organisation, situation d’une personne. Les décisions et les actions sont notées sans passer par un service en ligne.", ["Comité budget, T1", "Jeu. 19 nov. · 45 min", false]]],
    },
    en: {
      kicker: "Who it’s for", h: "For the conversations that are nobody else’s business.",
      cases: [["briefcase", "Consultants, lawyers, accountants, coaches", "The client meeting", "You listen to your client instead of taking notes. The minutes are ready to read over and hand over.", ["Atelier Lumen, project kick-off", "Tue 17 Nov · 31 min", true]],
        ["usercheck", "HR, managers", "The job interview", "You stay with the candidate. Their answers are noted, and you compare interviews on the record.", ["Interview, project manager", "Wed 11 Nov · 40 min", true]],
        ["building", "Leadership, finance", "The sensitive internal meeting", "Budget, organisation, a person’s situation. Decisions and actions are noted without going through an online service.", ["Budget committee, Q1", "Thu 19 Nov · 45 min", false]]],
    },
    keeps: "PDR-0009, the professionals whose client calls are confidential; made-up meetings; the summary's highlights and dated actions (minutes/v5); a confidential meeting, its lock (D110)",
  },
  ready: {
    fr: {
      kicker: "Fonctions", h: "Après l’appel, tout est prêt.", lead: "Le compte rendu ne reste pas dans un tiroir : il part où vous travaillez.", soon: "Bientôt",
      feats: [["share", "Partager d’un geste", "Vers Slack, Discord ou un webhook, quand vous le décidez, jamais tout seul.", false],
        ["calendar", "Les dates dans votre agenda", "Les échéances dites pendant l’appel vont dans Google Agenda ou dans un fichier .ics.", false],
        ["headphones", "Réécouter un passage", "Un chiffre à vérifier ? Cliquez la phrase, vous entendez le moment de l’appel.", false],
        ["folder", "Un fichier dans votre dossier", "Si vous choisissez un dossier dans les Réglages, vos comptes rendus s’y enregistrent tout seuls en Markdown. Les réunions confidentielles restent dans Kitokey.", false],
        ["history", "Les versions gardées", "Refaites un compte rendu sans perdre le précédent, et revenez-y si vous préférez.", false],
        ["file", "Exporter en PDF", "Un PDF accessible, prêt pour le dossier client, depuis le bouton Partager.", true],
        ["search", "Chercher dans ce qui a été dit", "Un mot suffit pour retrouver la réunion et le passage où il a été prononcé.", true],
        ["format", "Vos propres formats", "Choisissez les rubriques de vos comptes rendus, et gardez-en un par type de rendez-vous.", true]],
    },
    en: {
      kicker: "Features", h: "After the call, everything is ready.", lead: "The minutes don’t sit in a drawer: they go where you work.", soon: "Soon",
      feats: [["share", "Share in one gesture", "To Slack, Discord or a webhook, when you decide, never on its own.", false],
        ["calendar", "Dates in your calendar", "Deadlines said during the call go to Google Calendar or into an .ics file.", false],
        ["headphones", "Hear a passage again", "A figure to check? Click the line and you hear that moment of the call.", false],
        ["folder", "A file in your folder", "If you choose a folder in Settings, your summaries are saved there on their own as Markdown. Confidential meetings stay in Kitokey.", false],
        ["history", "Versions kept", "Redo a summary without losing the previous one, and go back to it if you prefer.", false],
        ["file", "Export to PDF", "An accessible PDF, ready for the client file, from the Share button.", true],
        ["search", "Search what was said", "One word finds the meeting and the passage where it was said.", true],
        ["format", "Your own formats", "Choose the sections of your minutes, and keep one per kind of meeting.", true]],
    },
    keeps: "on main: D116 the send, D113 the dates and .ics (RFC 5545), D26 a line played, D109 every summary written to a folder in Markdown, never a confidential meeting's (D60, e2e CF), D113 the previous version kept; « Bientôt », cycle 12 and not on main yet: D121 the PDF from the window (the program is D108's, merged), D127 the search, D129 and D130 the formats. Importing a file is not planned: not said",
  },
  cta: {
    en: ["Install on Linux, free", "Follow the releases"],
    fr: ["Installer sur Linux, gratuit", "Suivre les sorties"],
    keeps: "PDR-0001, free on Linux; the repository's tags, each a release (v0.2.6, 2026-11-18)",
  },
  systems: {
    en: ["Systems & price", "Free on Linux. One payment elsewhere.", "No account, no subscription. Linux today, Windows and macOS next.", "No subscription. The price will be announced at launch."],
    fr: ["Systèmes et prix", "Gratuit sur Linux. Un seul paiement ailleurs.", "Pas de compte, pas d’abonnement. Linux aujourd’hui, Windows et macOS ensuite.", "Sans abonnement. Le prix sera annoncé à la sortie."],
    keeps: "PDR-0001: free on Linux, paid once on macOS and Windows",
  },
  oses: {
    en: [[["linux"], "Linux", "Available", "Free", "Fedora, Ubuntu and Debian 13.", true],
      [["windows"], "Windows", "Coming soon", "One-time purchase", "A free trial, then a single payment. Coming to the Microsoft Store.", false],
      [["apple"], "macOS", "Coming soon", "One-time purchase", "A free trial, then a single payment.", false]],
    fr: [[["linux"], "Linux", "Disponible", "Gratuit", "Fedora, Ubuntu et Debian 13.", true],
      [["windows"], "Windows", "Bientôt", "Achat unique", "Un essai gratuit, puis un seul paiement. À venir dans le Microsoft Store.", false],
      [["apple"], "macOS", "Bientôt", "Achat unique", "Un essai gratuit, puis un seul paiement.", false]],
    keeps: "README.md, « Install it »: Fedora from Copr, Ubuntu 24.04 and Debian 13; PDR-0001 and PDR-0005, the Store's unlimited trial; ADR-0017, macOS sold once",
  },
  install: {
    en: ["Install", "Install Kitokey.", "Pick your system, open a terminal and paste the commands.", "Copy", "System", "Copied", "Select and copy"],
    fr: ["Installer", "Installez Kitokey.", "Choisissez votre système, ouvrez un terminal et collez les commandes.", "Copier", "Système", "Copié", "Sélectionnez et copiez"],
    keeps: "README.md, « Install it », the commands below",
  },
  channelNotes: {
    en: { debian: "On Debian 13, the package built for Ubuntu 24.04 installs and starts. Debian 12 is not supported yet.", ubuntu: "On Ubuntu 24.04, build the package. It needs <code>git</code>, <a href=\"https://docs.astral.sh/uv/\">uv</a> and <code>podman</code>. If <code>nstack</code> is then not found, <code>uv tool update-shell</code> puts it on your path. The build takes a while." },
    fr: { debian: "Sur Debian 13, le paquet construit pour Ubuntu 24.04 s’installe et démarre. Debian 12 n’est pas encore pris en charge.", ubuntu: "Sur Ubuntu 24.04, compilez le paquet. Il faut <code>git</code>, <a href=\"https://docs.astral.sh/uv/\">uv</a> et <code>podman</code>. Si <code>nstack</code> est ensuite introuvable, <code>uv tool update-shell</code> le met dans votre chemin. La construction prend un moment." },
    keeps: "README.md: « Needs git, uv and podman », and uv tool update-shell when nstack is not found; README.md, « Install it »: the Ubuntu package installs and starts on Debian 13, not on 12 (#1214)",
  },
  needs: {
    en: ["Prerequisites", [["transcript", "Transcription.", "At first start, its model (874 MB) downloads in one click."],
      ["summary", "Ollama, for the minutes.", "<a href=\"https://ollama.com/download\">Install Ollama</a>, which is free. Kitokey then offers its model, Qwen3 8B (about 5 GB), downloaded in one click."]]],
    fr: ["Prérequis", [["transcript", "La transcription.", "Au premier démarrage, son modèle (874 Mo) se télécharge en un clic."],
      ["summary", "Ollama, pour le compte rendu.", "<a href=\"https://ollama.com/download\">Installez Ollama</a>, qui est gratuit. Kitokey propose ensuite son modèle, Qwen3 8B (environ 5 Go), téléchargé en un clic."]]],
    keeps: "ADR-0003, the 874 MB model; cycle 03, qwen3:8b about 5 GB fetched through Ollama; docs/support.md",
  },
  hardware: {
    en: [["lock", "Everything on your computer", "Confidentiality assured, your data stays with you.",
      ["16 GB of memory at least, 24 to 32 GB to work in comfort", "A graphics card with Vulkan, a built-in one being enough", "10 GB free for the two models", "About 2 GB free for each hour that records, then 0.2 GB kept an hour"]],
      ["unlock", "With the service you choose", "No special hardware. The transcript and the minutes are made online.",
      ["An OpenAI-compatible service, such as Groq", "Often paid, billed by that service, not by Kitokey", "About 2 GB free while an hour records, then 0.2 GB kept"]]],
    fr: [["lock", "Tout sur votre ordinateur", "Confidentialité assurée, vos données restent chez vous.",
      ["16 Go de mémoire au moins, 24 à 32 Go pour être à l’aise", "Une carte graphique Vulkan, même intégrée", "10 Go libres pour les deux modèles", "Environ 2 Go libres par heure qui s’enregistre, puis 0,2 Go gardés par heure"]],
      ["unlock", "Avec le service de votre choix", "Aucun matériel particulier. La transcription et le compte rendu se font en ligne.",
      ["Un service compatible OpenAI, comme Groq", "Souvent payant, facturé par ce service, pas par Kitokey", "Environ 2 Go libres pendant l’enregistrement d’une heure, puis 0,2 Go gardés"]]],
    keeps: "README.md, « What a computer needs », measured on 2026-11-25: its two ways; ADR-0013, a transcription service the user names (Groq among them), ADR-0013 and the minutes' services, their cost the user's; the decider's « no special hardware » (2026-11-25)",
  },
  faq: {
    en: ["Frequently asked questions", [
      ["Do I need to install anything in Zoom, Meet or Teams?", "No. Kitokey records your computer’s sound. It works with any call app and no bot joins the meeting."],
      ["Does Kitokey recognise each participant?", "No. It separates two sides, your microphone (You) and the call’s audio (Others), and you can rename either. It does not tell the other participants’ voices apart."],
      ["Does Kitokey install Ollama?", "No, never. Ollama is a free program of its own, at <a href=\"https://ollama.com/download\">ollama.com</a>. Once it runs, Kitokey’s settings offer its model for the minutes."],
      ["What does Kitokey do without Ollama?", "It records and transcribes your meetings on your computer. Their minutes wait until Ollama runs, and the window tells you so. You can also choose an online service, with your own key."],
      ["Where are my meetings, and how do I delete them?", "In your Documents/Kitokey folder, unless you choose another. A meeting is deleted from its own page. To erase everything, uninstall Kitokey and delete that folder."],
      ["Can the minutes be wrong?", "Yes. They are written by AI, and Kitokey says so at the top of every summary. Read them before you share them. The transcript can be corrected line by line."],
      ["How long do the minutes take?", "It depends on your computer and the length of the call, because the work happens on your machine and not on a server. Think minutes rather than seconds."],
      ["Does it work on Wayland, with PipeWire?", "Yes, on GNOME (Wayland), with sound through PipeWire. That is the environment its end-to-end tests run in. Other desktops are not tested yet."],
      ["Am I allowed to record a call?", "It is up to you to tell the participants you are recording and to get their consent. It is common courtesy, and often the law. Kitokey reminds you, and the reminder can be turned off."],
      ["Which languages are transcribed?", "Kitokey detects the spoken language, among the many Whisper hears, or you choose it for a meeting among eight: English, French, German, Spanish, Italian, Portuguese, Dutch and Arabic."]]],
    fr: ["Questions fréquentes", [
      ["Faut-il installer quelque chose dans Zoom, Meet ou Teams ?", "Non. Kitokey enregistre le son de votre ordinateur. Il fonctionne avec n’importe quelle application d’appel et aucun robot ne rejoint la réunion."],
      ["Kitokey reconnaît-il chaque participant ?", "Non. Il sépare deux côtés, votre micro (Vous) et le son de l’appel (Les autres), et vous pouvez renommer chacun. Il ne distingue pas les voix des autres participants entre elles."],
      ["Kitokey installe-t-il Ollama ?", "Non, jamais. Ollama est un programme gratuit à part, sur <a href=\"https://ollama.com/download\">ollama.com</a>. Une fois qu’il tourne, les réglages de Kitokey proposent son modèle pour les comptes rendus."],
      ["Que fait Kitokey sans Ollama ?", "Il enregistre et transcrit vos réunions sur votre ordinateur. Leurs comptes rendus attendent qu’Ollama tourne, et la fenêtre vous le dit. Vous pouvez aussi choisir un service en ligne, avec votre propre clé."],
      ["Où sont mes réunions, et comment les supprimer ?", "Dans votre dossier Documents/Kitokey, sauf si vous en choisissez un autre. Une réunion se supprime depuis sa page. Pour tout effacer, désinstallez Kitokey puis supprimez ce dossier."],
      ["Le compte rendu peut-il se tromper ?", "Oui. Il est rédigé par une IA, et Kitokey le rappelle en tête de chaque compte rendu. Relisez-le avant de le partager. La transcription se corrige ligne par ligne."],
      ["Combien de temps faut-il pour le compte rendu ?", "Cela dépend de votre ordinateur et de la durée de l’appel, car le travail se fait sur votre machine et pas sur un serveur. Comptez des minutes plutôt que des secondes."],
      ["Fonctionne-t-il sous Wayland, avec PipeWire ?", "Oui, sous GNOME (Wayland), le son passe par PipeWire. C’est l’environnement où tournent ses tests de bout en bout. Les autres bureaux ne sont pas encore testés."],
      ["Ai-je le droit d’enregistrer un appel ?", "C’est à vous d’annoncer l’enregistrement aux participants et d’obtenir leur accord. C’est la règle de bon sens, et souvent celle de la loi. Kitokey vous le rappelle, et ce rappel peut se désactiver."],
      ["Quelles langues sont transcrites ?", "Kitokey détecte la langue parlée, parmi les nombreuses que Whisper entend, ou vous la choisissez pour une réunion parmi huit : le français, l’anglais, l’allemand, l’espagnol, l’italien, le portugais, le néerlandais et l’arabe."]]],
    keeps: "docs/support.md (Ollama, the folder); #1211 and #1205, the two sides named; words.rs minutes_ai; docs/privacy.md, the reminder; cycle 01, GNOME on Wayland; spoken.rs, the eight languages offered and any other of ISO 639-1, none said: the engine detects it (D64, D65)",
  },
  final: {
    en: "Try it on your next call.",
    fr: "Essayez-le sur votre prochain appel.",
    keeps: "the install above",
  },
};

// French spacing: U+202F before ; ! ? and inside « », U+00A0 before a colon (never in a link or markup).
function typo(v) {
  if (typeof v === "string") {
    const t = v.replace(/ ([;!?])/g, " $1").replace(/« /g, "« ").replace(/ »/g, " »");
    return /<|:\/\//.test(t) ? t : t.replace(/ :/g, " :");
  }
  if (Array.isArray(v)) return v.map(typo);
  if (v && typeof v === "object") return Object.fromEntries(Object.entries(v).map(([k, x]) => [k, typo(x)]));
  return v;
}

export const landing = Object.fromEntries(Object.entries(raw).map(([k, w]) => [k, { ...w, fr: typo(w.fr) }]));
export const repo = REPO;
// The install's commands, as README.md gives them: Fedora from Copr (D78, published 2026-11-18); Ubuntu built as
// README.md says, until a repository is published; Debian 13 takes Ubuntu's package (README.md, « Install it »).
export const channels = [
  ["fedora", "Fedora", "sudo dnf copr enable @napkinstack/kitokey\nsudo dnf install kitokey"],
  ["ubuntu", "Ubuntu 24.04", "git clone https://github.com/NapkinStack/kitokey && cd kitokey\nbash .nstack/install-engine.sh\nKITOKEY_TARGET=ubuntu-24.04 nstack release\nsudo apt install ./dist/install/kitokey_*_amd64.deb"],
  ["debian", "Debian 13", "git clone https://github.com/NapkinStack/kitokey && cd kitokey\nbash .nstack/install-engine.sh\nKITOKEY_TARGET=ubuntu-24.04 nstack release\nsudo apt install ./dist/install/kitokey_*_amd64.deb"],
];
