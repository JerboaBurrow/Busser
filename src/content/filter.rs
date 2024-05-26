use crate::util::matches_one;

use super::HasUir;

pub struct ContentFilter
{
    ignore_patterns: Vec<String>
}

impl ContentFilter
{
    pub fn new(ignore_patterns: Vec<String>) -> ContentFilter
    {
        ContentFilter { ignore_patterns }
    }

    pub fn filter<T: HasUir>(&self, items: Vec<T>) -> Vec<T>
    {
        items.into_iter().filter(|item| !matches_one(&item.get_uri(), &self.ignore_patterns)).collect()
    }

    pub fn filter_uris(&self, uris: Vec<String>) -> Vec<String>
    {
        uris.into_iter().filter(|item| !matches_one(&item, &self.ignore_patterns)).collect()
    }
}