use assert_matches::assert_matches;
use hedera::{
    Hbar,
    Status,
    TokenAssociateTransaction,
    TokenRevokeKycTransaction,
};

use crate::account::Account;
use crate::common::{
    setup_nonfree,
    TestEnvironment,
};

#[tokio::test]
async fn basic() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else { return Ok(()) };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(&client, &alice, 0).await?;

    TokenAssociateTransaction::new()
        .account_id(bob.id)
        .token_ids([token.id])
        .sign(bob.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    TokenRevokeKycTransaction::new()
        .account_id(bob.id)
        .token_id(token.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await?;

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}

#[tokio::test]
async fn missing_token_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else { return Ok(()) };

    let account = Account::create(Hbar::new(0), &client).await?;

    let res = TokenRevokeKycTransaction::new()
        .account_id(account.id)
        .sign(account.key.clone())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::InvalidTokenId,
            transaction_id: _
        })
    );

    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn missing_account_id_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else { return Ok(()) };

    let account = Account::create(Hbar::new(0), &client).await?;
    let token = super::FungibleToken::create(&client, &account, 0).await?;

    let res = TokenRevokeKycTransaction::new()
        .token_id(token.id)
        .sign(account.key.clone())
        .execute(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::TransactionPreCheckStatus {
            status: Status::InvalidAccountId,
            transaction_id: _
        })
    );

    token.delete(&client).await?;
    account.delete(&client).await?;

    Ok(())
}

#[tokio::test]
async fn non_associated_token_fails() -> anyhow::Result<()> {
    let Some(TestEnvironment { config: _, client }) = setup_nonfree() else { return Ok(()) };

    let (alice, bob) = tokio::try_join!(
        Account::create(Hbar::new(0), &client),
        Account::create(Hbar::new(0), &client)
    )?;

    let token = super::FungibleToken::create(&client, &alice, 0).await?;

    let res = TokenRevokeKycTransaction::new()
        .account_id(bob.id)
        .token_id(token.id)
        .sign(alice.key.clone())
        .execute(&client)
        .await?
        .get_receipt(&client)
        .await;

    assert_matches!(
        res,
        Err(hedera::Error::ReceiptStatus {
            status: Status::TokenNotAssociatedToAccount,
            transaction_id: _
        })
    );

    token.delete(&client).await?;

    tokio::try_join!(alice.delete(&client), bob.delete(&client))?;

    Ok(())
}
