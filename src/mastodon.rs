use std::{path::Path, time::Duration};

use anyhow::{anyhow, Result};
use megalodon::{
    entities::{Attachment, StatusVisibility, UploadMedia},
    megalodon::{PostStatusInputOptions, PostStatusOutput, UploadMediaInputOptions},
    Megalodon,
};
use tokio::time;

pub struct Client {
    client: Box<dyn Megalodon + Send + Sync>,
}

impl Client {
    pub fn new(instance_url: String, access_token: String) -> Result<Client> {
        Ok(Client {
            client: megalodon::generator(
                megalodon::SNS::Mastodon,
                instance_url,
                Some(access_token),
                None,
            )?,
        })
    }

    pub async fn get_notifications(
        &self,
    ) -> Result<Vec<(String, String, (String, String), String)>> {
        Ok(self
            .client
            .get_notifications(None)
            .await?
            .json
            .iter()
            .filter(|n| n.account.is_some() && n.status.is_some())
            .map(|n| {
                let status = n.status.as_ref().unwrap();
                let account = n.account.as_ref().unwrap();
                (
                    n.id.clone(),
                    status.id.clone(),
                    (account.acct.clone(), account.url.clone()),
                    status.content.clone(),
                )
            })
            .collect())
    }

    pub async fn clear_notification(&self, id: &str) -> Result<()> {
        self.client.dismiss_notification(id.into()).await?;
        Ok(())
    }

    async fn wait_until_media_uploaded(&self, id: &str) -> Result<Attachment> {
        loop {
            let res = self.client.get_media(id.to_string()).await;
            return match res {
                Ok(res) => Ok(res.json()),
                Err(err) => match err {
                    megalodon::error::Error::OwnError(ref own_err) => match own_err.kind {
                        megalodon::error::Kind::HTTPPartialContentError => {
                            // wait a few seconds to see if the file has already been uploaded
                            time::sleep(Duration::from_secs(5)).await;
                            continue;
                        }
                        _ => Err(err.into()),
                    },
                    _ => Err(err.into()),
                },
            };
        }
    }

    pub async fn post_result(
        &self,
        username: &str,
        id: &str,
        video_path: impl AsRef<Path>,
    ) -> Result<String> {
        let media = self
            .client
            .upload_media(
                video_path.as_ref().to_string_lossy().into(),
                Some(&UploadMediaInputOptions {
                    description: Some("Result of the Orca code".into()),
                    focus: None,
                }),
            )
            .await?
            .json;

        log::info!("Uploading media");

        let res = match media {
            UploadMedia::Attachment(a) => a.id,
            UploadMedia::AsyncAttachment(a) => {
                log::debug!("Waiting for media {} to be uploaded...", a.id);
                // wait for file to be uploaded
                self.wait_until_media_uploaded(&a.id).await?;
                a.id
            }
        };
        log::info!("Media {} uploaded", res);

        let status = format!("I ran @{username}'s program and here's the result!");

        let status = self
            .client
            .post_status(
                status,
                Some(&PostStatusInputOptions {
                    media_ids: Some(vec![res]),
                    poll: None,
                    in_reply_to_id: Some(id.into()),
                    sensitive: Some(false),
                    spoiler_text: None,
                    visibility: Some(StatusVisibility::Public),
                    scheduled_at: None,
                    language: Some("en".into()),
                    quote_id: None,
                }),
            )
            .await?
            .json;

        match status {
            PostStatusOutput::Status(status) => Ok(status.url.unwrap()),
            PostStatusOutput::ScheduledStatus(_scheduled_status) => {
                Err(anyhow!("Shouldn't be getting a scheduled status!"))
            }
        }
    }
}
