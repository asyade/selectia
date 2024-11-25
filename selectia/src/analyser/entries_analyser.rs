use crate::prelude::*;
use crate::views::entry_view::EntryView;
use nucleo_matcher::{pattern::{CaseMatching, Normalization, Pattern}, Config, Matcher};

pub struct EntriesAnalyser<'a> {
    entries: &'a [EntryView],
}

impl<'a> EntriesAnalyser<'a> {
    pub fn new(entries: &'a [EntryView]) -> Self {
        Self { entries }
    }

    pub fn get_tag_creation_suggestions(&self, tag_name_id: i64, input: &str) -> eyre::Result<Vec<String>> {
        let mut possible_values = HashSet::new();
        for entry in self.entries {
            for tag in entry.tags.0.iter().filter(|tag| tag.tag_name_id == tag_name_id) {
                possible_values.insert(tag.tag_value.clone());
            }
        }

        let mut matcher = Matcher::new(Config::DEFAULT);
        let matches = Pattern::parse(input, CaseMatching::Ignore, Normalization::Smart).match_list(possible_values, &mut matcher);

        Ok(matches.into_iter().map(|(value, _)| value).collect())
    }
}