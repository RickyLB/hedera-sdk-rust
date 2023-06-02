use assert_matches::assert_matches;
use hedera::{
    AccountCreateTransaction,
    AccountDeleteTransaction,
    AccountInfoQuery,
    Hbar,
    PrivateKey,
    Status,
};

use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn create_then_delete() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(())
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(())
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    AccountDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .account_id(account_id)
        .sign(key)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let res = AccountInfoQuery::new().account_id(account_id).execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::QueryPaymentPreCheckStatus {
            status: Status::AccountDeleted,
            transaction_id: _
        })
    );

    Ok(())
}

#[tokio::test]
async fn missing_account_id_fails() {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return;
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return;
    };

    let res =
        AccountDeleteTransaction::new().transfer_account_id(op.account_id).execute(&client).await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::AccountIdDoesNotExist,
            transaction_id: _
        })
    );
}

#[tokio::test]
async fn missing_deletee_signature_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config, client }) = setup_nonfree() else {
        return Ok(())
    };

    let Some(op) = &config.operator else {
        log::debug!("skipping test due to missing operator");
        return Ok(())
    };

    let key = PrivateKey::generate_ed25519();

    let receipt = AccountCreateTransaction::new()
        .key(key.public_key())
        .initial_balance(Hbar::new(1))
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    let account_id = receipt.account_id.unwrap();

    let res = AccountDeleteTransaction::new()
        .transfer_account_id(op.account_id)
        .account_id(account_id)
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus { status: Status::InvalidSignature, transaction_id: _ })
    );

    Ok(())
}
