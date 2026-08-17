use crate::DbPool;
use anyhow::anyhow;
use diesel::RunQueryDsl;
use diesel::{ExpressionMethods, QueryDsl};
use reqwest::Client;
use serde::Deserialize;
use tokio::sync::mpsc::Receiver;
use url::Url;

pub fn create_backlink_resolver(mut rx: Receiver<String>, mut db: DbPool) {
    tokio::spawn(async move {
        // TODO: Custom user-agent?
        let http_client = Client::new();

        log::info!("Starting backlink resolver");
        while let Some(url) = rx.recv().await {
            log::info!("should resolve new backlink: {url}");

            let known = has_backlinks(&mut db, &url).await.unwrap_or_else(|err| {
                log::error!("Encountered error while resolving backlink: {:?}", err);
                true
            });

            if known {
                // TODO: Re-resolve periodically?
                continue;
            }

            match Url::parse(&url) {
                Ok(url) => {
                    let ls_backlinks = get_lobsters_backlinks(&http_client, &url).await;
                    match ls_backlinks {
                        Ok(ls_backlinks) => {
                            if let Some(ls_backlinks) = ls_backlinks {
                                match store_backlinks(&mut db, &url, ls_backlinks).await {
                                    Ok(_) => {}
                                    Err(err) => {
                                        log::warn!(
                                            "Encountered error while storing backlinks: \
                                        {:?}",
                                            err
                                        )
                                    }
                                };
                            }
                        }
                        Err(err) => {
                            log::warn!("Could not retrieve lobsters backlinks: {:?}", err);
                        }
                    }
                }
                Err(err) => {
                    log::warn!("Failed to parse link during backlink resolve: {url}: {err:?}");
                }
            }
        }
    });
}

async fn has_backlinks(db: &mut DbPool, req_url: &String) -> anyhow::Result<bool> {
    use crate::schema::backlinks::dsl::*;

    let mut db = db.get()?;

    match backlinks.find(req_url).execute(&mut db)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => {
            log::warn!("More than one backlink for: {req_url}");
            // TODO: Fail save for now and don't re-resolve. Should think about if we can
            // eliminate all these "exists more than one cases". They should not be able to
            // happen no?
            Ok(true)
        }
    }
}

async fn store_backlinks(
    db: &mut DbPool,
    og_url: &Url,
    ls_backlinks: Vec<String>,
) -> anyhow::Result<()> {
    use crate::schema::backlinks::dsl::*;

    let mut db = db.get()?;

    let joined = <[String]>::join(&ls_backlinks, "🙂‍↕️");

    diesel::insert_into(backlinks)
        .values((url.eq(og_url.as_str()), lobsters_links.eq(joined)))
        .execute(&mut db)?;

    Ok(())
}

#[derive(Deserialize)]
struct LobsterDomainResult {
    url: String,
    comments_url: String,
}

pub async fn get_lobsters_backlinks(
    http_client: &Client,
    url: &Url,
) -> anyhow::Result<Option<Vec<String>>> {
    if let Some(host) = url.host_str() {
        const LOBSTERS_SEARCH_URL: &str = "https://lobste.rs/domains/";
        let lobster_search = format!("{}{}.json", LOBSTERS_SEARCH_URL, host);

        let resp = http_client.get(lobster_search).send().await?;

        match resp.status().as_u16() {
            200 => {
                let entries = resp.json::<Vec<LobsterDomainResult>>().await?;

                let mut backlinks: Vec<String> = Vec::new();

                for entry in entries {
                    if url.as_str() == entry.url {
                        backlinks.push(entry.comments_url);
                    }
                }

                log::debug!(
                    "Resolved lobsters backlinks for {:?}: {:?}",
                    url.as_str(),
                    backlinks
                );

                Ok(Some(backlinks))
            }
            404 => Ok(None),
            _ => {
                log::warn!(
                    "Unexpected lobsters status code {:?} while resolve of {}",
                    resp.status(),
                    url
                );
                Err(anyhow!("Lobsters issue"))
            }
        }
    } else {
        Err(anyhow!("Could not determine host of url: {}", url))
    }
}
