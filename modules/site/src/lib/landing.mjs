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
    en: [["privacy", "Privacy"], ["uses", "Use cases"], ["systems", "Systems & price"], ["faq", "FAQ"], ["install", "Install"]],
    fr: [["confidentialite", "Confidentialité"], ["usages", "Usages"], ["systemes", "Systèmes et prix"], ["faq", "FAQ"], ["installer", "Installer"]],
    keeps: "the page's own sections",
  },
  hero: {
    en: ["Listen.", "Kitokey takes the minutes.", "Meeting minutes written on your own computer, with no bot and no cloud."],
    fr: ["Écoutez.", "Kitokey prend les notes.", "Des comptes rendus écrits sur votre ordinateur, sans robot ni cloud."],
    keeps: "docs/store/listing.md: the minutes written on the computer, no bot in the call (D103)",
  },
  cta: {
    en: ["Install on Linux, free", "Follow the releases"],
    fr: ["Installer sur Linux, gratuit", "Suivre les sorties"],
    keeps: "PDR-0001, free on Linux; the repository's tags, each a release (v0.2.6, 2026-11-18)",
  },
  slider: {
    en: ["Kitokey, feature by feature", "slide", "carousel", "of"],
    fr: ["Kitokey, fonction par fonction", "diapositive", "carrousel", "sur"],
    keeps: "the carousel's own words (WAI-ARIA's carousel pattern)",
  },
  call: {
    en: ["Video call", "(you)", "Sarah records a video call with Camille, Hugo and Léa. At the top, Kitokey’s small bar shows the elapsed time, a level trace for Sarah’s microphone and one for the call’s audio."],
    fr: ["Appel vidéo", "(vous)", "Sarah enregistre un appel vidéo avec Camille, Hugo et Léa. En haut, la petite barre de Kitokey montre le temps écoulé, une trace de niveau pour le micro de Sarah et une pour le son de l’appel."],
    keeps: "proposal D's drawn call (the decider's request, 2026-11-25): generated portraits of people who do not exist; the bar as the app draws it while recording (#1179, « B · Trace »)",
  },
  // The slides: the clips of c16, filmed from the app at main dad969f (shell e2e, run i-sam-film16c), in their
  // order; the record slide shows the drawn call instead, the decider's request (2026-11-25).
  slides: {
    en: [["record", "mic", "Record", "One click, and you listen instead of taking notes.", "", "Sarah records a video call with Camille, Hugo and Léa. At the top, Kitokey’s small bar shows the elapsed time, a level trace for Sarah’s microphone and one for the call’s audio."],
      ["transcript", "transcript", "Transcribe", "The whole meeting in writing, on your computer.", "", "The transcript of the weekly team check-in. The two sides are renamed Sarah and Camille, Hugo and Léa, each line with its time."],
      ["summary", "summary", "Minutes", "Decisions and actions, ready to share.", "", "The minutes of the weekly team check-in, with the highlights, then Hugo’s, Camille’s and Léa’s actions and their due dates."],
      ["dates", "calendar", "Calendar", "Deadlines in Google Calendar, or as an .ics file for Outlook and Apple Calendar.", "", "The Dates mentioned panel. A deadline is ticked, then the cursor opens Google Calendar."],
      ["send", "send", "Share", "Shared in one click to Slack, Discord or your webhook.", "", "The cursor sends the minutes to the product team channel, then the Sent to line appears."],
      ["confidential", "lock", "Confidential", "One switch, and no online AI ever sees the meeting.", "", "During a recording, the cursor turns on the Confidential switch, and the padlock shows."]],
    fr: [["record", "mic", "Enregistrer", "Un clic, et vous écoutez au lieu de noter.", "", "Sarah enregistre un appel vidéo avec Camille, Hugo et Léa. En haut, la petite barre de Kitokey montre le temps écoulé, une trace de niveau pour le micro de Sarah et une pour le son de l’appel."],
      ["transcript", "transcript", "Transcrire", "Toute la réunion par écrit, sur votre ordinateur.", "", "La transcription du point d’équipe hebdo. Les deux côtés sont renommés Sarah et Camille, Hugo et Léa, chaque réplique avec son heure."],
      ["summary", "summary", "Résumer", "Les décisions et les actions, prêtes à partager.", "", "Le compte rendu du point d’équipe hebdo, avec les points saillants puis les actions de Hugo, Camille et Léa et leurs échéances."],
      ["dates", "calendar", "Agenda", "Les échéances dans Google Agenda, ou en fichier .ics pour Outlook et Apple Calendrier.", "", "Le panneau Dates mentionnées. Une échéance est cochée, puis le curseur ouvre Google Agenda."],
      ["send", "send", "Partager", "Partagé en un clic sur Slack, Discord ou votre webhook.", "", "Le curseur envoie le compte rendu au canal de l’équipe produit, puis la ligne Envoyé à apparaît."],
      ["confidential", "lock", "Confidentielle", "Un interrupteur, et aucune IA en ligne ne voit la réunion.", "", "Pendant un enregistrement, le curseur active l’interrupteur Confidentielle, et le cadenas s’affiche."]],
    keeps: "the decider's round 9 (2026-11-25): each clip shows it, filmed from the app (c16c); #1179 the trace; #1205 the names; D113 the dates, .ics as RFC 5545; D116 the send; #1208 the padlock, D110 the confidential meeting",
  },
  channels: {
    en: ["The minutes go where your team works.", "In one click, to a Slack channel, a Discord channel or your webhook. Nothing leaves without your gesture, and never a confidential meeting without your confirmation.", "Works with"],
    fr: ["Le compte rendu part là où l’équipe travaille.", "En un clic, vers un canal Slack, un salon Discord ou votre webhook. Rien ne part sans votre geste, jamais pour une réunion confidentielle sans votre confirmation.", "Fonctionne avec"],
    keeps: "PDR-0007 and D116, the sends to a channel; D119, a confidential meeting's send asked once more",
  },
  privacy: {
    en: ["Privacy", "On your computer, by default.", "Nothing of a meeting leaves your computer unless you decide so. No account, no bot in the call."],
    fr: ["Confidentialité", "Sur votre ordinateur, par défaut.", "Rien d’une réunion ne quitte votre ordinateur, sauf si vous le décidez. Pas de compte, pas de robot dans l’appel."],
    keeps: "docs/privacy.md, « By default, nothing of a meeting »",
  },
  promises: {
    en: [["mic", "No bot in the call.", "Kitokey listens to your computer’s sound. Nobody sees an extra participant join."],
      ["lock", "A Confidential switch.", "A meeting marked so is never given to an online AI, even if your settings chose one."],
      ["summary", "Free to choose your AI.", "A model on your computer, or the online service of your choice with your own key. That service’s cost is not included."],
      ["code", "Verifiable.", `The source code is public, under GPL v3. <a href="${REPO}">Read it on GitHub</a>.`]],
    fr: [["mic", "Aucun robot dans l’appel.", "Kitokey écoute le son de votre ordinateur. Personne ne voit arriver un participant de plus."],
      ["lock", "Un interrupteur Confidentielle.", "Une réunion marquée ainsi n’est jamais confiée à une IA en ligne, même si vos réglages en prévoient une."],
      ["summary", "Libre de choisir votre IA.", "Un modèle sur votre ordinateur, ou le service en ligne de votre choix avec votre propre clé. Le coût de ce service n’est pas inclus."],
      ["code", "Vérifiable.", `Le code source est public, sous licence GPL v3. <a href="${REPO}">Lisez-le sur GitHub</a>.`]],
    keeps: "docs/privacy.md (a confidential meeting made on this computer alone, D110); ADR-0013, a service the user names with their key; LICENSE",
  },
  proof: {
    en: "Only what you ask for leaves. A meeting’s sound and its transcript, to the online service you chose to transcribe it or write its minutes. The models you download (huggingface.co and Ollama’s registry), the list of models (raw.githubusercontent.com) and the minutes you send. This site uses no cookie and no tracker.",
    fr: "Ne sortent que ce que vous demandez. Le son et la transcription d’une réunion, vers le service en ligne que vous avez choisi pour la transcrire ou en écrire le compte rendu. Les modèles que vous téléchargez (huggingface.co et le registre d’Ollama), la liste des modèles (raw.githubusercontent.com) et les comptes rendus que vous envoyez. Ce site n’utilise ni cookie ni traceur.",
    keeps: "docs/privacy.md, its table of what leaves and to which host, and « This website »; tests/own-domain.test.mjs",
  },
  local: {
    en: ["Settings", "Model settings. For transcription, On this computer is chosen. Whisper, 874 MB, works offline and nothing is sent."],
    fr: ["Réglages", "Les réglages des modèles. Pour la transcription, Sur cet ordinateur est choisi. Whisper, 874 Mo, fonctionne hors connexion et rien n’est envoyé."],
    keeps: "a crop of D103's settings shot, the app as it is; ADR-0003, the 874 MB model",
  },
  uses: {
    en: ["Use cases", "Every meeting gets its minutes.", "The ones that start a project, the ones that come back every morning and the ones that must not leave the room."],
    fr: ["Usages", "Chaque réunion a son compte rendu.", "Celles qui lancent un projet, celles qui reviennent chaque matin et celles qui ne doivent pas sortir du bureau."],
    keeps: "the cases below, each kept by its own line",
  },
  cases: {
    en: [["summary", "Project kick-off", "Walk out with the plan.", "Who does what, and by when. The actions are in the minutes and their dates go to your calendar."],
      ["send", "The daily stand-up", "Whoever missed it is up to date.", "The stand-up’s minutes go to the team in one gesture. Nobody had to take notes."],
      ["transcript", "Client call", "Find what the client said.", "One search across all your meetings takes you to the exact passage."],
      ["lock", "Lawyers, doctors, advisers", "Confidential stays in the room.", "One switch, and the call goes through no online service."],
      ["mic", "Job interview", "Watch the candidate, not your notes.", "Every answer is transcribed and the minutes keep what matters to compare interviews. Tell the candidate you are recording."],
      ["export", "Export", "As Markdown, HTML, Word or subtitles."],
      ["play", "Play back a passage", "Click a line of the transcript to hear that moment again."],
      ["search", "Your files, at home", "Each meeting stays in Documents/Kitokey, with its recording, its transcript and its minutes."]],
    fr: [["summary", "Lancement de projet", "Repartez avec le plan.", "Qui fait quoi, et pour quand. Les actions sont dans le compte rendu et leurs dates vont dans votre agenda."],
      ["send", "Le point du matin", "Les absents sont à jour.", "Le compte rendu du point d’équipe part à l’équipe d’un geste. Personne n’a eu à prendre de notes."],
      ["transcript", "Appel client", "Retrouvez ce que le client a dit.", "Une recherche dans toutes vos réunions vous ramène au passage exact."],
      ["lock", "Avocat, médecin, conseil", "Le secret reste au cabinet.", "Un interrupteur, et l’appel ne passe par aucun service en ligne."],
      ["mic", "Entretien de recrutement", "Regardez le candidat, pas vos notes.", "Chaque réponse est transcrite et le compte rendu en garde l’essentiel pour comparer les entretiens. Annoncez au candidat que vous enregistrez."],
      ["export", "Exportez", "En Markdown, HTML, Word ou sous-titres."],
      ["play", "Réécoutez un passage", "Cliquez sur une phrase de la transcription pour réentendre ce moment."],
      ["search", "Vos fichiers, chez vous", "Chaque réunion reste dans Documents/Kitokey, avec son enregistrement, sa transcription et son compte rendu."]],
    keeps: "minutes/v5's actions and D113's dates; D116, the send; docs/store/listing.md, one search; D110, the confidential switch; exporting/v1, md html docx srt vtt; D26, play from a line; docs/privacy.md, the meetings folder",
  },
  systems: {
    en: ["Systems & price", "Free on Linux. One payment elsewhere.", "No account, no subscription. Linux today, Windows and macOS next.", "No subscription. The price will be announced at launch."],
    fr: ["Systèmes et prix", "Gratuit sur Linux. Un seul paiement ailleurs.", "Pas de compte, pas d’abonnement. Linux aujourd’hui, Windows et macOS ensuite.", "Sans abonnement. Le prix sera annoncé à la sortie."],
    keeps: "PDR-0001: free on Linux, paid once on macOS and Windows",
  },
  oses: {
    en: [[["fedora", "ubuntu"], "Linux", "Available", "Free", "Fedora and Ubuntu.", true],
      [["windows"], "Windows", "Coming soon", "One-time purchase", "A free trial, then a single payment. Coming to the Microsoft Store.", false],
      [["apple"], "macOS", "Coming soon", "One-time purchase", "A free trial, then a single payment.", false]],
    fr: [[["fedora", "ubuntu"], "Linux", "Disponible", "Gratuit", "Fedora et Ubuntu.", true],
      [["windows"], "Windows", "Bientôt", "Achat unique", "Un essai gratuit, puis un seul paiement. À venir dans le Microsoft Store.", false],
      [["apple"], "macOS", "Bientôt", "Achat unique", "Un essai gratuit, puis un seul paiement.", false]],
    keeps: "README.md, « Install it »: Fedora from Copr, Ubuntu 24.04; PDR-0001 and PDR-0005, the Store's unlimited trial; ADR-0017, macOS sold once",
  },
  install: {
    en: ["Install", "Install Kitokey.", "Pick your system, open a terminal and paste the commands.", "Copy", "System", "Copied", "Select and copy"],
    fr: ["Installer", "Installez Kitokey.", "Choisissez votre système, ouvrez un terminal et collez les commandes.", "Copier", "Système", "Copié", "Sélectionnez et copiez"],
    keeps: "README.md, « Install it », the commands below",
  },
  channelNotes: {
    en: { ubuntu: "On Ubuntu 24.04, build the package. It needs <code>git</code>, <a href=\"https://docs.astral.sh/uv/\">uv</a> and <code>podman</code>. If <code>nstack</code> is then not found, <code>uv tool update-shell</code> puts it on your path. The build takes a while." },
    fr: { ubuntu: "Sur Ubuntu 24.04, compilez le paquet. Il faut <code>git</code>, <a href=\"https://docs.astral.sh/uv/\">uv</a> et <code>podman</code>. Si <code>nstack</code> est ensuite introuvable, <code>uv tool update-shell</code> le met dans votre chemin. La construction prend un moment." },
    keeps: "README.md: « Needs git, uv and podman », and uv tool update-shell when nstack is not found",
  },
  needs: {
    en: ["Prerequisites", [["transcript", "Transcription.", "At first start, its model (874 MB) downloads in one click."],
      ["summary", "Ollama, for the minutes.", "<a href=\"https://ollama.com/download\">Install Ollama</a>, which is free. Kitokey then offers its model, Qwen3 8B (about 5 GB), downloaded in one click."]]],
    fr: ["Prérequis", [["transcript", "La transcription.", "Au premier démarrage, son modèle (874 Mo) se télécharge en un clic."],
      ["summary", "Ollama, pour le compte rendu.", "<a href=\"https://ollama.com/download\">Installez Ollama</a>, qui est gratuit. Kitokey propose ensuite son modèle, Qwen3 8B (environ 5 Go), téléchargé en un clic."]]],
    keeps: "ADR-0003, the 874 MB model; cycle 03, qwen3:8b about 5 GB fetched through Ollama; docs/support.md",
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
// README.md says, until a repository is published.
export const channels = [
  ["fedora", "Fedora", "sudo dnf copr enable @napkinstack/kitokey\nsudo dnf install kitokey"],
  ["ubuntu", "Ubuntu 24.04", "git clone https://github.com/NapkinStack/kitokey && cd kitokey\nbash .nstack/install-engine.sh\nKITOKEY_TARGET=ubuntu-24.04 nstack release\nsudo apt install ./dist/install/kitokey_*_amd64.deb"],
];
