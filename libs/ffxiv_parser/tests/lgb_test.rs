#[cfg(test)]
mod tests {
    use ffxiv_parser::Lgb;
    use sqpack_reader::{ExtractedFileProviderWeb, Result, SqPackReaderExtractedFile};

    #[tokio::test]
    async fn lgb_test() -> Result<()> {
        let _ = pretty_env_logger::formatted_timed_builder()
            .filter(Some("sqpack_reader"), log::LevelFilter::Debug)
            .try_init();

        let provider = ExtractedFileProviderWeb::new("https://ffxiv-data.dlunch.net/compressed/");
        let pack = SqPackReaderExtractedFile::new(provider)?;

        let lgb = Lgb::new(&pack, "bg/ffxiv/sea_s1/twn/s1t1/level/planner.lgb").await?;
        assert_eq!(lgb.name, "Planner");

        Ok(())
    }
}