/// Genre-specific pattern collections
///
/// This module provides curated collections of drum patterns organized by
/// musical genre and style. Each collection contains patterns that are
/// commonly used in that genre with appropriate tempo ranges and complexity.

use super::types::{DrumPattern, PatternGenre};
use super::builder::PatternBuilder;
use crate::timing::TimeSignature;

/// Rock pattern collection
pub struct RockPatterns;

impl RockPatterns {
    /// Basic rock beat - the foundation of rock drumming
    pub fn basic_rock() -> DrumPattern {
        PatternBuilder::new("basic_rock", TimeSignature::new(4, 4))
            .display_name("Basic Rock Beat")
            .tempo_range(80, 140)
            .genre(PatternGenre::Rock)
            .difficulty(2)
            .description("Classic rock beat with kick on 1 and 3, snare on 2 and 4")
            .tag("rock").tag("basic").tag("4/4")
            .beat(1.0).kick().hihat_closed().accent().build()
            .beat(1.5).hihat_closed().build()
            .beat(2.0).snare().hihat_closed().build()
            .beat(2.5).hihat_closed().build()
            .beat(3.0).kick().hihat_closed().build()
            .beat(3.5).hihat_closed().build()
            .beat(4.0).snare().hihat_closed().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Power rock beat with more aggressive kick pattern
    pub fn power_rock() -> DrumPattern {
        PatternBuilder::new("power_rock", TimeSignature::new(4, 4))
            .display_name("Power Rock Beat")
            .tempo_range(120, 160)
            .genre(PatternGenre::Rock)
            .difficulty(3)
            .description("Aggressive rock beat with double kick hits")
            .tag("rock").tag("power").tag("aggressive")
            .beat(1.0).kick().hihat_closed().accent().build()
            .beat(1.5).kick().hihat_closed().build()
            .beat(2.0).snare().hihat_closed().build()
            .beat(2.5).hihat_closed().build()
            .beat(3.0).kick().hihat_closed().build()
            .beat(3.5).kick().hihat_closed().build()
            .beat(4.0).snare().hihat_closed().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Half-time rock feel
    pub fn half_time_rock() -> DrumPattern {
        PatternBuilder::new("half_time_rock", TimeSignature::new(4, 4))
            .display_name("Half-Time Rock")
            .tempo_range(80, 120)
            .genre(PatternGenre::Rock)
            .difficulty(2)
            .description("Laid-back half-time rock feel")
            .tag("rock").tag("half-time").tag("laid-back")
            .beat(1.0).kick().hihat_closed().accent().build()
            .beat(1.5).hihat_closed().build()
            .beat(2.0).hihat_closed().build()
            .beat(2.5).hihat_closed().build()
            .beat(3.0).snare().hihat_closed().build()
            .beat(3.5).hihat_closed().build()
            .beat(4.0).hihat_closed().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Get all rock patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::basic_rock(),
            Self::power_rock(),
            Self::half_time_rock(),
        ]
    }
}

/// Jazz pattern collection
pub struct JazzPatterns;

impl JazzPatterns {
    /// Basic swing pattern
    pub fn basic_swing() -> DrumPattern {
        PatternBuilder::new("basic_swing", TimeSignature::new(4, 4))
            .display_name("Basic Swing")
            .tempo_range(120, 200)
            .genre(PatternGenre::Jazz)
            .difficulty(3)
            .description("Classic jazz swing pattern with ride cymbal")
            .tag("jazz").tag("swing").tag("ride")
            .from_notation("...H")  // Simplified notation
            .build()
            .unwrap()
    }

    /// Brushes ballad
    pub fn brushes_ballad() -> DrumPattern {
        PatternBuilder::new("brushes_ballad", TimeSignature::new(4, 4))
            .display_name("Brushes Ballad")
            .tempo_range(60, 100)
            .genre(PatternGenre::Jazz)
            .difficulty(4)
            .description("Soft ballad pattern with brush technique")
            .tag("jazz").tag("ballad").tag("brushes")
            .beat(1.0).kick().accent().build()
            .beat(2.5).snare().build()
            .beat(4.0).snare().build()
            .build()
            .unwrap()
    }

    /// Get all jazz patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::basic_swing(),
            Self::brushes_ballad(),
        ]
    }
}

/// Latin pattern collection
pub struct LatinPatterns;

impl LatinPatterns {
    /// Basic bossa nova pattern
    pub fn bossa_nova() -> DrumPattern {
        PatternBuilder::new("bossa_nova", TimeSignature::new(4, 4))
            .display_name("Bossa Nova")
            .tempo_range(100, 140)
            .genre(PatternGenre::Latin)
            .difficulty(3)
            .description("Classic bossa nova rhythm")
            .tag("latin").tag("bossa").tag("brazilian")
            .beat(1.0).kick().rimshot().accent().build()
            .beat(1.5).rimshot().build()
            .beat(2.5).rimshot().build()
            .beat(3.0).kick().rimshot().build()
            .beat(4.0).rimshot().build()
            .beat(4.5).rimshot().build()
            .build()
            .unwrap()
    }

    /// Samba pattern
    pub fn samba() -> DrumPattern {
        PatternBuilder::new("samba", TimeSignature::new(4, 4))
            .display_name("Samba")
            .tempo_range(120, 180)
            .genre(PatternGenre::Latin)
            .difficulty(4)
            .description("Energetic samba rhythm")
            .tag("latin").tag("samba").tag("brazilian")
            .beat(1.0).kick().accent().build()
            .beat(1.5).snare().build()
            .beat(2.0).kick().build()
            .beat(2.5).snare().build()
            .beat(3.0).kick().build()
            .beat(3.5).snare().build()
            .beat(4.0).kick().build()
            .beat(4.5).snare().build()
            .build()
            .unwrap()
    }

    /// Get all latin patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::bossa_nova(),
            Self::samba(),
        ]
    }
}

/// Funk pattern collection
pub struct FunkPatterns;

impl FunkPatterns {
    /// Basic funk groove
    pub fn basic_funk() -> DrumPattern {
        PatternBuilder::new("basic_funk", TimeSignature::new(4, 4))
            .display_name("Basic Funk Groove")
            .tempo_range(90, 120)
            .genre(PatternGenre::Funk)
            .difficulty(4)
            .description("Classic funk groove with syncopated kicks")
            .tag("funk").tag("groove").tag("syncopated")
            .beat(1.0).kick().hihat_closed().accent().build()
            .beat(1.75).kick().build()
            .beat(2.0).snare().hihat_open().build()
            .beat(2.5).hihat_closed().build()
            .beat(3.0).hihat_closed().build()
            .beat(3.25).kick().build()
            .beat(4.0).snare().hihat_closed().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Linear funk pattern
    pub fn linear_funk() -> DrumPattern {
        PatternBuilder::new("linear_funk", TimeSignature::new(4, 4))
            .display_name("Linear Funk")
            .tempo_range(80, 110)
            .genre(PatternGenre::Funk)
            .difficulty(5)
            .description("Advanced linear funk pattern")
            .tag("funk").tag("linear").tag("advanced")
            .beat(1.0).kick().accent().build()
            .beat(1.25).hihat_closed().build()
            .beat(2.0).snare().build()
            .beat(2.75).kick().build()
            .beat(3.5).hihat_closed().build()
            .beat(4.0).snare().build()
            .build()
            .unwrap()
    }

    /// Get all funk patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::basic_funk(),
            Self::linear_funk(),
        ]
    }
}

/// Pop pattern collection
pub struct PopPatterns;

impl PopPatterns {
    /// Basic pop beat
    pub fn basic_pop() -> DrumPattern {
        PatternBuilder::new("basic_pop", TimeSignature::new(4, 4))
            .display_name("Basic Pop Beat")
            .tempo_range(100, 130)
            .genre(PatternGenre::Pop)
            .difficulty(1)
            .description("Simple and clean pop rhythm")
            .tag("pop").tag("basic").tag("simple")
            .beat(1.0).kick().hihat_closed().accent().build()
            .beat(2.0).snare().hihat_closed().build()
            .beat(3.0).kick().hihat_closed().build()
            .beat(4.0).snare().hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Ballad pattern
    pub fn pop_ballad() -> DrumPattern {
        PatternBuilder::new("pop_ballad", TimeSignature::new(4, 4))
            .display_name("Pop Ballad")
            .tempo_range(60, 90)
            .genre(PatternGenre::Pop)
            .difficulty(2)
            .description("Gentle ballad rhythm with fills")
            .tag("pop").tag("ballad").tag("gentle")
            .beat(1.0).kick().accent().build()
            .beat(2.0).snare().build()
            .beat(3.0).kick().build()
            .beat(4.0).snare().build()
            .beat(4.75).stick().build()  // Small fill
            .build()
            .unwrap()
    }

    /// Get all pop patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::basic_pop(),
            Self::pop_ballad(),
        ]
    }
}

/// Electronic pattern collection
pub struct ElectronicPatterns;

impl ElectronicPatterns {
    /// Four-on-the-floor house beat
    pub fn four_on_floor() -> DrumPattern {
        PatternBuilder::new("four_on_floor", TimeSignature::new(4, 4))
            .display_name("Four-on-the-Floor")
            .tempo_range(120, 140)
            .genre(PatternGenre::Electronic)
            .difficulty(1)
            .description("Classic house music kick pattern")
            .tag("electronic").tag("house").tag("four-on-floor")
            .beat(1.0).kick().accent().build()
            .beat(1.5).hihat_closed().build()
            .beat(2.0).kick().snare().build()
            .beat(2.5).hihat_closed().build()
            .beat(3.0).kick().build()
            .beat(3.5).hihat_closed().build()
            .beat(4.0).kick().snare().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Breakbeat pattern
    pub fn breakbeat() -> DrumPattern {
        PatternBuilder::new("breakbeat", TimeSignature::new(4, 4))
            .display_name("Breakbeat")
            .tempo_range(140, 180)
            .genre(PatternGenre::Electronic)
            .difficulty(4)
            .description("Classic breakbeat rhythm")
            .tag("electronic").tag("breakbeat").tag("complex")
            .beat(1.0).kick().accent().build()
            .beat(1.5).hihat_closed().build()
            .beat(2.0).snare().build()
            .beat(2.25).kick().build()
            .beat(3.25).kick().build()
            .beat(3.75).snare().build()
            .beat(4.0).kick().build()
            .beat(4.5).hihat_closed().build()
            .build()
            .unwrap()
    }

    /// Get all electronic patterns
    pub fn all() -> Vec<DrumPattern> {
        vec![
            Self::four_on_floor(),
            Self::breakbeat(),
        ]
    }
}

/// Master pattern collection containing all genre patterns
pub struct MasterCollection;

impl MasterCollection {
    /// Get all patterns from all genres
    pub fn all() -> Vec<DrumPattern> {
        let mut patterns = Vec::new();
        patterns.extend(RockPatterns::all());
        patterns.extend(JazzPatterns::all());
        patterns.extend(LatinPatterns::all());
        patterns.extend(FunkPatterns::all());
        patterns.extend(PopPatterns::all());
        patterns.extend(ElectronicPatterns::all());
        patterns
    }

    /// Get patterns by genre
    pub fn by_genre(genre: &PatternGenre) -> Vec<DrumPattern> {
        match genre {
            PatternGenre::Rock => RockPatterns::all(),
            PatternGenre::Jazz => JazzPatterns::all(),
            PatternGenre::Latin => LatinPatterns::all(),
            PatternGenre::Funk => FunkPatterns::all(),
            PatternGenre::Pop => PopPatterns::all(),
            PatternGenre::Electronic => ElectronicPatterns::all(),
            _ => vec![], // Other genres not implemented yet
        }
    }

    /// Get patterns suitable for beginners (difficulty 1-2)
    pub fn beginner() -> Vec<DrumPattern> {
        Self::all().into_iter()
            .filter(|p| p.metadata.difficulty <= 2)
            .collect()
    }

    /// Get patterns for intermediate players (difficulty 3)
    pub fn intermediate() -> Vec<DrumPattern> {
        Self::all().into_iter()
            .filter(|p| p.metadata.difficulty == 3)
            .collect()
    }

    /// Get patterns for advanced players (difficulty 4-5)
    pub fn advanced() -> Vec<DrumPattern> {
        Self::all().into_iter()
            .filter(|p| p.metadata.difficulty >= 4)
            .collect()
    }

    /// Get patterns for a specific tempo range
    pub fn for_tempo_range(min_bpm: u32, max_bpm: u32) -> Vec<DrumPattern> {
        Self::all().into_iter()
            .filter(|p| {
                let (pattern_min, pattern_max) = p.tempo_range;
                pattern_min <= max_bpm && pattern_max >= min_bpm
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rock_patterns() {
        let patterns = RockPatterns::all();
        assert!(!patterns.is_empty());

        let basic = RockPatterns::basic_rock();
        assert_eq!(basic.name, "basic_rock");
        assert_eq!(basic.metadata.genre, PatternGenre::Rock);
    }

    #[test]
    fn test_jazz_patterns() {
        let patterns = JazzPatterns::all();
        assert!(!patterns.is_empty());

        for pattern in patterns {
            assert_eq!(pattern.metadata.genre, PatternGenre::Jazz);
        }
    }

    #[test]
    fn test_master_collection() {
        let all_patterns = MasterCollection::all();
        assert!(all_patterns.len() > 5);

        let rock_patterns = MasterCollection::by_genre(&PatternGenre::Rock);
        assert!(!rock_patterns.is_empty());

        let beginner_patterns = MasterCollection::beginner();
        assert!(!beginner_patterns.is_empty());
        assert!(beginner_patterns.iter().all(|p| p.metadata.difficulty <= 2));
    }

    #[test]
    fn test_tempo_filtering() {
        let patterns_120_140 = MasterCollection::for_tempo_range(120, 140);
        assert!(!patterns_120_140.is_empty());

        for pattern in patterns_120_140 {
            let (min, max) = pattern.tempo_range;
            assert!(min <= 140 && max >= 120);
        }
    }

    #[test]
    fn test_difficulty_filtering() {
        let intermediate = MasterCollection::intermediate();
        assert!(intermediate.iter().all(|p| p.metadata.difficulty == 3));

        let advanced = MasterCollection::advanced();
        assert!(advanced.iter().all(|p| p.metadata.difficulty >= 4));
    }
}