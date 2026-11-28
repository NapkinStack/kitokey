//! The formats a summary is written in (D130): the six `summarising/v1` builds in — hidden and shown
//! again, never changed nor deleted — and those a person made, each a name, its sections — a title
//! and what goes in it — and the one holding the actions. One is the default. They are kept in the
//! settings' file, never sent anywhere; `kitokey-summary` is given one as the file its `--format`
//! reads (`summarising/v1`'s `format.schema.json`), written by [`file`].

use serde_json::{Value, json};

use crate::words::{self, Language};

/// The structures `summarising/v1` builds in, in the order they are offered (`minutes/v5`).
pub const BUILT_IN: [&str; 6] = ["general", "one_to_one", "stand_up", "client_call", "interview", "project_review"];

/// One section of a person's format: its title, as they read it in the editor, and what goes in
/// it, as they ask it of the model. Its id in the minutes is drawn from its title.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Section {
    pub title: String,
    pub about: String,
}

/// A format a person made. Its structure is drawn from its first name and kept: renamed, the
/// minutes already written in it still name it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
pub struct Format {
    pub structure: String,
    pub name: String,
    pub sections: Vec<Section>,
    /// The place of the section holding the actions; none, the format has none.
    #[serde(default)]
    pub actions: Option<usize>,
}

/// Why a change to the formats is not made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Refused {
    /// The name is empty, on two lines or longer than 40 characters.
    Name,
    /// Another format of the person's has that name.
    NameTaken,
    /// Not one to six sections.
    Sections,
    /// This section's title is empty, on two lines, longer than 40 characters or another's id;
    /// or what goes in it is empty, on two lines or longer than 200 characters.
    Section(usize),
    /// The section holding the actions is not one of them.
    Actions,
    /// A built-in format is never renamed, changed nor deleted.
    BuiltIn,
    /// A person's format is deleted, never hidden.
    NotBuiltIn,
    /// No format of that structure.
    Unknown,
    /// The default is never hidden.
    Default,
    /// A hidden format is never the default.
    Hidden,
}

/// The formats: the person's own, the built-in ones they hid, and the default.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Formats {
    pub own: Vec<Format>,
    pub hidden: Vec<String>,
    pub default: String,
}

impl Default for Formats {
    fn default() -> Self {
        Self {
            own: Vec::new(),
            hidden: Vec::new(),
            default: BUILT_IN[0].into(),
        }
    }
}

impl Formats {
    /// The formats as the settings' file holds them: each kept as it would have been made, once by
    /// its structure and its name; one written by hand otherwise is left aside. Only a built-in one
    /// is hidden, and the default is one offered — `general`, or the first, otherwise.
    pub fn kept(own: Vec<Format>, hidden: Vec<String>, default: Option<String>) -> Self {
        let mut formats = Self::default();
        for format in own {
            let structure_free = id_of(&format.structure).as_deref() == Some(format.structure.as_str())
                && !BUILT_IN.contains(&format.structure.as_str())
                && formats.own(&format.structure).is_none();
            if let (true, Ok(name), Ok(())) = (
                structure_free,
                formats.free_name(&format.name, None),
                checked(&format.sections, format.actions),
            ) {
                formats.own.push(Format { name, ..format });
            }
        }
        for built in hidden {
            if BUILT_IN.contains(&built.as_str()) && !formats.hidden.contains(&built) {
                formats.hidden.push(built);
            }
        }
        let shown = formats.shown();
        formats.default = match default {
            Some(default) if shown.contains(&default.as_str()) => default,
            _ if shown.contains(&BUILT_IN[0]) => BUILT_IN[0].into(),
            _ => shown.first().copied().unwrap_or(BUILT_IN[0]).into(),
        };
        formats
    }

    /// The structures offered, in their order: the built-in ones not hidden, then the person's.
    pub fn shown(&self) -> Vec<&str> {
        BUILT_IN
            .iter()
            .copied()
            .filter(|built| !self.hidden.iter().any(|hidden| hidden == built))
            .chain(self.own.iter().map(|format| format.structure.as_str()))
            .collect()
    }

    /// The person's format of that structure.
    pub fn own(&self, structure: &str) -> Option<&Format> {
        self.own.iter().find(|format| format.structure == structure)
    }

    /// A format made, kept after the others: its structure.
    pub fn made(&mut self, name: &str, sections: Vec<Section>, actions: Option<usize>) -> Result<String, Refused> {
        let name = self.free_name(name, None)?;
        checked(&sections, actions)?;
        let structure = self.free_structure(&name);
        self.own.push(Format {
            structure: structure.clone(),
            name,
            sections,
            actions,
        });
        Ok(structure)
    }

    /// A copy of a format — the person's or a built-in one, its sections in words — under a new name.
    pub fn duplicated(&mut self, of: &Format, name: &str) -> Result<String, Refused> {
        self.made(name, of.sections.clone(), of.actions)
    }

    pub fn renamed(&mut self, structure: &str, name: &str) -> Result<(), Refused> {
        let at = self.at(structure)?;
        self.own[at].name = self.free_name(name, Some(at))?;
        Ok(())
    }

    pub fn changed(&mut self, structure: &str, sections: Vec<Section>, actions: Option<usize>) -> Result<(), Refused> {
        let at = self.at(structure)?;
        checked(&sections, actions)?;
        self.own[at].sections = sections;
        self.own[at].actions = actions;
        Ok(())
    }

    /// Deleted — once the person confirmed it, which is the screen's to ask (PDR-0008). The default
    /// deleted, `general` is the default again, or the first offered when it is hidden.
    pub fn deleted(&mut self, structure: &str) -> Result<(), Refused> {
        let at = self.at(structure)?;
        self.own.remove(at);
        if self.default == structure {
            let shown = self.shown();
            self.default = shown.first().copied().unwrap_or(BUILT_IN[0]).to_string();
        }
        Ok(())
    }

    /// A built-in format hidden, or shown again.
    pub fn hide(&mut self, structure: &str, hidden: bool) -> Result<(), Refused> {
        if !BUILT_IN.contains(&structure) {
            return Err(if self.own(structure).is_some() {
                Refused::NotBuiltIn
            } else {
                Refused::Unknown
            });
        }
        if hidden && self.default == structure {
            return Err(Refused::Default);
        }
        self.hidden.retain(|kept| kept != structure);
        if hidden {
            self.hidden.push(structure.into());
        }
        Ok(())
    }

    pub fn default_to(&mut self, structure: &str) -> Result<(), Refused> {
        if !BUILT_IN.contains(&structure) && self.own(structure).is_none() {
            return Err(Refused::Unknown);
        }
        if self.hidden.iter().any(|hidden| hidden == structure) {
            return Err(Refused::Hidden);
        }
        self.default = structure.into();
        Ok(())
    }

    /// The place of the person's format of that structure.
    fn at(&self, structure: &str) -> Result<usize, Refused> {
        if BUILT_IN.contains(&structure) {
            return Err(Refused::BuiltIn);
        }
        self.own
            .iter()
            .position(|format| format.structure == structure)
            .ok_or(Refused::Unknown)
    }

    /// The name trimmed, when it is one that no other format has — a built-in one's name in either language
    /// Kitokey speaks, or another of the person's —, whatever its case and its accents (lina, #1324).
    fn free_name(&self, name: &str, besides: Option<usize>) -> Result<String, Refused> {
        let name = name.trim();
        if !one_line(name, 40) {
            return Err(Refused::Name);
        }
        let said = folded(name);
        let built = [Language::English, Language::French]
            .into_iter()
            .flat_map(|language| words::window(language).format_names)
            .any(|built| folded(built) == said);
        let own = self
            .own
            .iter()
            .enumerate()
            .any(|(at, format)| Some(at) != besides && folded(&format.name) == said);
        if built || own {
            return Err(Refused::NameTaken);
        }
        Ok(name.to_string())
    }

    /// A structure drawn from the name that no format has: `_2`, `_3`… when it would be taken.
    fn free_structure(&self, name: &str) -> String {
        let drawn = match id_of(name) {
            Some(drawn) => drawn,
            None => "format".to_string(),
        };
        let taken = |structure: &str| BUILT_IN.contains(&structure) || self.own(structure).is_some();
        if !taken(&drawn) {
            return drawn;
        }
        (2..)
            .map(|n| {
                let end = format!("_{n}");
                let kept: String = drawn.chars().take(40 - end.len()).collect();
                format!("{}{end}", kept.trim_end_matches('_'))
            })
            .find(|structure| !taken(structure))
            .expect("a free structure")
    }
}

/// The file `kitokey-summary --format` reads (`summarising/v1`'s `format.schema.json`).
pub fn file(format: &Format) -> Value {
    let ids = ids(&format.sections);
    let mut file = json!({
        "structure": format.structure,
        "sections": format.sections.iter().zip(&ids).map(|(section, id)| json!({ "id": id, "about": section.about })).collect::<Vec<_>>(),
    });
    if let Some(id) = format.actions.and_then(|at| ids.get(at)) {
        file["actions"] = json!(id);
    }
    file
}

/// Each section's id: drawn from its title, its place when the title holds no letter of [a-z0-9].
fn ids(sections: &[Section]) -> Vec<String> {
    sections
        .iter()
        .enumerate()
        .map(|(at, section)| id_of(&section.title).unwrap_or_else(|| format!("section_{}", at + 1)))
        .collect()
}

/// Within minutes/v5's bounds: one to six sections, each title and what goes in it on one line,
/// each id once, the actions one of them.
fn checked(sections: &[Section], actions: Option<usize>) -> Result<(), Refused> {
    if sections.is_empty() || sections.len() > 6 {
        return Err(Refused::Sections);
    }
    let ids = ids(sections);
    for (at, section) in sections.iter().enumerate() {
        let twice = ids[..at].contains(&ids[at]);
        if !one_line(section.title.trim(), 40) || !one_line(&section.about, 200) || twice {
            return Err(Refused::Section(at));
        }
    }
    if actions.is_some_and(|at| at >= sections.len()) {
        return Err(Refused::Actions);
    }
    Ok(())
}

/// A name as it is compared: lower case, the marks of Latin letters taken off (`plain`).
fn folded(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|character| plain(character).unwrap_or_else(|| character.to_string()))
        .collect()
}

/// A Latin letter without its mark, or one of [a-z0-9] as it is; none for any other character.
fn plain(character: char) -> Option<String> {
    Some(
        match character {
            'a'..='z' | '0'..='9' => return Some(character.to_string()),
            'à' | 'á' | 'â' | 'ã' | 'ä' | 'å' => "a",
            'ç' => "c",
            'è' | 'é' | 'ê' | 'ë' => "e",
            'ì' | 'í' | 'î' | 'ï' => "i",
            'ñ' => "n",
            'ò' | 'ó' | 'ô' | 'õ' | 'ö' | 'ø' => "o",
            'ù' | 'ú' | 'û' | 'ü' => "u",
            'ý' | 'ÿ' => "y",
            'œ' => "oe",
            'æ' => "ae",
            'ß' => "ss",
            _ => return None,
        }
        .to_string(),
    )
}

/// Not empty, not blank, on one line, of at most so many characters.
fn one_line(text: &str, most: usize) -> bool {
    !text.trim().is_empty() && !text.contains(['\n', '\r']) && text.chars().count() <= most
}

/// The word in [a-z0-9_]{1,40}: lower case, the marks of Latin letters taken off, any other
/// character a `_`, none twice nor at an end. None when no letter of [a-z0-9] is left.
fn id_of(text: &str) -> Option<String> {
    let mut id = String::new();
    for character in text.to_lowercase().chars() {
        let plain = plain(character).unwrap_or_else(|| "_".into());
        if plain == "_" && (id.is_empty() || id.ends_with('_')) {
            continue;
        }
        id.push_str(&plain);
    }
    let id: String = id.trim_end_matches('_').chars().take(40).collect();
    let id = id.trim_end_matches('_');
    (!id.is_empty()).then(|| id.to_string())
}
