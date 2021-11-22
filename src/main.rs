use futures::StreamExt;

use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::PrintToPdfParams;

fn sleep_ms(ms: u64) {
    std::thread::sleep(std::time::Duration::from_millis(ms));
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cov_username =
        std::env::var("COVERITY_USER").expect("COVERITY_USER environment variable is not set");
    let cov_password =
        std::env::var("COVERITY_PASS").expect("COVERITY_PASS environment variable is not set");

    eprintln!("Starting...");

    // create a `Browser` that spawns a `chromium` process running with UI (`with_head()`, headless is default)
    // and the handler that drives the websocket etc.
    let (browser, mut handler) = Browser::launch(BrowserConfig::builder().build()?).await?;
    // Browser::launch(BrowserConfig::builder().with_head().build()?).await?;

    // spawn a new task that continuously polls the handler
    let handle = async_std::task::spawn(async move {
        loop {
            let _ = handler.next().await.unwrap();
        }
    });

    // create a new browser page and navigate to the url
    let page = browser.new_page("https://scan.coverity.com/").await?;
    eprintln!("sleeping before first action");
    sleep_ms(1000);

    eprintln!("clicking");

    page.find_element(".btn-default").await?.click().await?;
    eprintln!("clicked!");

    let html = page.wait_for_navigation().await?.content().await?;
    eprintln!("Login page loaded, enter username/pass");
    sleep_ms(1000);
    eprintln!("fill username");
    page.find_element("#user_email")
        .await?
        .click()
        .await?
        .type_str(cov_username)
        .await?;
    eprintln!("fill password");
    page.find_element("#user_password")
        .await?
        .click()
        .await?
        .type_str(cov_password)
        .await?;
    eprintln!("press button");
    page.find_element("input.btn.btn-primary")
        .await?
        .click()
        .await?;
    eprintln!("waiting for loading...");

    page.wait_for_navigation().await?;
    eprintln!("Logged in");
    sleep_ms(500);
    page.goto("https://scan.coverity.com/projects/fd-io-vpp/view_defects")
        .await?;
    let html = page.wait_for_navigation().await?.content().await?;
    eprintln!("Sleeping before the final step...");

    for i in 1..5 {
        async_std::task::sleep(std::time::Duration::from_millis(3000)).await;
        eprintln!("Page url: {:?}", page.url().await?);
    }

    eprintln!("going to the new URL");
    let new_url =
        "https://scan9.coverity.com/api/viewContents/issues/v1/28863?projectId=12999&rowCount=-1";

    page.goto(new_url).await?;

    let html = page.wait_for_navigation().await?.content().await?;

    for i in 1..5 {
        async_std::task::sleep(std::time::Duration::from_millis(3000)).await;
        eprintln!("Page url: {:?}", page.url().await?);
    }

    let print_params = PrintToPdfParams::builder().build();
    page.save_pdf(print_params, "/tmp/coverity.pdf").await?;

    let data = page.find_element("pre").await?.inner_text().await?;

    println!("{}", data.unwrap());

    // handle.await;
    Ok(())
}
