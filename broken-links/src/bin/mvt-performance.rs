
use std::time::Instant;
use headless_chrome::Browser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error>{
    let browser = Browser::default()?;
    let tab = browser.new_tab()?;
    tab.navigate_to("https://en.wikipedia.org")?;
    tab.wait_until_navigated()?;

    let start = Instant::now();
    let _es = tab.find_elements("[href]")?;
    let duration = start.elapsed();
    
    println!("Time elapsed in find_elements() is: {:?}", duration);
    Ok(())


}
