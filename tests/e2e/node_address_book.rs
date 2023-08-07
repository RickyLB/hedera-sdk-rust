use hedera::NodeAddressBookQuery;

use crate::common::TestEnvironment;

// a fairly trivial function, but one worth having because it actually tests mirror-net.
#[tokio::test]
async fn query_address_book() -> anyhow::Result<()> {
    let TestEnvironment { client, config: _ } = crate::common::setup_global();

    let _ = NodeAddressBookQuery::new().execute(&client).await?;

    Ok(())
}
