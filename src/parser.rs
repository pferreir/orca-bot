use anyhow::Result;
use regex::Regex;
use thiserror::Error;

pub struct OrcaSource {
    data: Vec<char>,
    width: u8,
}

pub struct LineIter<'t> {
    source: &'t OrcaSource,
    ptr: usize,
}

impl<'t> Iterator for LineIter<'t> {
    type Item = &'t [char];

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr >= self.source.data.len() {
            None
        } else {
            let line_start = &self.source.data[self.ptr..(self.ptr + self.source.width as usize)];
            self.ptr += self.source.width as usize;
            Some(line_start)
        }
    }
}

impl OrcaSource {
    pub fn iter_lines(&self) -> LineIter<'_> {
        LineIter {
            source: self,
            ptr: 0,
        }
    }
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Line lenghts don't match")]
    MismatchLineLengths,
    #[error("No Orca source code found")]
    NoCodeFound,
    #[error("No run tag found")]
    NoPreludeFound,
    #[error("Lines are too long")]
    LinesTooLong,
}

pub struct ParseConfig<'t> {
    pub tag: &'t str,
    pub max_line_length: u8,
    pub max_num_lines: u8,
}

impl<'t> Default for ParseConfig<'t> {
    fn default() -> Self {
        Self { tag: "run", max_line_length: 64, max_num_lines: 64 }
    }
}

pub(crate) fn parse_orca_code(text: &str, parse_config: &ParseConfig) -> Result<OrcaSource> {
    let re = Regex::new(r"([a-zA-Z0-9#\.\*=]+\s*\n)*[a-zA-Z0-9#\.\*=]+\s*\n?")?;

    match re.find(text) {
        Some(m) => {
            let lines: Vec<_> = m
                .as_str()
                .lines()
                .take(parse_config.max_num_lines as usize)
                .map(|l| l.trim())
                .collect();

            let first_len = lines[0].len();

            if first_len > parse_config.max_line_length as usize {
                return Err(ParseError::LinesTooLong.into());
            }

            if !lines.iter().all(|line| line.len() == first_len) {
                return Err(ParseError::MismatchLineLengths.into());
            }

            Ok(OrcaSource {
                data: lines.join("").chars().collect(),
                width: first_len as u8,
            })
        }
        None => Err(ParseError::NoCodeFound.into()),
    }
}

pub fn parse_html(html: &str, parse_config: &ParseConfig) -> Result<OrcaSource> {
    // get plain text representation, while removing empty lines
    let plain_str = htmd::convert(html)?;
    let plain_lines: Vec<_> = plain_str.lines().filter(|l| !l.trim().is_empty()).collect();

    if !plain_lines[0].contains(parse_config.tag) {
        return Err(ParseError::NoPreludeFound.into());
    }

    // little quirk of markdown conversion: it escapes asterisks
    let unescaped_lines: Vec<_> = plain_lines[1..]
        .iter()
        .map(|l| l.replace(r"\*", "*"))
        .collect();

    parse_orca_code(&unescaped_lines.join("\n"), parse_config)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_PARSE_CONFIG: ParseConfig = ParseConfig {
        max_line_length: 16,
        max_num_lines: 16,
        tag: "run",
    };

    #[test]
    fn test_parsing_ok() {
        let input = ".....C8.........\n......8TCDGCGDCE\n....81X..D..C2..\n..........Y..A4.\n...........=0...";
        let OrcaSource { data, width } = parse_orca_code(input, &DEFAULT_PARSE_CONFIG).unwrap();
        assert!(data == ".....C8...............8TCDGCGDCE....81X..D..C2............Y..A4............=0...".chars().collect::<Vec<_>>());
        assert!(width == 16);
    }

    #[test]
    fn test_parsing_html_ok() {
        let input = "<p><span class=\"h-card\" translate=\"no\"><a href=\"https://fedi.turbofish.cc/@orcabot\" class=\"u-url mention\">@<span>orcabot</span></a></span> <a href=\"https://mastodon.xyz/tags/run\" class=\"mention hashtag status-link\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">#<span>run</span></a><br />.....C8.........<br />......8TCDGCGDCE<br />....81X..D..C2..<br />..........Y..A4.<br />...........=0...</p>";
        let OrcaSource { data, width } = parse_html(input, &DEFAULT_PARSE_CONFIG).unwrap();
        assert!(data == ".....C8...............8TCDGCGDCE....81X..D..C2............Y..A4............=0...".chars().collect::<Vec<_>>());
        assert!(width == 16);

        let input = "<p><span class=\"h-card\" translate=\"no\"><a href=\"https://fedi.turbofish.cc/@orcabot\" class=\"u-url mention\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">@<span>orcabot</span></a></span> please <a href=\"https://mastodon.xyz/tags/run\" class=\"mention hashtag status-link\" rel=\"nofollow noopener noreferrer\" target=\"_blank\">#<span>run</span></a> this<br>.....C8.........<br>......8TCDGCGDCE<br>....81X..D..C2..<br>..........Y..A4.<br>...........=0...</p>";
        let OrcaSource { data, width } = parse_html(input, &DEFAULT_PARSE_CONFIG).unwrap();
        assert!(data == ".....C8...............8TCDGCGDCE....81X..D..C2............Y..A4............=0...".chars().collect::<Vec<_>>());
        assert!(width == 16);
    }

    #[test]
    #[should_panic(expected = "No run tag found")]
    fn test_parsing_html_fail_no_tag() {
        let input = "<p><span class=\"h-card\" translate=\"no\"><a href=\"https://fedi.turbofish.cc/@orcabot\" class=\"u-url mention\">@<span>orcabot</span></a></span><br />.....C8.........<br />......8TCDGCGDCE<br />....81X..D..C2..<br />..........Y..A4.<br />...........=0...</p>";
        parse_html(input, &DEFAULT_PARSE_CONFIG).unwrap();
    }
}
