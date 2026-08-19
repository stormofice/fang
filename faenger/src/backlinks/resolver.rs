use crate::DbPool;
use crate::backlinks::join_backlinks;
use crate::backlinks::models::Backlink;
use crate::db_utils::coalesce;
use anyhow::{anyhow, bail};
use diesel::RunQueryDsl;
use diesel::upsert::excluded;
use diesel::{ExpressionMethods, QueryDsl};
use reqwest::Client;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::mpsc::Receiver;
use url::Url;

pub struct BacklinkResolver {
    http_client: Client,
    db: DbPool,
}

impl BacklinkResolver {
    pub fn new(db: DbPool) -> Self {
        Self {
            // TODO: Custom user-agent?
            http_client: Client::new(),
            db,
        }
    }

    // Hmm, not sure how happy I am with the Arc<Self> here.
    pub fn schedule_backlink_resolver(self: Arc<Self>, mut rx: Receiver<String>) {
        tokio::spawn(async move {
            log::info!("Starting backlink resolver");
            while let Some(url) = rx.recv().await {
                log::info!("should resolve new backlink: {url}");

                let backlinks = self.retrieve_backlinks(url.as_str()).await;
                match backlinks {
                    Ok(backlinks) => {
                        log::debug!("Resolved to: {:?}", backlinks);
                    }
                    Err(err) => {
                        log::error!("Failed to resolve backlink {url}: {err}");
                    }
                }
            }
        });
    }

    // This function tries to resolve backlinks for the given url. If there are any, they are stored in
    // the database and subsequently returned.
    pub async fn retrieve_backlinks(&self, request_url: &str) -> anyhow::Result<Option<Backlink>> {
        use crate::schema::backlinks::dsl::*;

        let mut db = self.db.get()?;

        let existing_backlinks: Vec<Backlink> =
            backlinks.filter(url.eq(request_url)).load(&mut db)?;
        match existing_backlinks.len() {
            // If there are none, try to find them
            0 => {
                let lobster_backlinks =
                    resolve_lobster_backlinks(&self.http_client, request_url).await?;

                let backlink = Backlink {
                    url: request_url.to_string(),
                    lobsters_links: lobster_backlinks.map(join_backlinks),
                    hn_links: None,
                };

                diesel::insert_into(backlinks)
                    .values(&backlink)
                    .on_conflict(url)
                    .do_update()
                    .set((
                        lobsters_links.eq(coalesce(lobsters_links, excluded(lobsters_links))),
                        hn_links.eq(coalesce(hn_links, excluded(hn_links))),
                    ))
                    .execute(&mut db)?;

                Ok(Some(backlink))
            }
            // If a single backlink is found, return it
            1 => Ok(Some(existing_backlinks.first().cloned().unwrap())),
            // If there are multiple, gg
            _ => {
                log::error!("Multiple backlinks found for: {request_url}");
                bail!("Multiple backlinks found for: {request_url}")
            }
        }
    }
}

#[derive(Deserialize, Debug)]
struct LobsterDomainResult {
    url: String,
    comments_url: String,
}

async fn resolve_lobster_backlinks(
    http_client: &Client,
    url: &str,
) -> anyhow::Result<Option<Vec<String>>> {
    let url = Url::parse(url)?;

    if let Some(host) = url.host_str() {
        // According to my testing, lobsters filters/does not expect a "www" subdomain. Thus, we
        // also filter it here.
        let host = host.strip_prefix("www.").unwrap_or(host);

        const LOBSTERS_SEARCH_URL: &str = "https://lobste.rs/domains/";
        let lobster_search = format!("{}{}.json", LOBSTERS_SEARCH_URL, host);

        log::debug!("Searching on lobsters: {lobster_search:?}");

        let resp = http_client.get(lobster_search).send().await?;

        match resp.status().as_u16() {
            200 => {
                let entries = resp.json::<Vec<LobsterDomainResult>>().await?;

                log::debug!("Lobsters backlinks found for {url:?}: {entries:?}");

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

                if backlinks.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(backlinks))
                }
            }
            404 => {
                log::debug!("No lobsters backlinks found for {url:?}");
                Ok(None)
            }
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
