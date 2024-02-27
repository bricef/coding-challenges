use fantoccini::{ClientBuilder, Locator};

// let's set up the sequence of steps we want the browser to take
#[tokio::main]
async fn main() -> Result<(), fantoccini::error::CmdError> {
    let c = ClientBuilder::native().connect("http://localhost:4444").await.expect("failed to connect to WebDriver");

    // first, go to the Wikipedia page for Foobar
    c.goto("https://en.wikipedia.org/wiki/Foobar").await?;
    let url = c.current_url().await?;
    assert_eq!(url.as_ref(), "https://en.wikipedia.org/wiki/Foobar");

    
    let refs =  c.find_all(Locator::Css("[href]")).await?;

    for r in refs {
        match r.attr("href").await {
            Ok(Some(href)) => println!("{:?}", href),
            Ok(None) => println!("No href"),
            Err(_) => println!("No href"),
        }
    }

    c.close().await
}