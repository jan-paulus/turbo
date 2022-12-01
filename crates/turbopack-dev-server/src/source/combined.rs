use anyhow::Result;
use turbo_tasks::{primitives::StringVc, TryJoinIterExt, Value};
use turbopack_core::introspect::{Introspectable, IntrospectableChildrenVc, IntrospectableVc};

use super::{
    specificity::SpecificityReadRef, ContentSource, ContentSourceData, ContentSourceResultVc,
    ContentSourceVc,
};

/// Combines multiple [ContentSource]s by trying all content sources in order.
/// First [ContentSource] that responds with something other than NotFound will
/// serve the request.
#[turbo_tasks::value(shared)]
pub struct CombinedContentSource {
    pub sources: Vec<ContentSourceVc>,
}

impl CombinedContentSourceVc {
    pub fn new(sources: Vec<ContentSourceVc>) -> Self {
        CombinedContentSource { sources }.cell()
    }
}

#[turbo_tasks::value_impl]
impl ContentSource for CombinedContentSource {
    #[turbo_tasks::function]
    async fn get(
        &self,
        path: &str,
        data: Value<ContentSourceData>,
    ) -> Result<ContentSourceResultVc> {
        let mut max: Option<(SpecificityReadRef, ContentSourceResultVc)> = None;
        for source in self.sources.iter() {
            let result = source.get(path, data.clone());
            let specificity = result.await?.specificity.await?;
            if specificity.is_exact() {
                return Ok(result);
            }
            if let Some((max, _)) = max.as_ref() {
                if *max >= specificity {
                    // we can keep the current max
                    continue;
                }
            }
            max = Some((specificity, result));
        }
        if let Some((_, result)) = max {
            Ok(result)
        } else {
            Ok(ContentSourceResultVc::not_found())
        }
    }
}

#[turbo_tasks::function]
fn introspectable_type() -> StringVc {
    StringVc::cell("combined content source".to_string())
}

#[turbo_tasks::value_impl]
impl Introspectable for CombinedContentSource {
    #[turbo_tasks::function]
    fn ty(&self) -> StringVc {
        introspectable_type()
    }

    #[turbo_tasks::function]
    async fn title(&self) -> Result<StringVc> {
        let titles = self
            .sources
            .iter()
            .map(|&source| async move {
                Ok(
                    if let Some(source) = IntrospectableVc::resolve_from(source).await? {
                        Some(source.title().await?)
                    } else {
                        None
                    },
                )
            })
            .try_join()
            .await?;
        let mut titles = titles.into_iter().flatten().collect::<Vec<_>>();
        titles.sort();
        const NUMBER_OF_TITLES_TO_DISPLAY: usize = 5;
        let mut titles = titles
            .iter()
            .map(|t| t.as_str())
            .filter(|t| !t.is_empty())
            .take(NUMBER_OF_TITLES_TO_DISPLAY + 1)
            .collect::<Vec<_>>();
        if titles.len() > NUMBER_OF_TITLES_TO_DISPLAY {
            titles[NUMBER_OF_TITLES_TO_DISPLAY] = "...";
        }
        Ok(StringVc::cell(titles.join(", ")))
    }

    #[turbo_tasks::function]
    async fn children(&self) -> Result<IntrospectableChildrenVc> {
        let source = StringVc::cell("source".to_string());
        Ok(IntrospectableChildrenVc::cell(
            self.sources
                .iter()
                .copied()
                .map(|s| async move { Ok(IntrospectableVc::resolve_from(s).await?) })
                .try_join()
                .await?
                .into_iter()
                .flatten()
                .map(|i| (source, i))
                .collect(),
        ))
    }
}