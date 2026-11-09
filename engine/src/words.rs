//! Offline text sources: common word lists, a handful of quotes, and the
//! generator that turns them into a test text with optional punctuation and
//! numbers.

pub const ENGLISH: &[&str] = &[
    "the", "be", "of", "and", "a", "to", "in", "he", "have", "it", "that", "for", "they", "with",
    "as", "not", "on", "she", "at", "by", "this", "we", "you", "do", "but", "from", "or", "which",
    "one", "would", "all", "will", "there", "say", "who", "make", "when", "can", "more", "if",
    "no", "man", "out", "other", "so", "what", "time", "up", "go", "about", "than", "into",
    "could", "state", "only", "new", "year", "some", "take", "come", "these", "know", "see",
    "use", "get", "like", "then", "first", "any", "work", "now", "may", "such", "give", "over",
    "think", "most", "even", "find", "day", "also", "after", "way", "many", "must", "look",
    "before", "great", "back", "through", "long", "where", "much", "should", "well", "people",
    "down", "own", "just", "because", "good", "each", "those", "feel", "seem", "how", "high",
    "too", "place", "little", "world", "very", "still", "nation", "hand", "old", "life", "tell",
    "write", "become", "here", "show", "house", "both", "between", "need", "mean", "call",
    "develop", "under", "last", "right", "move", "thing", "general", "school", "never", "same",
    "another", "begin", "while", "number", "part", "turn", "real", "leave", "might", "want",
    "point", "form", "off", "child", "few", "small", "since", "against", "ask", "late", "home",
    "interest", "large", "person", "end", "open", "public", "follow", "during", "present",
    "without", "again", "hold", "govern", "around", "possible", "head", "consider", "word",
    "program", "problem", "however", "lead", "system", "set", "order", "eye", "plan", "run",
    "keep", "face", "fact", "group", "play", "stand", "increase", "early", "course", "change",
    "help", "line",
];

pub const FRENCH: &[&str] = &[
    "le", "de", "un", "être", "et", "à", "il", "avoir", "ne", "je", "son", "que", "se", "qui",
    "ce", "dans", "en", "du", "elle", "au", "pour", "pas", "on", "par", "sur", "faire", "plus",
    "dire", "me", "lui", "nous", "comme", "mais", "pouvoir", "avec", "tout", "y", "aller", "voir",
    "bien", "où", "sans", "tu", "ou", "leur", "homme", "si", "deux", "mari", "moi", "vouloir",
    "te", "femme", "venir", "quand", "grand", "celui", "notre", "devoir", "là", "jour", "prendre",
    "même", "votre", "rien", "petit", "encore", "aussi", "quelque", "dont", "mer", "trouver",
    "donner", "temps", "ça", "peu", "autre", "falloir", "sous", "parler", "alors", "main",
    "chose", "ton", "mettre", "vie", "savoir", "yeux", "passer", "après", "regarder", "toujours",
    "puis", "cela", "aimer", "croire", "trois", "monde", "maison", "fois", "porte", "rester",
    "avant", "nuit", "contre", "pendant", "vers", "chez", "depuis", "ville", "terre", "partir",
    "penser", "tête", "année", "enfant", "entre", "heure", "jamais", "mot", "nom", "rendre",
    "sortir", "air", "pays", "travail", "école", "eau", "comprendre", "connaître", "tenir",
    "demander", "lieu", "place", "suivre", "vivre", "beau", "cœur", "ami", "côté", "moment",
    "point", "porter", "rouge", "noir", "blanc", "nouveau", "route", "chemin", "livre", "classe",
    "cours", "train", "parole", "raison", "écrire", "lire", "ouvrir", "simple", "fort", "jeune",
    "lettre", "feu", "matin", "soir", "question", "réponse", "idée", "groupe", "compte", "ligne",
    "nombre", "voix", "pied", "corps", "soleil",
];

pub struct Quote {
    pub text: &'static str,
    pub source: &'static str,
}

pub const QUOTES_EN: &[Quote] = &[
    Quote { text: "Simplicity is prerequisite for reliability.", source: "Edsger W. Dijkstra" },
    Quote { text: "The best way to predict the future is to invent it.", source: "Alan Kay" },
    Quote { text: "Programs must be written for people to read, and only incidentally for machines to execute.", source: "Harold Abelson" },
    Quote { text: "Premature optimization is the root of all evil.", source: "Donald Knuth" },
    Quote { text: "Talk is cheap. Show me the code.", source: "Linus Torvalds" },
    Quote { text: "Any fool can write code that a computer can understand. Good programmers write code that humans can understand.", source: "Martin Fowler" },
    Quote { text: "First, solve the problem. Then, write the code.", source: "John Johnson" },
    Quote { text: "The most disastrous thing that you can ever learn is your first programming language.", source: "Alan Kay" },
    Quote { text: "Controlling complexity is the essence of computer programming.", source: "Brian Kernighan" },
    Quote { text: "Everyone knows that debugging is twice as hard as writing a program in the first place. So if you are as clever as you can be when you write it, how will you ever debug it?", source: "Brian Kernighan" },
    Quote { text: "The question of whether a computer can think is no more interesting than the question of whether a submarine can swim.", source: "Edsger W. Dijkstra" },
    Quote { text: "It is not that I am so smart, it is just that I stay with problems longer.", source: "Albert Einstein" },
    Quote { text: "Education is the most powerful weapon which you can use to change the world.", source: "Nelson Mandela" },
    Quote { text: "If you want to go fast, go alone. If you want to go far, go together.", source: "African proverb" },
    Quote { text: "The man who moves a mountain begins by carrying away small stones.", source: "Confucius" },
    Quote { text: "Tell me and I forget. Teach me and I remember. Involve me and I learn.", source: "Benjamin Franklin" },
    Quote { text: "We are what we repeatedly do. Excellence, then, is not an act, but a habit.", source: "Will Durant" },
    Quote { text: "The only way to learn a new programming language is by writing programs in it.", source: "Dennis Ritchie" },
    Quote { text: "A language that does not affect the way you think about programming is not worth knowing.", source: "Alan Perlis" },
    Quote { text: "Software is like entropy: it is difficult to grasp, weighs nothing, and obeys the second law of thermodynamics; that is, it always increases.", source: "Norman Augustine" },
    Quote { text: "There are two ways of constructing a software design: one way is to make it so simple that there are obviously no deficiencies, and the other way is to make it so complicated that there are no obvious deficiencies. The first method is far more difficult.", source: "C. A. R. Hoare" },
    Quote { text: "Measuring programming progress by lines of code is like measuring aircraft building progress by weight.", source: "Bill Gates" },
    Quote { text: "Perfection is achieved, not when there is nothing more to add, but when there is nothing left to take away.", source: "Antoine de Saint-Exupéry" },
    Quote { text: "Do or do not. There is no try.", source: "Proverb" },
];

pub const QUOTES_FR: &[Quote] = &[
    Quote { text: "Je pense, donc je suis.", source: "René Descartes" },
    Quote { text: "On ne voit bien qu'avec le cœur. L'essentiel est invisible pour les yeux.", source: "Antoine de Saint-Exupéry" },
    Quote { text: "Ce que l'on conçoit bien s'énonce clairement, et les mots pour le dire arrivent aisément.", source: "Nicolas Boileau" },
    Quote { text: "Rien ne sert de courir ; il faut partir à point.", source: "Jean de La Fontaine" },
    Quote { text: "La perfection est atteinte, non pas lorsqu'il n'y a plus rien à ajouter, mais lorsqu'il n'y a plus rien à retirer.", source: "Antoine de Saint-Exupéry" },
    Quote { text: "Le cœur a ses raisons que la raison ne connaît point.", source: "Blaise Pascal" },
    Quote { text: "Petit à petit, l'oiseau fait son nid.", source: "Proverbe" },
    Quote { text: "Il faut cultiver notre jardin.", source: "Voltaire" },
    Quote { text: "Hâtez-vous lentement, et sans perdre courage, vingt fois sur le métier remettez votre ouvrage.", source: "Nicolas Boileau" },
    Quote { text: "Le bois a beau rester longtemps dans l'eau, il ne deviendra jamais un crocodile.", source: "Proverbe africain" },
    Quote { text: "Quand un vieillard meurt en Afrique, c'est une bibliothèque qui brûle.", source: "Amadou Hampâté Bâ" },
    Quote { text: "L'éducation est l'arme la plus puissante que vous puissiez utiliser pour changer le monde.", source: "Nelson Mandela" },
    Quote { text: "La simplicité est la sophistication suprême.", source: "Léonard de Vinci" },
    Quote { text: "Celui qui déplace une montagne commence par déplacer de petites pierres.", source: "Confucius" },
];

/// Quote length buckets, by character count, as typing sites usually split them.
pub fn quote_bucket(text: &str) -> u32 {
    match text.chars().count() {
        0..=100 => 1,
        101..=250 => 2,
        _ => 3,
    }
}

pub fn quotes(language: &str) -> &'static [Quote] {
    if language == "fr" { QUOTES_FR } else { QUOTES_EN }
}

/// Picks a quote of the given length (0 = any), avoiding `exclude` when another
/// one fits, so restarting doesn't serve the same quote twice in a row.
pub fn pick_quote(language: &str, length: u32, exclude: Option<usize>, rng: &mut fastrand::Rng) -> usize {
    let all = quotes(language);
    let mut fitting: Vec<usize> = (0..all.len())
        .filter(|&i| length == 0 || quote_bucket(all[i].text) == length)
        .collect();
    if fitting.is_empty() {
        fitting = (0..all.len()).collect();
    }
    if fitting.len() > 1 {
        if let Some(e) = exclude {
            fitting.retain(|&i| i != e);
        }
    }
    fitting[rng.usize(..fitting.len())]
}

pub struct Generator {
    pub language: String,
    pub punctuation: bool,
    pub numbers: bool,
    rng: fastrand::Rng,
    capitalize_next: bool,
    last: String,
}

impl Generator {
    pub fn new(language: &str, punctuation: bool, numbers: bool, seed: Option<u64>) -> Self {
        Generator {
            language: language.to_string(),
            punctuation,
            numbers,
            rng: seed.map(fastrand::Rng::with_seed).unwrap_or_default(),
            capitalize_next: true,
            last: String::new(),
        }
    }

    pub fn rng(&mut self) -> &mut fastrand::Rng {
        &mut self.rng
    }

    fn list(&self) -> &'static [&'static str] {
        if self.language == "fr" { FRENCH } else { ENGLISH }
    }

    /// The next word. Never repeats the previous word, which reads oddly.
    pub fn next_word(&mut self) -> String {
        if self.numbers && self.rng.f32() < 0.1 {
            let digits = self.rng.u32(1..=4);
            let mut n = self.rng.u32(10u32.pow(digits - 1)..10u32.pow(digits)).to_string();
            if digits == 1 {
                n = self.rng.u32(0..10).to_string();
            }
            self.last = n.clone();
            return self.punctuate(n);
        }
        let list = self.list();
        let mut word = list[self.rng.usize(..list.len())];
        while word == self.last {
            word = list[self.rng.usize(..list.len())];
        }
        self.last = word.to_string();
        self.punctuate(word.to_string())
    }

    fn punctuate(&mut self, word: String) -> String {
        if !self.punctuation {
            return word;
        }
        let mut w = word;
        if self.capitalize_next {
            let mut chars = w.chars();
            if let Some(first) = chars.next() {
                w = first.to_uppercase().collect::<String>() + chars.as_str();
            }
            self.capitalize_next = false;
        }
        let r = self.rng.f32();
        if r < 0.08 {
            w.push('.');
            self.capitalize_next = true;
        } else if r < 0.18 {
            w.push(',');
        } else if r < 0.20 {
            w.push('?');
            self.capitalize_next = true;
        } else if r < 0.22 {
            w.push('!');
            self.capitalize_next = true;
        } else if r < 0.24 {
            w = format!("\"{w}\"");
        } else if r < 0.26 {
            w = format!("({w})");
        } else if r < 0.27 {
            w.push(';');
        } else if r < 0.28 {
            w.push(':');
        } else if r < 0.29 {
            w = format!("{w}-");
            // A dangling hyphen reads as a compound word with the next one.
        }
        w
    }

    pub fn words(&mut self, count: usize) -> Vec<String> {
        (0..count).map(|_| self.next_word()).collect()
    }

    /// Closes a punctuated text with a full stop so it reads as a sentence.
    pub fn finish_text(&self, words: &mut [String]) {
        if !self.punctuation {
            return;
        }
        if let Some(last) = words.last_mut() {
            while last.ends_with([',', ';', ':', '-']) {
                last.pop();
            }
            if !last.ends_with(['.', '?', '!', '"', ')']) {
                last.push('.');
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_words_come_from_the_list() {
        let mut g = Generator::new("en", false, false, Some(1));
        for w in g.words(100) {
            assert!(ENGLISH.contains(&w.as_str()), "{w}");
        }
    }

    #[test]
    fn numbers_appear_when_enabled() {
        let mut g = Generator::new("en", false, true, Some(2));
        assert!(g.words(200).iter().any(|w| w.chars().all(|c| c.is_ascii_digit())));
    }

    #[test]
    fn punctuation_starts_with_a_capital_and_ends_with_a_stop() {
        let mut g = Generator::new("en", true, false, Some(3));
        let mut words = g.words(30);
        g.finish_text(&mut words);
        assert!(words[0].chars().next().unwrap().is_uppercase());
        assert!(words.last().unwrap().ends_with(['.', '?', '!', '"', ')']));
    }

    #[test]
    fn quote_buckets_exist_for_every_length() {
        for lang in ["en", "fr"] {
            for len in 1..=3 {
                let mut rng = fastrand::Rng::with_seed(4);
                let i = pick_quote(lang, len, None, &mut rng);
                assert!(i < quotes(lang).len());
            }
        }
    }
}
