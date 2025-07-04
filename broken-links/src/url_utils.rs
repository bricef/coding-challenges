




use anyhow::{anyhow, Result};
use url::{Url, ParseError};

pub fn canonical (base : &Url, url: &str) -> Result<Url, anyhow::Error> {
    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            if !scheme.is_empty() && !scheme.starts_with("http") {
                // println!("Invalid scheme for {}: {}", parsed.as_str(), parsed.scheme());
                return Err(anyhow!("Invalid scheme for {}", parsed.as_str()));
            }
            let mut parsed = parsed;
            parsed.set_fragment(None);
            Ok(parsed)
        },
        Err(ParseError::RelativeUrlWithoutBase) => {
            let mut parsed = base.join(url)?;
            parsed.set_fragment(None);
            Ok(parsed)
        },
        Err(e) => {
            println!("Invalid URL: {}", e);
            Err(anyhow!("Invalid URL: {}", e))
        }
    }
}


#[cfg(test)]
mod test{
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn test_canonical() {
        let base = Url::parse("http://example.com").unwrap();
        let canon = move | url: &str | canonical(&base, url);

        assert_eq!(canon("http://example.com").unwrap().to_string(), "http://example.com/");
        assert_eq!(canon("https://example.com").unwrap().to_string(), "https://example.com/");
        assert_eq!(canon("ftp://example.com").unwrap_err().to_string(), "Invalid scheme for ftp://example.com/");
        assert_eq!(canon("/foo/bar").unwrap().to_string(), "http://example.com/foo/bar");
    }

}