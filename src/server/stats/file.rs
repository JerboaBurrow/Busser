use crate::{config::{Config, CONFIG_PATH}, filesystem::file::{read_file_utf8, write_file_bytes, File}};

use super::hits::{Hit, HitStats};

pub struct StatsFile
{
    pub hits: Vec<Hit>,
    pub path: Option<String>
}

/// Save hits to disk.
///  Each stats files takes the current date YYYY-MM-DD as
///  its file name, if multiple saves occur on the same date
///  the file is appended to
/// See [crate::server::stats::HitStats]
impl StatsFile
{
    pub fn new() -> StatsFile
    {
        StatsFile { hits: vec![], path: None }
    }

    pub fn load(&mut self, stats: &HitStats)
    {
        let mut old_hits: Vec<Hit> = match self.read_utf8()
        {
            Some(d) =>
            {
                match serde_json::from_str(&d)
                {
                    Ok(s) => s,
                    Err(_e) => vec![]
                }
            },
            None => vec![]
        };

        self.hits = stats.hits.values().cloned().collect();
        self.hits.append(&mut old_hits);
    }
}

impl File for StatsFile
{
    fn write_bytes(&self)
    {
        match serde_json::to_string(&self.hits)
        {
            Ok(s) => {write_file_bytes(&self.path(), s.as_bytes())},
            Err(e) => {crate::debug(format!("Error saving stats {}", e), None)}
        }
    }

    fn read_bytes(&self) -> Option<Vec<u8>> {
        match self.read_utf8()
        {
            Some(s) => Some(s.as_bytes().to_vec()),
            None => None
        }
    }

    fn read_utf8(&self) -> Option<String>
    {
        let file_name = StatsFile::path(&self);
        if std::path::Path::new(&file_name).exists()
        {
            read_file_utf8(&file_name)
        }
        else
        {
            None
        }
    }

    fn path(&self) -> String
    {
        match &self.path
        {
            Some(s) => s.clone(),
            None =>
            {
                let config = Config::load_or_default(CONFIG_PATH);
                config.stats.path.to_string()+"/"+&crate::util::date_now()
            }
        }
    }
}